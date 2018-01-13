// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

use super::failure::Error;
use super::sentry::io::{SE_NT_DEVICE_NAME, IOCTL_SENTRY_TYPE};
use super::iochannel::{Device, IoCtl};
use std::ptr;

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("errors")
            .subcommand(SubCommand::with_name("ioctl"))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("ioctl", Some(_))  => test_ioctl_incorrect_function(logger),
        _                   => Ok(println!("{}", matches.usage()))
    }
}

pub fn test_ioctl_incorrect_function(logger: &Logger) -> Result<(), Error> {
    debug!(logger, "creating an invalid i/o call");

    let device = Device::new(SE_NT_DEVICE_NAME).unwrap();
    let no_name_control = IoCtl::new(None, IOCTL_SENTRY_TYPE, 0x0777, None, None);
    let named_control = IoCtl::new(Some("IOCTL_NAME_EXAMPLE"), IOCTL_SENTRY_TYPE, 0x0777, None, None);

    if let Err(err) = device.raw_call(no_name_control, ptr::null_mut(), 0) {
        debug!(logger, "Unnamed I/O control: {}", err.to_string());
    }

    if let Err(err) = device.raw_call(named_control, ptr::null_mut(), 0) {
        debug!(logger, "Named I/O control: {}", err.to_string());
    }

    Ok(())
}