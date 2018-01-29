// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};

use super::failure::Error;
use super::sentry::io::{IOCTL_SENTRY_TYPE, SE_NT_DEVICE_NAME};
use super::iochannel::{Device, IoCtl};
use std::ptr;

use std::sync::mpsc::Sender;
use super::cli::output::{MessageType, ShellMessage};
use super::console::style;

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("errors").subcommand(SubCommand::with_name("ioctl"))
}

pub fn tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("ioctl", Some(_)) => test_ioctl_incorrect_function(messenger),
        _ => Ok(println!("{}", matches.usage())),
    }
}

pub fn test_ioctl_incorrect_function(messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    // debug!(logger, "creating an invalid i/o call");
    ShellMessage::send(
        messenger,
        format!("{}",style("Creating an invalid i/o call...").yellow()),
        MessageType::Spinner,
        0,
    );

    let device = Device::new(SE_NT_DEVICE_NAME).unwrap();
    let no_name_control = IoCtl::new(None, IOCTL_SENTRY_TYPE, 0x0777, None, None);
    let named_control = IoCtl::new(
        Some("IOCTL_NAME_EXAMPLE"),
        IOCTL_SENTRY_TYPE,
        0x0777,
        None,
        None,
    );

    if let Err(err) = device.raw_call(no_name_control, ptr::null_mut(), 0) {
        ShellMessage::send(
            messenger,
            format!("Unnamed I/O control: {}", style(err.to_string()).underlined().red()),
            MessageType::Close,
            0,
        );
        // debug!(logger, "Unnamed I/O control: {}", err.to_string());
    }

    if let Err(err) = device.raw_call(named_control, ptr::null_mut(), 0) {
        ShellMessage::send(
            messenger,
            format!("Named I/O control: {}", style(err.to_string()).underlined().yellow()),
            MessageType::Close,
            0,
        );
        // debug!(logger, "Named I/O control: {}", err.to_string());
    }

    Ok(())
}
