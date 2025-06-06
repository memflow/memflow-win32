/*!
Module for reading a target's keyboard state.

The `gafAsyncKeyState` array contains the current Keyboard state on Windows targets.
This array will internally be read by the [`GetAsyncKeyState()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate) function of Windows.

Although the gafAsyncKeyState array is exported by the win32kbase.sys kernel module it is only properly mapped into user mode processes.
Therefor the Keyboard will by default find the winlogon.exe or wininit.exe process and use it as a proxy to read the data.

# Examples:

```
use std::{thread, time};

use memflow::mem::{PhysicalMemory, VirtualTranslate2};
use memflow::os::{Keyboard, KeyboardState};
use memflow_win32::win32::{Win32Kernel, Win32Keyboard};

fn test<T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone>(kernel: &mut Win32Kernel<T, V>) {
    let mut kbd = Win32Keyboard::with_kernel_ref(kernel).unwrap();

    loop {
        let kbs = kbd.state().unwrap();
        println!("space down: {:?}", kbs.is_down(0x20)); // VK_SPACE
        thread::sleep(time::Duration::from_millis(1000));
    }
}
```
*/
use super::{Win32Kernel, Win32ProcessInfo, Win32VirtualTranslate};
#[cfg(feature = "regex")]
use crate::ida_regex;

use memflow::cglue::*;
use memflow::error::PartialResultExt;
use memflow::error::{Error, ErrorKind, ErrorOrigin, Result};
use memflow::mem::{MemoryView, PhysicalMemory, VirtualDma, VirtualTranslate2};
use memflow::os::keyboard::*;
use memflow::prelude::{ExportInfo, ModuleInfo, Os, Pid, Process};
use memflow::types::{umem, Address};

#[cfg(feature = "plugins")]
use memflow::cglue;

use muddy::muddy;

use log::{debug, info};
use std::convert::TryInto;

#[cfg(feature = "plugins")]
cglue_impl_group!(Win32Keyboard<T>, IntoKeyboard);

/// Interface for accessing the target's keyboard state.
#[derive(Clone, Debug)]
pub struct Win32Keyboard<T> {
    pub virt_mem: T,
    key_state_addr: Address,
}

impl<T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone>
    Win32Keyboard<VirtualDma<T, V, Win32VirtualTranslate>>
{
    pub fn with_kernel(mut kernel: Win32Kernel<T, V>) -> Result<Self> {
        let (user_process_info, key_state_addr) = Self::find_keystate(&mut kernel)?;

        let (phys_mem, vat) = kernel.virt_mem.into_inner();
        let virt_mem = VirtualDma::with_vat(
            phys_mem,
            user_process_info.base_info.proc_arch,
            user_process_info.translator(),
            vat,
        );

        Ok(Self {
            virt_mem,
            key_state_addr,
        })
    }

    /// Consumes this keyboard, returning the underlying memory and vat objects
    pub fn into_inner(self) -> (T, V) {
        self.virt_mem.into_inner()
    }
}

impl<'a, T: 'static + PhysicalMemory + Clone, V: 'static + VirtualTranslate2 + Clone>
    Win32Keyboard<VirtualDma<Fwd<&'a mut T>, Fwd<&'a mut V>, Win32VirtualTranslate>>
{
    /// Constructs a new keyboard object by borrowing a kernel object.
    ///
    /// Internally this will create a `VirtualDma` object that also
    /// borrows the PhysicalMemory and Vat objects from the kernel.
    ///
    /// The resulting process object is NOT cloneable due to the mutable borrowing.
    ///
    /// When u need a cloneable Process u have to use the `::with_kernel` function
    /// which will move the kernel object.
    pub fn with_kernel_ref(kernel: &'a mut Win32Kernel<T, V>) -> Result<Self> {
        let (user_process_info, key_state_addr) = Self::find_keystate(kernel)?;

        let (phys_mem, vat) = kernel.virt_mem.mem_vat_pair();
        let virt_mem = VirtualDma::with_vat(
            phys_mem.forward_mut(),
            user_process_info.base_info.proc_arch,
            user_process_info.translator(),
            vat.forward_mut(),
        );

        Ok(Self {
            virt_mem,
            key_state_addr,
        })
    }
}

