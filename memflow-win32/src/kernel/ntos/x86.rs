use std::prelude::v1::*;

use super::pehelper;
use crate::kernel::StartBlock;

use memflow::dataview::PodMethods;
use memflow::error::{Error, ErrorKind, ErrorOrigin, PartialResultExt, Result};
use memflow::mem::MemoryView;
use memflow::types::{size, umem, Address};

use log::{debug, info};

use muddy::muddy;

use pelite::image::IMAGE_DOS_HEADER;

const SIZE_256MB: usize = size::mb(256);
const SIZE_8MB: usize = size::mb(8);
const SIZE_4KB: usize = size::kb(4);

// https://github.com/ufrisk/MemProcFS/blob/f2d15cf4fe4f19cfeea3dad52971fae2e491064b/vmm/vmmwininit.c#L410
pub fn find<T: MemoryView>(virt_mem: &mut T, _start_block: &StartBlock) -> Result<(Address, umem)> {
    debug!("x86::find: trying to find ntoskrnl.exe");

    for base_addr in (0..SIZE_256MB).step_by(SIZE_8MB) {
        let base_addr = size::gb(2) + base_addr;
        // search in each page in the first 8mb chunks in the first 64mb of virtual memory
        let mut buf = vec![0; SIZE_8MB];
        virt_mem
            .read_raw_into(base_addr.into(), &mut buf)
            .data_part()?;

        for addr in (0..SIZE_8MB).step_by(SIZE_4KB) {
            // TODO: potential endian mismatch in pod
            let view = PodMethods::as_data_view(&buf[addr..]);

            // check for dos header signature (MZ) // TODO: create global
            if view.read::<IMAGE_DOS_HEADER>(0).e_magic != 0x5a4d {
                continue;
            }

            if view.read::<IMAGE_DOS_HEADER>(0).e_lfanew > 0x800 {
                continue;
            }

            let image_base = Address::from(base_addr + addr);
            if let Ok(name) = pehelper::try_get_pe_name(virt_mem, image_base) {
                if name == muddy!("ntoskrnl.exe") {
                    info!("{}", muddy!("ntoskrnl found"));
                    // TODO: unify pe name + size
                    if let Ok(size_of_image) = pehelper::try_get_pe_size(virt_mem, image_base) {
                        return Ok((image_base, size_of_image));
                    }
                }
            }
        }
    }

    Err(
        Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound).log_trace(muddy!(
            "find_x86(): unable to locate ntoskrnl.exe in high mem"
        )),
    )
}
