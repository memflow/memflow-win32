use std::cmp::{Ord, Ordering, PartialEq};
use std::fmt;

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32Guid {
    pub file_name: String,
    pub guid: String,
}

impl Win32Guid {
    pub fn new(file_name: &str, guid: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            guid: guid.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[repr(C)]
pub struct Win32Version {
    nt_major_version: u32,
    nt_minor_version: u32,
    nt_build_number: u32,
}

impl Win32Version {
    pub fn new(nt_major_version: u32, nt_minor_version: u32, nt_build_number: u32) -> Self {
        Self {
            nt_major_version,
            nt_minor_version,
            nt_build_number,
        }
    }

    #[inline]
    pub const fn mask_build_number(mut self) -> Self {
        self.nt_build_number &= 0xFFFF;
        self
    }

    #[inline]
    pub const fn major_version(&self) -> u32 {
        self.nt_major_version
    }

    #[inline]
    pub const fn minor_version(&self) -> u32 {
        self.nt_minor_version
    }

    #[inline]
    pub const fn build_number(&self) -> u32 {
        self.nt_build_number & 0xFFFF
    }

    #[inline]
    pub const fn is_checked_build(&self) -> bool {
        (self.nt_build_number & 0xF0000000) == 0xC0000000
    }

    #[inline]
    pub const fn as_tuple(&self) -> (u32, u32, u32) {
        (
            self.major_version(),
            self.minor_version(),
            self.build_number(),
        )
    }
}

impl PartialOrd for Win32Version {
    fn partial_cmp(&self, other: &Win32Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Win32Version {
    fn cmp(&self, other: &Win32Version) -> Ordering {
        if self.build_number() != 0 && other.build_number() != 0 {
            return self.build_number().cmp(&other.build_number());
        }

        if self.nt_major_version != other.nt_major_version {
            self.nt_major_version.cmp(&other.nt_major_version)
        } else if self.nt_minor_version != other.nt_minor_version {
            self.nt_minor_version.cmp(&other.nt_minor_version)
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for Win32Version {
    fn eq(&self, other: &Win32Version) -> bool {
        if self.nt_build_number != 0 && other.nt_build_number != 0 {
            self.nt_build_number.eq(&other.nt_build_number)
        } else {
            self.nt_major_version == other.nt_major_version
                && self.nt_minor_version == other.nt_minor_version
        }
    }
}

impl Eq for Win32Version {}

impl From<(u32, u32)> for Win32Version {
    fn from((nt_major_version, nt_minor_version): (u32, u32)) -> Win32Version {
        Win32Version {
            nt_major_version,
            nt_minor_version,
            nt_build_number: 0,
        }
    }
}

impl From<(u32, u32, u32)> for Win32Version {
    fn from(
        (nt_major_version, nt_minor_version, nt_build_number): (u32, u32, u32),
    ) -> Win32Version {
        Win32Version {
            nt_major_version,
            nt_minor_version,
            nt_build_number,
        }
    }
}

impl fmt::Display for Win32Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nt_major_version != 0 {
            write!(
                f,
                "{}.{}.{}",
                self.major_version(),
                self.minor_version(),
                self.build_number()
            )
        } else {
            write!(f, "{}", self.build_number())
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use super::Win32Version;

    #[test]
    fn win32_version_cmp() {
        let a = Win32Version::new(10, 0, 22621); // windows 11
        let b = Win32Version::new(10, 0, 4026550885); // windows 10
        assert_eq!(a.cmp(&b), Ordering::Greater);
    }
}