impl<T> Win32Keyboard<T> {
    fn find_keystate<
        P: 'static + PhysicalMemory + Clone,
        V: 'static + VirtualTranslate2 + Clone,
    >(
        kernel: &mut Win32Kernel<P, V>,
    ) -> Result<(Win32ProcessInfo, Address)> {
        /*
        ref: https://www.unknowncheats.me/forum/3359384-post23.html
        Since Win11, key state bitmap has been moved into win32ksgd.sys

        Previously, Windows 10's win32kfull.sys would store the key buffer in gafAsyncKeyState
        but, since Win11, the key buffer is now stored in win32ksgd.sys under gSessionGlobalSlots.

        There is a global session slot for each session active on the machine so we need to offset
        the list with the target session. Currently, it is hardcoded to Session 1.

        Win10 key presence test:

        (*((_BYTE *)&gafAsyncKeyState + (virtual_key_code >> 2)) & (unsigned __int8)(1 << (2 * (virtual_key_code & 3))))

        Win11 key presence test:

        *(_BYTE *)((virtual_key_code >> 2) + SGDGetUserSessionState() + 0x3690) & (1 << (2 * (virtual_key_code & 3)))

        It is worth exploring win32ksgd!SGDGetUserSessionState and win32ksgd!SGDGetSessionState

        __int64 SGDGetUserSessionState()
        {
            // Dereference the session state pointer
            return *SGDGetSessionState();
        }

        void * SGDGetSessionState()
        {
            int CurrentProcessSessionId;

            CurrentProcessSessionId = GetCurrentProcessSessionId();
            if ( CurrentProcessSessionId )
                return (void *)*((void *)gSessionGlobalSlots + (unsigned int)(CurrentProcessSessionId - 1));
            else
                return gLowSessionGlobalSlots;
        }

        To replicate this via DRM, we need to find our session's gSessionGlobalSlot, dereference the pointer three times, and add the 0x3690 hardcoded offset.

        */

        let procs = kernel.process_info_list()?;

        let gaf = procs
            .iter()
            .filter(|p| {
                p.name.as_ref() == muddy!("winlogon.exe")
                    || p.name.as_ref() == muddy!("explorer.exe")
                    || p.name.as_ref() == muddy!("taskhostw.exe")
                    || p.name.as_ref() == muddy!("smartscreen.exe")
                    || p.name.as_ref() == muddy!("dwm.exe")
            })
            .find_map(|p| Self::find_in_user_process(kernel, p.pid).ok())
            .ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::ExportNotFound)
                    .log_info("unable to find any proxy process that contains gafAsyncKeyState")
            })?;

        Ok((gaf.0, gaf.1))
    }

    fn find_in_user_process<
        P: 'static + PhysicalMemory + Clone,
        V: 'static + VirtualTranslate2 + Clone,
    >(
        kernel: &mut Win32Kernel<P, V>,
        pid: Pid,
    ) -> Result<(Win32ProcessInfo, Address)> {
        let user_process_info = kernel.process_info_by_pid(pid)?;
        let user_process_info_win32 =
            kernel.process_info_from_base_info(user_process_info.clone())?;

        let win32kbase_mod_name = muddy!("win32kbase.sys");
        let win32kbase_module_info: ModuleInfo = kernel.module_by_name(win32kbase_mod_name)?;
        debug!("found {win32kbase_mod_name}: {:?}", win32kbase_module_info);

        // Win32k temporary session global driver was first introduced in 22H2 (10.0.22621.1) (2022-09-20)
        // so we cannot be sure it will be active on all Win11 devices
        let winver = kernel.kernel_info.kernel_winver;
        let is_win11 = winver >= (10, 0, 22621).into();
        // let at_least_23_h2 = winver >= (10, 0, 22621).into();
        let at_least_24_h2 = winver >= (10, 0, 22632).into();
        info!(
            "Loading keyboard for {} build {}",
            if is_win11 { "Win11" } else { "Win10" },
            winver
        );

        if is_win11 {
            // Load for Windows 11
            // On Windows 11 there are special cases to handle on the border of 23h2 and 24h2

            // offset from module base of gSessionGlobalSlots in win32k.sys (24h2) or win32ksgd.sys (on 23h2)
            // todo add a signature scan for this and use this behaviour as a fallback
            let (
                g_session_global_slots_signature,
                target_kernel_module_name,
                g_session_global_slots_offset_fallback,
                key_state_offset_fallback,
            ) = if at_least_24_h2 {
                // win32ksgd actually may not even exist anymore here
                // now it is stored in win32k.sys
                /*
                int64_t W32GetSessionStateForSession(int32_t arg1)
                ... omitted
                +0x15508  488b05e1cf0600     mov     rax, qword [rel gSessionGlobalSlots]
                +0x1550f  ffc9               dec     ecx // THIS LINE HERE BREAKS THE SIGNATURE SINCE LAST VERSION
                +0x15511  488b04c8           mov     rax, qword [rax+rcx*8]
                +0x15515  c3                 retn     {__return_addr}
                 */

                // 48 8B 05 ? ? ? ? FF C9 + 3 -> rel32 deref
                // or 48 8B 05 ? ? ? ? FF C9 48 8B 04 C8 + 3 -> rel32 deref

                let sig: &'static str;
                #[cfg(feature = "regex")]
                {
                    sig = ida_regex![48 8B 05 ? ? ? ? FF C9];
                }
                #[cfg(not(feature = "regex"))]
                {
                    return Err(
                        Error(ErrorOrigin::OsLayer, ErrorKind::UnsupportedOptionalFeature)
                            .log_error(
                                "cannot find keyboard signature because regex feature is disabled",
                            ),
                    ); // todo: repalce with pelite sig
                }

                (sig, muddy!("win32k.sys"), 0x824F0, 0x3808) // 24H2  win32k.sys + 0x824F0
            } else {
                /*
                +0x1260    int64_t SGDGetSessionState()
                ... omitted
                +0x127d  8d48ff             lea     ecx, [rax-0x1]
                +0x1280  488b05891e0000     mov     rax, qword [rel gSessionGlobalSlots]
                +0x1287  488b04c8           mov     rax, qword [rax+rcx*8]

                +0x128b  4883c428           add     rsp, 0x28
                +0x128f  c3                 retn     {__return_addr}

                 */
                // signature for 23h2
                // 48 8B 05 ? ? ? ? 48 8B 04 C8 + 3 -> rel32 deref

                let sig;
                #[cfg(feature = "regex")]
                {
                    sig = ida_regex![48 8B 05 ? ? ? ? 48 8B 04 C8];
                }
                #[cfg(not(feature = "regex"))]
                {
                    return Err(
                        Error(ErrorOrigin::OsLayer, ErrorKind::UnsupportedOptionalFeature)
                            .log_error(
                                "cannot find keyboard signature because regex feature is disabled",
                            ),
                    ); // todo: repalce with pelite sig
                }

                (sig, muddy!("WIN32KSGD.SYS"), 0x3110, 0x36a8) // 0x3690 or 0x36a8 // 23h2 and below win32ksgd.sys + 0x3110
            };

            // find either win32k.sys or win32ksgd.sys
            let win32ksgd_module_info = match kernel.module_by_name(target_kernel_module_name) {
                Ok(m) => m,
                Err(_) => {
                    return Err(
                        Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound).log_info(
                            ["unable to find kernel module", target_kernel_module_name].join(" "),
                        ),
                    )
                }
            };

            debug!("Found kernel module: {:?}", win32ksgd_module_info);

            // find the key states offset:
            // win32kbase.sys
            // b9 00 80 ff ff ? 22 B4 ? ? ? ? ? 41 -> + 9 -> rip rel32

            /* 24H2
            +0x1a8da5  d3e6               shl     esi, cl
            +0x1a8da7  b90080ffff         mov     ecx, 0xffff8000
            🔖+0x1a8dac  4022b41008380000   and     sil, byte [rax+rdx+0x3808]
            +0x1a8db4  410fb7c7           movzx   eax, r15w
            +0x1a8db8  660bc1             or      ax, cx
            +0x1a8dbb  4084f6             test    sil, sil
            +0x1a8dbe  66410f44c7         cmove   ax, r15w
             */

            /* 23H2
            +0x0971e5  d3e6               shl     esi, cl
            +0x0971e7  b90080ffff         mov     ecx, 0xffff8000
            🔖+0x0971ec  4122b406a8360000   and     sil, byte [r14+rax+0x36a8]
            +0x0971f4  410fb7c7           movzx   eax, r15w
            +0x0971f8  660bc1             or      ax, cx
            +0x0971fb  4084f6             test    sil, sil
            +0x0971fe  66410f44c7         cmove   ax, r15w
            */

            let mut user_process = kernel.process_by_info(user_process_info)?;

            let key_state_offset: umem;
            #[cfg(feature = "regex")]
            {
                let module_buf = user_process
                    .virt_mem
                    .read_raw(
                        win32kbase_module_info.base,
                        win32kbase_module_info.size.try_into().unwrap(),
                    )
                    .data_part()?;
                key_state_offset = Self::scan_module_sig_val32(
                    module_buf,
                    ida_regex![b9 00 80 ff ff ? 22 B4 ? ? ? ? ? 41],
                    9,
                )
                .unwrap_or(key_state_offset_fallback);
            }

            #[cfg(not(feature = "regex"))]
            {
                key_state_offset = key_state_offset_fallback
            };

            let g_session_global_slots_address: Address; // =
            #[cfg(feature = "regex")]
            {
                g_session_global_slots_address = win32ksgd_module_info.base
                    + Self::find_global_slots_sig(
                        &mut user_process.virt_mem,
                        &win32ksgd_module_info,
                        g_session_global_slots_signature,
                    )
                    .map(|offset| {
                        // santity check
                        if offset == 0 {
                            g_session_global_slots_offset_fallback
                        } else {
                            offset
                        }
                    })
                    .unwrap_or(g_session_global_slots_offset_fallback);
            };
            #[cfg(not(feature = "regex"))]
            {
                g_session_global_slots_address =
                    (win32ksgd_module_info.base + g_session_global_slots_offset_fallback);
            };

            debug!(
                "{} address: {:?} : {:?}",
                muddy!("gSessionGlobalSlots"),
                win32ksgd_module_info.base + g_session_global_slots_offset_fallback,
                g_session_global_slots_address
            );

            let g_session_global_slot_first_deref = user_process.virt_mem.read_addr_arch(
                win32ksgd_module_info.arch.into(),
                g_session_global_slots_address,
            )?;
            debug!(
                "gSessionGlobalSlot 1st deref: {:?}",
                g_session_global_slot_first_deref
            );

            let g_session_global_slot_second_deref = user_process.virt_mem.read_addr_arch(
                win32ksgd_module_info.arch.into(),
                g_session_global_slot_first_deref,
            )?;
            debug!(
                "{} 2nd deref: {:?}",
                muddy!("gSessionGlobalSlots"),
                g_session_global_slot_second_deref
            );

            let g_session_global_slot_third_deref = user_process.virt_mem.read_addr_arch(
                win32ksgd_module_info.arch.into(),
                g_session_global_slot_second_deref,
            )?;
            debug!(
                "{} 3rd deref: {:?}",
                muddy!("gSessionGlobalSlots"),
                g_session_global_slot_third_deref
            );

            debug!(
                "Key State Buffer Address: {:?}",
                g_session_global_slot_third_deref + key_state_offset // 0x3690  or 0x36a8 or 0x3808 (key_state_offset_fallback)
            );

            Ok((
                user_process_info_win32,
                g_session_global_slot_third_deref + key_state_offset, // todo: signature scan for the key state offset
            ))
        } else {
            // Load for Windows 10
            let mut user_process = kernel.process_by_info(user_process_info)?;
            debug!(
                "trying to find gaf signature in user proxy process `{}`",
                user_process.info().name.as_ref()
            );

            // TODO: lazy
            let export_addr =
                Self::find_gaf_pe(&mut user_process.virt_mem, &win32kbase_module_info).or_else(
                    |_| Self::find_gaf_sig(&mut user_process.virt_mem, &win32kbase_module_info),
                )?;
            debug!(
                "found gaf signature in user proxy process `{}` at {:x}",
                user_process.info().name.as_ref(),
                export_addr
            );

            Ok((
                user_process_info_win32,
                win32kbase_module_info.base + export_addr,
            ))
        }
    }

    /// returns the 32bit value in a the function assembly (instead of reading it as a RIP Relative (rel32) Address)
    #[cfg(feature = "regex")]
    fn scan_module_sig_val32(
        module_buf: Vec<u8>,
        sig: &str,
        offset_to_val32: usize,
    ) -> Result<umem> {
        use ::regex::bytes::*;
        let re = Regex::new(sig).map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Encoding)
                .log_info(muddy!("malformed rip signature"))
        })?;
        let buf_offs = re
            .find(module_buf.as_slice())
            .ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::NotFound)
                    .log_info(muddy!("unable to find rip signature"))
            })?
            .start()
            + offset_to_val32;

        // Return the Little Endian 32bit value directly
        Ok(u32::from_le_bytes(module_buf[buf_offs..buf_offs + 4].try_into().unwrap()) as umem)
    }

    fn find_gaf_pe(
        virt_mem: &mut impl MemoryView,
        win32kbase_module_info: &ModuleInfo,
    ) -> Result<umem> {
        let mut offset = None;
        let gaf_name = muddy!("gafAsyncKeyState");
        let callback = &mut |export: ExportInfo| {
            if export.name.as_ref() == gaf_name {
                offset = Some(export.offset);
                false
            } else {
                true
            }
        };
        memflow::os::util::module_export_list_callback(
            virt_mem,
            win32kbase_module_info,
            callback.into(),
        )?;
        offset.ok_or_else(|| {
            Error(ErrorOrigin::OsLayer, ErrorKind::ExportNotFound)
                .log_info(muddy!("unable to find gafAsyncKeyState"))
        })
    }

    // TODO: replace with a custom signature scanning crate
    #[cfg(feature = "regex")]
    fn find_gaf_sig(
        virt_mem: &mut impl MemoryView,
        win32kbase_module_info: &ModuleInfo,
    ) -> Result<umem> {
        use ::regex::bytes::*;

        let module_buf = virt_mem
            .read_raw(
                win32kbase_module_info.base,
                win32kbase_module_info.size.try_into().unwrap(),
            )
            .data_part()?;

        // 48 8B 05 ? ? ? ? 48 89 81 ? ? 00 00 48 8B 8F + 0x3
        let re =
            Regex::new(ida_regex![48 8B 05 ? ? ? ? 48 89 81 ? ? 00 00 48 8B 8F]).map_err(|_| {
                Error(ErrorOrigin::OsLayer, ErrorKind::Encoding)
                    .log_info(muddy!("malformed gafAsyncKeyState signature"))
            })?;
        let buf_offs = re
            .find(module_buf.as_slice())
            .ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::NotFound)
                    .log_info(muddy!("unable to find gafAsyncKeyState signature"))
            })?
            .start()
            + 0x3;

        // compute rip relative addr
        let export_offs = buf_offs as u32
            + u32::from_le_bytes(module_buf[buf_offs..buf_offs + 4].try_into().unwrap())
            + 0x4;
        debug!(
            "{} export found at: {:x}",
            muddy!("gafAsyncKeyState"),
            export_offs
        );
        Ok(export_offs as umem)
    }

    /// This is for windows 11 support
    #[cfg(feature = "regex")]
    fn find_global_slots_sig(
        virt_mem: &mut impl MemoryView,
        win32k_module_info: &ModuleInfo,
        signature: &str,
    ) -> Result<umem> {
        use ::regex::bytes::*;

        let module_buf = virt_mem
            .read_raw(
                win32k_module_info.base,
                win32k_module_info.size.try_into().unwrap(),
            )
            .data_part()?;

        let re = Regex::new(signature).map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Encoding)
                .log_info(muddy!("malformed gSessionGlobalSlots signature"))
        })?;
        let buf_offs = re
            .find(module_buf.as_slice())
            .ok_or_else(|| {
                Error(ErrorOrigin::OsLayer, ErrorKind::NotFound)
                    .log_info(muddy!("unable to find gSessionGlobalSlots signature"))
            })?
            .start()
            + 0x3;

        // compute rip relative addr
        let export_offs = buf_offs as u32
            + u32::from_le_bytes(module_buf[buf_offs..buf_offs + 4].try_into().unwrap())
            + 0x4;
        debug!(
            "{} export found at: {:x}",
            muddy!("gSessionGlobalSlots"),
            export_offs
        );
        Ok(export_offs as umem)
    }

    // feature disabled stubs:
    #[cfg(not(feature = "regex"))]
    fn find_gaf_sig(
        virt_mem: &mut impl MemoryView,
        win32kbase_module_info: &ModuleInfo,
    ) -> Result<umem> {
        Err(
            Error(ErrorOrigin::OsLayer, ErrorKind::UnsupportedOptionalFeature)
                .log_error("cannot find keyboard signature because regex feature is disabled"),
        )
    }

    #[cfg(not(feature = "regex"))]
    fn find_global_slots_sig(
        virt_mem: &mut impl MemoryView,
        win32kbase_module_info: &ModuleInfo,
    ) -> Result<umem> {
        Err(
            Error(ErrorOrigin::OsLayer, ErrorKind::UnsupportedOptionalFeature)
                .log_error("cannot find keyboard signature because regex feature is disabled"),
        )
    }
}

