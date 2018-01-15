// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread};
use std::time::Duration;

use super::failure::Error;
use super::sentry::{memory, misc};
use super::sentry::memguard::{Patch, Partition, Guard};


pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("patches")
                .subcommand(SubCommand::with_name("patch-vuln"))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("patch-vuln",      Some(matches))  => test_patch_driver(matches, logger),
        _                                   => Ok(println!("{}", matches.usage()))
    }
}

fn test_patch_driver(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    const PAGE_SIZE: usize = 0x1000;
    const PATCH_PAGE: u64 = 0x5000;
    const PATCH_OFFSET: u64 = 0xBEC;

    if let Some(driver) = misc::Drivers::iter().find(|driver| driver.name.contains("HEVD")) {

		let partition = Partition::root();
		
        let patch_base = driver.base + PATCH_PAGE;
        let patch = vec![0x90; 6];

        let new_code = memory::alloc_virtual_memory(&partition.device, PAGE_SIZE).unwrap();
        let _ = memory::copy_virtual_memory(&partition.device, patch_base, new_code, PAGE_SIZE);
        let _ = memory::write_virtual_memory(&partition.device, new_code + PATCH_OFFSET, patch);

        let patch = Patch::new(&partition, patch_base, new_code, PAGE_SIZE as u64).unwrap();
        debug!(logger, "{}", patch);

        let mut guard = Guard::new(&partition, None);

        debug!(logger, "adding {} to {}", patch, guard);
        guard.add(patch);

        debug!(logger, "starting guard, and sleeping 30 secs");
        guard.start();
        colorize::success("HEVD patch applied.");
        thread::sleep(Duration::from_secs(30));
        guard.stop();
		colorize::info("HEVD patch revoked.");
		Ok(())
    }
	else {
        colorize::failed("HEVD driver not found.");
        Err(format_err!("HEVD driver not found."))
	}
}