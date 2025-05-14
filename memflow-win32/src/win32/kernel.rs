mod mem_map;

use crate::{
    offsets::{Win32ArchOffsets, Win32Offsets},
    prelude::{VirtualReadUnicodeString, Win32ExitStatus, EXIT_STATUS_STILL_ACTIVE},
};

use super::{
    process::IMAGE_FILE_NAME_LENGTH, Win32KernelBuilder, Win32KernelInfo, Win32Keyboard,
    Win32ModuleListInfo, Win32Process, Win32ProcessInfo, Win32VirtualTranslate,
};

use memflow::mem::virt_translate::*;
use memflow::prelude::v1::{Result, *};

#[cfg(feature = "plugins")]
use memflow::cglue;
#[cfg(feature = "plugins")]
use memflow::mem::{memory_view::*, phys_mem::*};
#[cfg(feature = "plugins")]
use memflow::os::keyboard::*;

use log::{info, trace};
use std::convert::TryInto;
use std::fmt;
use std::prelude::v1::*;

use muddy::muddy;

use pelite::{self, pe64::exports::Export, PeView};

const MAX_ITER_COUNT: usize = 65536;

#[cfg(feature = "plugins")]
cglue_impl_group!(Win32Kernel<T, V>, OsInstance<'a>, { PhysicalMemory, MemoryView, VirtualTranslate, OsKeyboard });

#[derive(Clone)]
pub struct Win32Kernel<T, V> {
    pub virt_mem: VirtualDma<T, V, Win32VirtualTranslate>,
    pub offsets: Win32Offsets,

    pub kernel_info: Win32KernelInfo,
    pub sysproc_dtb: Address,

    pub kernel_modules: Option<Win32ModuleListInfo>,
}

impl<T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone>
    Win32Kernel<T, V>
{
    pub fn new(phys_mem: T, vat: V, offsets: Win32Offsets, kernel_info: Win32KernelInfo) -> Self {
        let mut virt_mem = VirtualDma::with_vat(
            phys_mem,
            kernel_info.os_info.arch,
            Win32VirtualTranslate::new(kernel_info.os_info.arch, kernel_info.dtb),
            vat,
        );

        if offsets.phys_mem_block() != 0 {
            match kernel_info.os_info.arch.into_obj().bits() {
                32 => {
                    if let Some(mem_map) = mem_map::parse::<_, u32>(
                        &mut virt_mem,
                        kernel_info.os_info.base + offsets.phys_mem_block(),
                    ) {
                        // update mem mapping in connector
                        info!("updating connector mem_map={:?}", mem_map);
                        let (mut phys_mem, vat) = virt_mem.into_inner();
                        phys_mem.set_mem_map(mem_map.into_vec().as_slice());
                        virt_mem = VirtualDma::with_vat(
                            phys_mem,
                            kernel_info.os_info.arch,
                            Win32VirtualTranslate::new(kernel_info.os_info.arch, kernel_info.dtb),
                            vat,
                        );
                    }
                }
                64 => {
                    if let Some(mem_map) = mem_map::parse::<_, u64>(
                        &mut virt_mem,
                        kernel_info.os_info.base + offsets.phys_mem_block(),
                    ) {
                        // update mem mapping in connector
                        info!("updating connector mem_map={:?}", mem_map);
                        let (mut phys_mem, vat) = virt_mem.into_inner();
                        phys_mem.set_mem_map(mem_map.into_vec().as_slice());
                        virt_mem = VirtualDma::with_vat(
                            phys_mem,
                            kernel_info.os_info.arch,
                            Win32VirtualTranslate::new(kernel_info.os_info.arch, kernel_info.dtb),
                            vat,
                        );
                    }
                }
                _ => {}
            }
        }

        // start_block only contains the winload's dtb which might
        // be different to the one used in the actual kernel.
        // In case of a failure this will fall back to the winload dtb.
        // Read dtb of first process in eprocess list:
        let sysproc_dtb = if let Some(Some(dtb)) = virt_mem
            .read_addr_arch(
                kernel_info.os_info.arch.into(),
                kernel_info.eprocess_base + offsets.kproc_dtb(),
            )
            .ok()
            .map(|a| a.as_page_aligned(4096).non_null())
        {
            info!("updating sysproc_dtb={:x}", dtb);
            let (phys_mem, vat) = virt_mem.into_inner();
            virt_mem = VirtualDma::with_vat(
                phys_mem,
                kernel_info.os_info.arch,
                Win32VirtualTranslate::new(kernel_info.os_info.arch, dtb),
                vat,
            );
            dtb
        } else {
            kernel_info.dtb
        };

        Self {
            virt_mem,
            offsets,

            kernel_info,
            sysproc_dtb,
            kernel_modules: None,
        }
    }

    pub fn kernel_modules(&mut self) -> Result<Win32ModuleListInfo> {
        if let Some(info) = self.kernel_modules {
            Ok(info)
        } else {
            let image = self.virt_mem.read_raw(
                self.kernel_info.os_info.base,
                self.kernel_info.os_info.size.try_into().unwrap(),
            )?;
            let pe = PeView::from_bytes(&image).map_err(|err| {
                Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile).log_info(err)
            })?;
            let addr = match pe.get_export_by_name(muddy!("PsLoadedModuleList")).map_err(|err| {
                Error(ErrorOrigin::OsLayer, ErrorKind::ExportNotFound).log_info(err)
            })? {
                Export::Symbol(s) => self.kernel_info.os_info.base + *s as umem,
                Export::Forward(_) => {
                    return Err(Error(ErrorOrigin::OsLayer, ErrorKind::ExportNotFound)
                        .log_info(muddy!("PsLoadedModuleList found but it was a forwarded export")))
                }
            };

            let addr = self
                .virt_mem
                .read_addr_arch(self.kernel_info.os_info.arch.into(), addr)?;

            let info = Win32ModuleListInfo::with_base(addr, self.kernel_info.os_info.arch)?;

            self.kernel_modules = Some(info);
            Ok(info)
        }
    }

    /// Consumes this kernel and return the underlying owned memory and vat objects
    pub fn into_inner(self) -> (T, V) {
        self.virt_mem.into_inner()
    }

    pub fn kernel_process_info(&mut self) -> Result<Win32ProcessInfo> {
        let kernel_modules = self.kernel_modules()?;

        let vad_root = self.read_addr_arch(
            self.kernel_info.os_info.arch.into(),
            self.kernel_info.os_info.base + self.offsets.eproc_vad_root(),
        )?;

        Ok(Win32ProcessInfo {
            base_info: ProcessInfo {
                address: self.kernel_info.os_info.base,
                pid: 0,
                state: ProcessState::Alive,
                name: muddy!("ntoskrnl.exe").into(),
                path: "".into(),
                command_line: "".into(),
                sys_arch: self.kernel_info.os_info.arch,
                proc_arch: self.kernel_info.os_info.arch,
                dtb1: self.sysproc_dtb,
                dtb2: Address::invalid(),
            },
            section_base: Address::NULL, // TODO: see below
            ethread: Address::NULL,      // TODO: see below
            wow64: Address::NULL,

            teb: None,
            teb_wow64: None,

            peb_native: None,
            peb_wow64: None,

            module_info_native: Some(kernel_modules),
            module_info_wow64: None,

            vad_root,
        })
    }

    pub fn process_info_from_base_info(
        &mut self,
        base_info: ProcessInfo,
    ) -> Result<Win32ProcessInfo> {
        let section_base = self.virt_mem.read_addr_arch(
            self.kernel_info.os_info.arch.into(),
            base_info.address + self.offsets.eproc_section_base(),
        )?;
        trace!("section_base={:x}", section_base);

        // find first ethread
        let ethread = self.virt_mem.read_addr_arch(
            self.kernel_info.os_info.arch.into(),
            base_info.address + self.offsets.eproc_thread_list(),
        )? - self.offsets.ethread_list_entry();
        trace!("ethread={:x}", ethread);

        let peb_native = self
            .virt_mem
            .read_addr_arch(
                self.kernel_info.os_info.arch.into(),
                base_info.address + self.offsets.eproc_peb(),
            )?
            .non_null();

        // TODO: Avoid doing this twice
        let wow64 = if self.offsets.eproc_wow64() == 0 {
            trace!("eproc_wow64=null; skipping wow64 detection");
            Address::null()
        } else {
            trace!(
                "eproc_wow64={:x}; trying to read wow64 pointer",
                self.offsets.eproc_wow64()
            );
            self.virt_mem.read_addr_arch(
                self.kernel_info.os_info.arch.into(),
                base_info.address + self.offsets.eproc_wow64(),
            )?
        };
        trace!("wow64={:x}", wow64);

        let mut peb_wow64 = None;

        // TODO: does this need to be read with the process ctx?
        let (teb, teb_wow64) = if self.kernel_info.kernel_winver >= (6, 2).into() {
            let teb = self.virt_mem.read_addr_arch(
                self.kernel_info.os_info.arch.into(),
                ethread + self.offsets.kthread_teb(),
            )?;

            trace!("teb={:x}", teb);

            if !teb.is_null() {
                (
                    Some(teb),
                    if base_info.proc_arch == base_info.sys_arch {
                        None
                    } else {
                        Some(teb + 0x2000)
                    },
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let vad_root = self.virt_mem.read_addr_arch(
            self.kernel_info.os_info.arch.into(),
            base_info.address + self.offsets.eproc_vad_root(),
        )?;

        // construct reader with process dtb - win32 only uses/requires one dtb so we always store it in `dtb1`
        // TODO: can tlb be used here already?
        let (phys_mem, vat) = self.virt_mem.mem_vat_pair();
        let mut proc_reader = VirtualDma::with_vat(
            phys_mem.forward_mut(),
            base_info.proc_arch,
            Win32VirtualTranslate::new(self.kernel_info.os_info.arch, base_info.dtb1),
            vat,
        );

        if let Some(teb) = teb_wow64 {
            // from here on out we are in the process context
            // we will be using the process type architecture now
            peb_wow64 = proc_reader
                .read_addr_arch(
                    self.kernel_info.os_info.arch.into(),
                    teb + self.offsets.teb_peb_x86(),
                )?
                .non_null();

            trace!("peb_wow64={:?}", peb_wow64);
        }

        trace!("peb_native={:?}", peb_native);

        let module_info_native = peb_native
            .map(|peb| Win32ModuleListInfo::with_peb(&mut proc_reader, peb, base_info.sys_arch))
            .transpose()?;

        let module_info_wow64 = peb_wow64
            .map(|peb| Win32ModuleListInfo::with_peb(&mut proc_reader, peb, base_info.proc_arch))
            .transpose()?;

        Ok(Win32ProcessInfo {
            base_info,

            section_base,
            ethread,
            wow64,

            teb,
            teb_wow64,

            peb_native,
            peb_wow64,

            module_info_native,
            module_info_wow64,

            vad_root,
        })
    }

    fn process_info_fill(&mut self, info: Win32ProcessInfo) -> Result<Win32ProcessInfo> {
        // get full process name from module list
        let cloned_base = info.base_info.clone();
        let mut name = info.base_info.name.clone();
        let callback = &mut |m: ModuleInfo| {
            if m.name.as_ref().starts_with(name.as_ref()) {
                name = m.name;
                false
            } else {
                true
            }
        };
        let sys_arch = info.base_info.sys_arch;
        let mut process = self.process_by_info(cloned_base)?;
        process.module_list_callback(Some(&sys_arch), callback.into())?;

        // get process_parameters
        let offsets = Win32ArchOffsets::from(info.base_info.proc_arch);
        let (path, command_line) = if let Some(Ok(peb_process_params)) = info.peb().map(|peb| {
            process.read_addr_arch(
                info.base_info.proc_arch.into(),
                peb + offsets.peb_process_params,
            )
        }) {
            trace!("peb_process_params={:x}", peb_process_params);
            let image_path_name = process
                .read_unicode_string(
                    info.base_info.proc_arch.into(),
                    peb_process_params + offsets.ppm_image_path_name,
                )
                .unwrap_or_default();

            let command_line = process
                .read_unicode_string(
                    info.base_info.proc_arch.into(),
                    peb_process_params + offsets.ppm_command_line,
                )
                .unwrap_or_default();

            (image_path_name.into(), command_line.into())
        } else {
            ("".into(), "".into())
        };

        Ok(Win32ProcessInfo {
            base_info: ProcessInfo {
                name,
                path,
                command_line,
                ..info.base_info
            },
            ..info
        })
    }

    fn process_info_base_by_address(&mut self, address: Address) -> Result<ProcessInfo> {
        let dtb = self.virt_mem.read_addr_arch(
            self.kernel_info.os_info.arch.into(),
            address + self.offsets.kproc_dtb(),
        )?;
        trace!("dtb={:x}", dtb);

        let pid: Pid = self.virt_mem.read(address + self.offsets.eproc_pid())?;
        trace!("pid={}", pid);

        let state = if let Ok(exit_status) = self
            .virt_mem
            .read::<Win32ExitStatus>(address + self.offsets.eproc_exit_status())
        {
            if exit_status == EXIT_STATUS_STILL_ACTIVE {
                ProcessState::Alive
            } else {
                ProcessState::Dead(exit_status)
            }
        } else {
            ProcessState::Unknown
        };

        let name: ReprCString = self
            .virt_mem
            .read_utf8_lossy(address + self.offsets.eproc_name(), IMAGE_FILE_NAME_LENGTH)?
            .into();
        trace!("name={}", name);

        let wow64 = if self.offsets.eproc_wow64() == 0 {
            trace!("eproc_wow64=null; skipping wow64 detection");
            Address::null()
        } else {
            trace!(
                "eproc_wow64={:x}; trying to read wow64 pointer",
                self.offsets.eproc_wow64()
            );
            self.virt_mem.read_addr_arch(
                self.kernel_info.os_info.arch.into(),
                address + self.offsets.eproc_wow64(),
            )?
        };
        trace!("wow64={:x}", wow64);

        // determine process architecture
        let sys_arch = self.kernel_info.os_info.arch;
        trace!("sys_arch={:?}", sys_arch);
        let proc_arch = match ArchitectureObj::from(sys_arch).bits() {
            64 => {
                if wow64.is_null() {
                    sys_arch
                } else {
                    ArchitectureIdent::X86(32, true)
                }
            }
            32 => sys_arch,
            _ => return Err(Error(ErrorOrigin::OsLayer, ErrorKind::InvalidArchitecture)),
        };
        trace!("proc_arch={:?}", proc_arch);

        Ok(ProcessInfo {
            address,
            pid,
            state,
            name,
            path: "".into(),
            command_line: "".into(),
            sys_arch,
            proc_arch,
            dtb1: dtb,
            dtb2: Address::invalid(),
        })
    }
}

impl<T: PhysicalMemory> Win32Kernel<T, DirectTranslate> {
    pub fn builder(connector: T) -> Win32KernelBuilder<T, T, DirectTranslate> {
        Win32KernelBuilder::<T, T, DirectTranslate>::new(connector)
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> AsMut<T> for Win32Kernel<T, V> {
    fn as_mut(&mut self) -> &mut T {
        self.virt_mem.phys_mem()
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> AsMut<VirtualDma<T, V, Win32VirtualTranslate>>
    for Win32Kernel<T, V>
{
    fn as_mut(&mut self) -> &mut VirtualDma<T, V, Win32VirtualTranslate> {
        &mut self.virt_mem
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> PhysicalMemory for Win32Kernel<T, V> {
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()> {
        self.virt_mem.phys_mem().phys_read_raw_iter(data)
    }

    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        self.virt_mem.phys_mem().phys_write_raw_iter(data)
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.virt_mem.phys_mem_ref().metadata()
    }

    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]) {
        self.virt_mem.phys_mem().set_mem_map(mem_map)
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> MemoryView for Win32Kernel<T, V> {
    fn read_raw_iter(&mut self, data: ReadRawMemOps) -> Result<()> {
        self.virt_mem.read_raw_iter(data)
    }

    fn write_raw_iter(&mut self, data: WriteRawMemOps) -> Result<()> {
        self.virt_mem.write_raw_iter(data)
    }

    fn metadata(&self) -> MemoryViewMetadata {
        self.virt_mem.metadata()
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> VirtualTranslate for Win32Kernel<T, V> {
    fn virt_to_phys_list(
        &mut self,
        addrs: &[VtopRange],
        out: VirtualTranslationCallback,
        out_fail: VirtualTranslationFailCallback,
    ) {
        self.virt_mem.virt_to_phys_list(addrs, out, out_fail)
    }
}

impl<T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone> Os
    for Win32Kernel<T, V>
{
    type ProcessType<'a> = Win32Process<Fwd<&'a mut T>, Fwd<&'a mut V>, Win32VirtualTranslate>;
    type IntoProcessType = Win32Process<T, V, Win32VirtualTranslate>;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(
        &mut self,
        mut callback: AddressCallback,
    ) -> memflow::error::Result<()> {
        let list_start = self.kernel_info.eprocess_base + self.offsets.eproc_link();
        let mut list_entry = list_start;

        for _ in 0..MAX_ITER_COUNT {
            let eprocess = list_entry - self.offsets.eproc_link();
            trace!("eprocess={}", eprocess);

            // test flink + blink before adding the process
            let flink_entry = self
                .virt_mem
                .read_addr_arch(self.kernel_info.os_info.arch.into(), list_entry)?;
            trace!("flink_entry={}", flink_entry);
            let blink_entry = self.virt_mem.read_addr_arch(
                self.kernel_info.os_info.arch.into(),
                list_entry + self.offsets.list_blink(),
            )?;
            trace!("blink_entry={}", blink_entry);

            if flink_entry.is_null()
                || blink_entry.is_null()
                || flink_entry == list_start
                || flink_entry == list_entry
            {
                break;
            }

            trace!("found eprocess {:x}", eprocess);
            if !callback.call(eprocess) {
                break;
            }
            trace!("Continuing {:x} -> {:x}", list_entry, flink_entry);

            // continue
            list_entry = flink_entry;
        }

        Ok(())
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> memflow::error::Result<ProcessInfo> {
        let base_info = self.process_info_base_by_address(address)?;
        if let Ok(info) = self.process_info_from_base_info(base_info.clone()) {
            Ok(self.process_info_fill(info)?.base_info)
        } else {
            Ok(base_info)
        }
    }

    /// Creates a process by its internal address
    ///
    /// It will share the underlying memory resources
    fn process_by_info(
        &mut self,
        info: ProcessInfo,
    ) -> memflow::error::Result<Self::ProcessType<'_>> {
        let proc_info = self.process_info_from_base_info(info)?;
        Ok(Win32Process::with_kernel_ref(self, proc_info))
    }

    /// Creates a process by its internal address
    ///
    /// It will consume the kernel and not affect memory usage
    ///
    /// If no process with the specified address can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_info(
        mut self,
        info: ProcessInfo,
    ) -> memflow::error::Result<Self::IntoProcessType> {
        let proc_info = self.process_info_from_base_info(info)?;
        Ok(Win32Process::with_kernel(self, proc_info))
    }

    /// Walks the kernel module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(
        &mut self,
        callback: AddressCallback,
    ) -> memflow::error::Result<()> {
        self.kernel_modules()?
            .module_entry_list_callback::<Self, VirtualDma<T, V, Win32VirtualTranslate>>(
                self,
                self.kernel_info.os_info.arch,
                callback,
            )
            .map_err(From::from)
    }

    /// Retrieves a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> memflow::error::Result<ModuleInfo> {
        self.kernel_modules()?
            .module_info_from_entry(
                address,
                self.kernel_info.eprocess_base,
                &mut self.virt_mem,
                self.kernel_info.os_info.arch,
            )
            .map_err(From::from)
    }

    /// Retrieves address of the primary module structure of the process
    ///
    /// This will generally be for the initial executable that was run
    fn primary_module_address(&mut self) -> Result<Address> {
        Ok(self.module_by_name(muddy!("ntoskrnl.exe"))?.address)
    }

    /// Retrieves information for the primary module of the process
    ///
    /// This will generally be the initial executable that was run
    fn primary_module(&mut self) -> Result<ModuleInfo> {
        self.module_by_name(muddy!("ntoskrnl.exe"))
    }

    /// Retrieves a list of all imports of a given module
    fn module_import_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ImportCallback,
    ) -> Result<()> {
        memflow::os::util::module_import_list_callback(&mut self.virt_mem, info, callback)
    }

    /// Retrieves a list of all exports of a given module
    fn module_export_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ExportCallback,
    ) -> Result<()> {
        memflow::os::util::module_export_list_callback(&mut self.virt_mem, info, callback)
    }

    /// Retrieves a list of all sections of a given module
    fn module_section_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: SectionCallback,
    ) -> Result<()> {
        memflow::os::util::module_section_list_callback(&mut self.virt_mem, info, callback)
    }

    /// Retrieves the kernel info
    fn info(&self) -> &OsInfo {
        &self.kernel_info.os_info
    }
}

impl<T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone> OsKeyboard
    for Win32Kernel<T, V>
{
    type KeyboardType<'a> =
        Win32Keyboard<VirtualDma<Fwd<&'a mut T>, Fwd<&'a mut V>, Win32VirtualTranslate>>;
    type IntoKeyboardType = Win32Keyboard<VirtualDma<T, V, Win32VirtualTranslate>>;

    fn keyboard(&mut self) -> memflow::error::Result<Self::KeyboardType<'_>> {
        Win32Keyboard::with_kernel_ref(self)
    }

    fn into_keyboard(self) -> memflow::error::Result<Self::IntoKeyboardType> {
        Win32Keyboard::with_kernel(self)
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate2> fmt::Debug for Win32Kernel<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kernel_info)
    }
}