macro_rules! get_ks_byte {
    ($vk:expr) => {
        $vk * 2 / 8
    };
}

macro_rules! get_ks_down_bit {
    ($vk:expr) => {
        1 << (($vk % 4) * 2)
    };
}

macro_rules! is_key_down {
    ($ks:expr, $vk:expr) => {
        ($ks[get_ks_byte!($vk) as usize] & get_ks_down_bit!($vk)) != 0
    };
}

macro_rules! set_key_down {
    ($ks:expr, $vk:expr, $down:expr) => {
        if $down {
            ($ks[get_ks_byte!($vk) as usize] |= get_ks_down_bit!($vk))
        } else {
            ($ks[get_ks_byte!($vk) as usize] &= !get_ks_down_bit!($vk))
        }
    };
}

impl<T: MemoryView> Keyboard for Win32Keyboard<T> {
    type KeyboardStateType = Win32KeyboardState;

    /// Reads the gafAsyncKeyState global from the win32kbase.sys kernel module and
    /// returns true wether the given key was pressed.
    /// This function accepts a valid microsoft virtual keycode.
    /// In case of supplying a invalid key this function will just return false cleanly.
    ///
    /// A list of all Keycodes can be found on the [msdn](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
    fn is_down(&mut self, vk: i32) -> bool {
        if !(0..=256).contains(&vk) {
            false
        } else if let Ok(buffer) = self
            .virt_mem
            .read::<[u8; 256 * 2 / 8]>(self.key_state_addr)
            .data_part()
        {
            is_key_down!(buffer, vk)
        } else {
            false
        }
    }

