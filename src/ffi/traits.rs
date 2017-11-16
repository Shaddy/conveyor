// Copyright © ByteHeed.  All rights reserved.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

pub trait EncodeUtf16 {
    fn encode_utf16(&self) -> Vec<u16>;
    fn encode_utf16_null(&self) -> Vec<u16>;
}

impl<T> EncodeUtf16 for T where T: AsRef<OsStr> {
    fn encode_utf16(&self) -> Vec<u16> {
        self.as_ref().encode_wide().collect()
    }
    fn encode_utf16_null(&self) -> Vec<u16> {
        self.as_ref().encode_wide().chain(Some(0)).collect()
    }
}