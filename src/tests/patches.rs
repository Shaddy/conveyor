// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::console::style;

use super::service;
use super::iochannel::Device;
use super::failure::Error;
use super::sentry::{memory, misc, io};
use super::sentry::memguard::{Patch, Partition, Guard};

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("patch")
                .subcommand(SubCommand::with_name("vulnerability"))
}

pub fn parse(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("vulnerability",      Some(matches))  => test_patch_driver(matches, messenger),
        _                                   => Ok(println!("{}", matches.usage()))
    }
}

fn test_patch_driver(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    const PAGE_SIZE: usize = 0x1000;
    const PATCH_PAGE: u64 = 0x5000;
    const PATCH_OFFSET: u64 = 0xBEC;

    ShellMessage::send(messenger, "scanning drivers:".to_string(), MessageType::Spinner, 0);

    if let Some(driver) = misc::Drivers::iter()
                                    .inspect(|driver| {
                                        ShellMessage::send(messenger,
                                             format!("scanning drivers: {}", driver.name),
                                            MessageType::Spinner, 0);
                                    })
                                    .find(|driver| driver.name.to_lowercase().contains("hevd")) {

        ShellMessage::send(messenger, format!("scanning drivers: {} [FOUND]",
                                                style("HEVD.sys").green()),
                                    MessageType::Close, 0);

        let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

        let new_code = memory::alloc_virtual_memory(&device, PAGE_SIZE).unwrap();
        {
            let partition = Partition::root();

            let patch_base = driver.base + PATCH_PAGE;
            let patch = vec![0x90; 6];


            let _ = memory::copy_virtual_memory(&device, patch_base, new_code, PAGE_SIZE);
            let _ = memory::write_virtual_memory(&device, new_code + PATCH_OFFSET, patch);

            let patch = Patch::new(&partition, patch_base, new_code, PAGE_SIZE as u64).unwrap();
            ShellMessage::send(messenger, format!("{}", patch), MessageType::Spinner, 0);

            let mut guard = Guard::new(&partition, None);

            ShellMessage::send(messenger, format!("adding {} to {}", patch, guard),
                                MessageType::Spinner,0);
            guard.add(patch);

            ShellMessage::send(messenger, format!("HEVD: {}", style("Applying patch.").green()),
                                MessageType::Spinner, 0);
            guard.start();
            ShellMessage::send(messenger, format!("HEVD: {}", style("Patch applied.").green()),
                                MessageType::Close, 0);

            ShellMessage::sleep_bar(messenger, 30);
            ShellMessage::send(messenger, format!("HEVD: {}", style("Revoking patch").red()),
                                MessageType::Spinner, 0);
            guard.stop();
        }

        let _ = memory::free_virtual_memory(&device, new_code);

        let services: Vec<&str> = "lynxv memguard sentry".split(' ').collect();

        services.iter().rev().for_each(|service| {
            service::stop(service, messenger);
        });

        ShellMessage::send(messenger, format!("HEVD: {}", style("Patch Revoked").red()),
                            MessageType::Close, 0);
        Ok(())
    }
    else {
        Err(format_err!("HEVD driver not found."))
    }
}