    /// Writes the gafAsyncKeyState global to the win32kbase.sys kernel module.
    ///
    /// # Remarks:
    ///
    /// This will not enforce key presses in all applications on Windows.
    /// It will only modify calls to GetKeyState / GetAsyncKeyState.
    fn set_down(&mut self, vk: i32, down: bool) {
        if (0..=256).contains(&vk) {
            if let Ok(mut buffer) = self.virt_mem.read::<[u8; 256 * 2 / 8]>(self.key_state_addr) {
                set_key_down!(buffer, vk, down);
                self.virt_mem.write(self.key_state_addr, &buffer).ok();
            }
        }
    }

    /// Reads the gafAsyncKeyState global from the win32kbase.sys kernel module.
    fn state(&mut self) -> memflow::error::Result<Self::KeyboardStateType> {
        let buffer: [u8; 256 * 2 / 8] = self.virt_mem.read(self.key_state_addr)?;
        Ok(Win32KeyboardState { buffer })
    }
}

/// Represents the current Keyboardstate.
///
/// Internally this will hold a 256 * 2 / 8 byte long copy of the gafAsyncKeyState array from the target.
#[derive(Clone)]
pub struct Win32KeyboardState {
    buffer: [u8; 256 * 2 / 8],
}

impl KeyboardState for Win32KeyboardState {
    /// Returns true wether the given key was pressed.
    /// This function accepts a valid microsoft virtual keycode.
    /// In case of supplying a invalid key this function will just return false cleanly.
    ///
    /// A list of all Keycodes can be found on the [msdn](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
    fn is_down(&self, vk: i32) -> bool {
        if !(0..=256).contains(&vk) {
            false
        } else {
            is_key_down!(self.buffer, vk)
        }
    }
}
