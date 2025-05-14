/*!
This crate contains memflow's win32 implementation.
It is used to interface with windows targets.
*/

#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

pub mod kernel;

pub mod offsets;

pub mod win32;

#[cfg(feature = "regex")]
pub mod ida_signatures;

pub mod prelude {
    pub mod v1 {
        pub use crate::kernel::*;
        pub use crate::offsets::*;
        pub use crate::win32::*;
    }
    pub use v1::*;
}

#[cfg(feature = "plugins")]
pub mod plugins;
