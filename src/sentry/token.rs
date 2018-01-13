// Copyright © ByteHeed.  All rights reserved.

use super::io::IOCTL_SENTRY_TYPE;
use super::iochannel::{Device, IoCtl};
use super::structs::{RawStruct, SE_STEAL_TOKEN};

pub use super::structs::TokenType;                

pub fn steal_token(device: &Device, source: u64, target: u64, kind: TokenType) {
    let control = IoCtl::new(Some("SE_STEAL_TOKEN"), IOCTL_SENTRY_TYPE, 0x0A60, None, None);

    let mut token = SE_STEAL_TOKEN::init();

    token.SourcePid = source;
    token.TargetPid = target;
    token.StealType = kind as u32;

    let (ptr, len) = (token.as_ptr(), token.size());

    device.raw_call(control, ptr, len)
          .expect("Error calling IOCTL_SENTRY_STEAL_TOKEN");

}