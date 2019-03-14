// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::num::NonZeroU32;
use core::convert::From;
use core::fmt;
#[cfg(not(target_env = "sgx"))]
use std::{io, error};

// A randomly-chosen 16-bit prefix for our codes
pub(crate) const CODE_PREFIX: u32 = 0x57f40000;
const CODE_UNKNOWN: u32 = CODE_PREFIX | 0;
const CODE_UNAVAILABLE: u32 = CODE_PREFIX | 1;

/// An unknown error.
/// 
/// This is the following constant: 57F40000 (hex) / 1475608576 (decimal).
pub const ERROR_UNKNOWN: Error = Error(unsafe {
    NonZeroU32::new_unchecked(CODE_UNKNOWN)
});

/// No generator is available.
/// 
/// This is the following constant: 57F40001 (hex) / 1475608577 (decimal).
pub const ERROR_UNAVAILABLE: Error = Error(unsafe {
    NonZeroU32::new_unchecked(CODE_UNAVAILABLE)
});

/// The error type.
/// 
/// This type is small and no-std compatible.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(NonZeroU32);

impl Error {
    /// Extract the error code.
    /// 
    /// This may equal one of the codes defined in this library or may be a
    /// system error code.
    /// 
    /// One may attempt to format this error via the `Display` implementation.
    pub fn code(&self) -> NonZeroU32 {
        self.0
    }
    
    fn msg(&self) -> Option<&'static str> {
        if let Some(msg) = super::error_msg_inner(self.0) {
            Some(msg)
        } else {
            match *self {
                ERROR_UNKNOWN => Some("getrandom: unknown error"),
                ERROR_UNAVAILABLE => Some("getrandom: unavailable"),
                _ => None
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.msg() {
            Some(msg) => write!(f, "Error(\"{}\")", msg),
            None => write!(f, "Error(0x{:08X})", self.0),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self.msg() {
            Some(msg) => write!(f, "{}", msg),
            None => write!(f, "getrandom: unknown code 0x{:08X}", self.0),
        }
    }
}

impl From<NonZeroU32> for Error {
    fn from(code: NonZeroU32) -> Self {
        Error(code)
    }
}

#[cfg(not(target_env = "sgx"))]
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        err.raw_os_error()
            .and_then(|code| NonZeroU32::new(code as u32))
            .map(|code| Error(code))
            // in practice this should never happen
            .unwrap_or(ERROR_UNKNOWN)
    }
}

#[cfg(not(target_env = "sgx"))]
impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        match err.msg() {
            Some(msg) => io::Error::new(io::ErrorKind::Other, msg),
            None => io::Error::from_raw_os_error(err.0.get() as i32),
        }
    }
}

#[cfg(not(target_env = "sgx"))]
impl error::Error for Error { }

#[cfg(test)]
mod tests {
    use std::mem::size_of;
    use super::Error;
    
    #[test]
    fn test_size() {
        assert_eq!(size_of::<Error>(), 4);
        assert_eq!(size_of::<Result<(), Error>>(), 4);
    }
}