// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread, mem, fmt};
use std::time::Duration;

use super::failure::Error;
use super::common;
use super::iochannel::Device;
use super::sentry::{memory, search, io};
use super::sentry::memguard::{Response,
                              Interception,
                              Partition,
                              Region,
                              Guard,
                              Access,
                              Action,
                              Filter,
                              MatchType};

use super::ssdt::SSDT_FUNCTIONS;

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("interceptions")
                .subcommand(SubCommand::with_name("kernel"))
                .subcommand(SubCommand::with_name("stealth"))
                .subcommand(SubCommand::with_name("analysis-normal"))
                .subcommand(SubCommand::with_name("analysis-no-message"))
                .subcommand(SubCommand::with_name("callback"))
                .subcommand(SubCommand::with_name("ssdt"))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("kernel",               Some(matches))  => test_intercept_kernel_region(matches, logger),
        ("stealth",              Some(matches))  => test_stealth_interception(matches, logger),
        ("analysis-normal",      Some(matches))  => test_analysis_normal(matches, logger),
        ("analysis-no-message",  Some(matches))  => test_analysis_no_message(matches, logger),
        ("callback",             Some(matches))  => test_interception_callback(matches, logger),
        ("ssdt",                 Some(matches))  => test_ssdt_address(matches, logger),
        _                                 => Ok(println!("{}", matches.usage()))
    }
}

fn test_ssdt_address(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    debug!(logger, "{:?}", find_ssdt_address());
    Ok(())
}

#[repr(C)]
struct ServiceTable {
    pub address: u64,
    pub tables: u64,
    pub count: u32,
    pub arguments: u64
}

impl fmt::Debug for ServiceTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceTable {{
                    address:   0x{:016x},
                    tables:    0x{:016x},
                    count:            {},
                    arguments: 0x{:016x},
        }}", self.address,
             self.tables,
             self.count,
             self.arguments)
    }
}

fn find_ssdt_address() -> ServiceTable {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("sentry device");
    let pattern = vec![0x48, 0x89, 0xA3, 0xD8,
                       0x01, 0x00, 0x00, 0x8B,
                       0xF8, 0xC1, 0xEF, 0x07,
                       0x83, 0xE7, 0x20, 0x25];

    let address = search::pattern(&device,
                                  "ntoskrnl",
                                  &pattern,
                                  Some("ZwCreateResourceManager"))
                                  .expect("unable to find SSDT pattern");

    let instruction = pattern.len() as u64 + 7;

    let rva = memory::read_u32(&device, address + instruction).unwrap() as i32;

    let ssdt_reference = address.wrapping_add(rva as u64) + instruction + 4;

    println!("ref: 0x{:016x}", ssdt_reference);
    let data = memory::read_virtual_memory(&device, ssdt_reference, mem::size_of::<ServiceTable>())
                    .expect("error reading ServiceTable");

    let ssdt: ServiceTable = unsafe { mem::transmute_copy(&*data.as_ptr()) };

    ssdt
}

enum Print {
    Show,
    Hide
}

fn test_analysis_no_message(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    test_analysis_interception_messaged(Print::Hide, &logger)
}

fn test_analysis_normal(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    test_analysis_interception_messaged(Print::Show, &logger)
}


fn test_analysis_interception_messaged(show: Print, logger: &Logger) -> Result<(), Error> {

    debug!(logger, "discovering SSDT");
    let ssdt = find_ssdt_address();

    debug!(logger, "{:?}", ssdt);

    let address = ssdt.address;

    debug!(logger, "found at 0x{:16x}", address);

    let partition = Partition::root();

    let filter = Filter::process(&partition.device, "notepad", MatchType::EQUAL)
                            .expect("can't find notepad process");

    let mut guard = Guard::new(&partition, Some(filter));

    let region = Region::new(&partition, address,
                              ssdt.count as u64 * 4,
                              Some(Action::NOTIFY | Action::INSPECT),
                              Access::READ)
                            .expect("can't create region");

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(move |interception| {
        let index = interception.address.wrapping_sub(address) / 4;
        let name = SSDT_FUNCTIONS.get(index as usize).unwrap_or(&"<invalid-index>");

        let message = format!("{}", name);
        match show {
            Print::Show => Response::new(Some(message), Action::CONTINUE),
            Print::Hide => Response::new(None, Action::CONTINUE),
        }
    }));

    debug!(logger, "starting guard");
    guard.start();

    let duration = Duration::from_secs(60);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);

    debug!(logger, "stoping guard");
    guard.stop();
    Ok(())
}

//
// This test aims to demostrate that we are able to ignore any write to any memory address
//
fn test_stealth_interception(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x10;

    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();
    debug!(logger, "new pool: 0x{:016x} ({} bytes)", addr, POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, vec![0; POOL_SIZE]).unwrap();
    debug!(logger, "zeroed {} bytes", bytes);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    let output = common::dump_vector(&v);
    debug!(logger, "dumping buffer 0x{:016x} \n{}", addr, output);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, Some(Action::NOTIFY | Action::INSPECT), Access::WRITE).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("Attempt to write at 0x{:016X} - IGNORING", interception.address);
        // TODO: set optional color for response
        // colorize::info(&message);
        Response::new(Some(message), Action::STEALTH)
    }));

    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);

    let v = common::dummy_vector(POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, v).unwrap();
    debug!(logger, "{} bytes written", bytes);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    if v.iter().any(|&b| b != 0x00) {
        colorize::failed("STEALTH test result has FAILED.");
        let output = common::dump_vector(&v);
        debug!(logger, "inspecting buffer 0x{:016x}", addr);
        colorize::warning(&output);
    } else {
        colorize::success("STEALTH test result has PASSED.");
    }

    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();
    Ok(())
}

// example of declared function as callback
#[allow(dead_code)]
fn callback_test(interception: Interception) -> Action {
    println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address,
                                    interception.access);

    Action::CONTINUE
}

fn test_interception_callback(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();

    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    // guard.set_callback(Box::new(callback_test));
    guard.set_callback(Box::new(|interception| {
        let message = format!("The offensive address is 0x{:016X} (accessing {:?})", interception.address,
                                        interception.access);

        Response::new(Some(message), Action::CONTINUE)
    }));
    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);
    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();
    Ok(())
}

fn test_intercept_kernel_region(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();
    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    debug!(logger, "adding {} to {}", region, guard);

    guard.add(region);
    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);

    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();

    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();

    Ok(())
}

// fn test_intercept_region(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
//     let mut v: Vec<u8> = Vec::new();
//     v.push(13);

//     let partition: Partition = Partition::root();
//     let mut guard = Guard::new(&partition, None);
//     let region = Region::new(&partition, v.as_ptr() as u64, 10, None, Access::READ);
//     debug!(logger, "adding {} to {}", region, guard);
//     guard.add(region);
//     debug!(logger, "starting guard, and sleeping 5 seconds");
//     guard.start();
//     thread::sleep(Duration::from_secs(5));

//     // accessing memory
//     debug!(logger, "accessing memory {:?} 5 times", v.as_ptr());
//     let _ = v[0];
//     let _ = v[0];
//     let _ = v[0];
//     let _ = v[0];
//     let value = v[0];

//     debug!(logger, "value: {}", value);
//     debug!(logger, "sleeping 5 secs");
//     thread::sleep(Duration::from_secs(5));
//     debug!(logger, "stoping guard");
//     guard.stop();
// }
