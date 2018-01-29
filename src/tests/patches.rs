// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::cli::colorize;

use std::{thread};
use std::time::Duration;

use super::iochannel::Device;
use super::failure::Error;
use super::sentry::{memory, misc, io};
use super::sentry::memguard::{Patch, Partition, Guard};

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("patches")
                .subcommand(SubCommand::with_name("patch-vuln"))
}

pub fn parse(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("patch-vuln",      Some(matches))  => test_patch_driver(matches, messenger),
        _                                   => Ok(println!("{}", matches.usage()))
    }
}

fn test_patch_driver(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    const PAGE_SIZE: usize = 0x1000;
    const PATCH_PAGE: u64 = 0x5000;
    const PATCH_OFFSET: u64 = 0xBEC;

    if let Some(driver) = misc::Drivers::iter().find(|driver| driver.name.contains("HEVD")) {
        let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

        let new_code = memory::alloc_virtual_memory(&device, PAGE_SIZE).unwrap();
        {
            let partition = Partition::root();

            let patch_base = driver.base + PATCH_PAGE;
            let patch = vec![0x90; 6];


            let _ = memory::copy_virtual_memory(&device, patch_base, new_code, PAGE_SIZE);
            let _ = memory::write_virtual_memory(&device, new_code + PATCH_OFFSET, patch);

            let patch = Patch::new(&partition, patch_base, new_code, PAGE_SIZE as u64).unwrap();
            ShellMessage::send(messenger,format!("{}",patch),MessageType::Spinner,0);

            let mut guard = Guard::new(&partition, None);

            ShellMessage::send(messenger,format!("adding {} to {}", patch, guard),MessageType::Spinner,0);
            guard.add(patch);

            ShellMessage::send(messenger,"Starting guard, and sleeping 30 secs".to_string(),MessageType::Spinner,0);
            guard.start();
            colorize::success("HEVD patch applied.");
            thread::sleep(Duration::from_secs(30));
            guard.stop();
            colorize::info("HEVD patch revoked.");
            ShellMessage::send(messenger,"Done!".to_string(),MessageType::Close,0);

        }

        let _ = memory::free_virtual_memory(&device, new_code);

        Ok(())
    }
    else {
        colorize::failed("HEVD driver not found.");
        Err(format_err!("HEVD driver not found."))
    }
}
