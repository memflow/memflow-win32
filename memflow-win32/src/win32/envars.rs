use std::prelude::v1::*;

use crate::offsets::Win32ArchOffsets;

use log::trace;

use memflow::architecture::ArchitectureIdent;
use memflow::error::Result;
use memflow::mem::MemoryView;
use memflow::os::util::env_block_list_utf16_callback;
use memflow::os::{EnvVarCallback, EnvVarInfo};
use memflow::types::{umem, Address};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32EnvListInfo {
    env_block: Address,
    env_size: umem,
    offsets: Win32ArchOffsets,
    proc_params: Address,
}

impl Win32EnvListInfo {
    /// Construct from a PEB: reads _PEB::ProcessParameters and
    /// _RTL_USER_PROCESS_PARAMETERS::Environment (PWSTR) and
    /// _RTL_USER_PROCESS_PARAMETERS::EnvironmentSize.

    pub fn with_peb(
        mem: &mut impl MemoryView,
        peb: Address,
        arch: ArchitectureIdent,
    ) -> Result<Self> {
        let offsets = Win32ArchOffsets::from(arch);
        let arch_obj = arch.into();

        trace!("peb_process_params_offs={:x}", offsets.peb_process_params);
        let proc_params = mem.read_addr_arch(arch_obj, peb + offsets.peb_process_params)?;
        trace!("ProcessParameters={:x}", proc_params);

        trace!("ppm_environment_offs={:x}", offsets.ppm_environment);
        let env_block = mem.read_addr_arch(arch_obj, proc_params + offsets.ppm_environment)?;
        trace!("Environment={:x}", env_block);

        trace!(
            "ppm_environment_size_offs={:x}",
            offsets.ppm_environment_size
        );
        let size_addr = proc_params + offsets.ppm_environment_size;

        let env_size: umem = mem.read(size_addr)?;

        trace!("EnvironmentSize={:x}", env_size);
        Ok(Self {
            env_block,
            env_size,
            offsets,
            proc_params,
        })
    }

    /// Construct directly from a known Environment pointer and size.
    pub fn with_base(env_block: Address, env_size: umem, arch: ArchitectureIdent) -> Result<Self> {
        let offsets = Win32ArchOffsets::from(arch);
        trace!(
            "env_block={:x} offsets={:?} size={:?}",
            env_block,
            offsets,
            env_size
        );
        Ok(Self {
            env_block,
            env_size,
            offsets,
            proc_params: Address::NULL,
        })
    }

    #[inline]
    pub fn env_block(&self) -> Address {
        self.env_block
    }

    /// Collect all environment variables into a Vec.
    pub fn envar_list<V: MemoryView>(
        &self,
        mem: &mut impl AsMut<V>,
        arch: ArchitectureIdent,
    ) -> Result<Vec<EnvVarInfo>> {
        let mut out = vec![];
        self.envar_list_callback(mem, arch, (&mut out).into())?;
        Ok(out)
    }

    /// Enumerate environment variables via callback (UTF-16LE multi-string).
    ///
    /// # Arguments
    /// * `mem` - memory view in process context
    /// * `arch` - view architecture (native or WOW64)
    /// * `callback` - receives each EnvVarInfo; return `true` to continue, `false` to stop.
    pub fn envar_list_callback<M: AsMut<V>, V: MemoryView>(
        &self,
        mem: &mut M,
        arch: ArchitectureIdent,
        callback: EnvVarCallback,
    ) -> Result<()> {
        env_block_list_utf16_callback(mem.as_mut(), self.env_block, self.env_size, arch, callback)
    }
}
