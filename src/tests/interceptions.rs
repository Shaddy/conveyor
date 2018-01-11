// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread};
use std::time::Duration;

use super::common;
use super::sentry::{memory};
use super::sentry::memguard::{Interception, Partition, Sentinel, Guard, Access, Action, Filter, MatchType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("interceptions")
                .subcommand(SubCommand::with_name("kernel"))
                .subcommand(SubCommand::with_name("stealth"))
                .subcommand(SubCommand::with_name("analysis"))
                .subcommand(SubCommand::with_name("callback"))
}

pub fn tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("kernel",      Some(matches))  => test_intercept_kernel_region(matches, logger),
        ("stealth",     Some(matches))  => test_stealth_interception(matches, logger),
        ("analysis",    Some(matches))  => test_analysis_interception(matches, logger),
        ("callback",    Some(matches))  => test_interception_callback(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn test_analysis_interception(_matches: &ArgMatches, logger: Logger) {
    let partition = Partition::root();

    let mut guard = Guard::new(&partition, Filter::current_process(&partition.device, MatchType::NOT_EQUAL));
    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);

    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    // let addr = misc::fixed_procedure_address(misc::get_kernel_base(), "ntoskrnl.exe", "ZwCreateKey");
    // let region = Sentinel::region(&partition, addr, 
    //                               1, 
    //                               Some(Action::NOTIFY | Action::INSPECT), 
    //                               Access::EXECUTE);

    // debug!(logger, "adding {} to {}", region, guard);
    // guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("reading 0x{:016x}", interception.address);
        println!("{}", message);
        Action::CONTINUE
    }));

    debug!(logger, "starting guard");
    guard.start();

    debug!(logger, "allocating pool");
    let _ = memory::read_virtual_memory(&partition.device, addr, 10);
    let _ = memory::read_virtual_memory(&partition.device, addr, 5);
    let _ = memory::read_virtual_memory(&partition.device, addr, 4);
    let _ = memory::read_virtual_memory(&partition.device, addr, 1);
    let duration = Duration::from_secs(60);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);

    debug!(logger, "stoping guard");
    guard.stop();
}

//
// This test aims to demostrate that we are able to ignore any write to any memory address
//
fn test_stealth_interception(_matches: &ArgMatches, logger: Logger) {

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x10;

    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);
    debug!(logger, "new pool: 0x{:016x} ({} bytes)", addr, POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, vec![0; POOL_SIZE]);
    debug!(logger, "zeroed {} bytes", bytes);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);
    let output = common::dump_vector(v);
    debug!(logger, "dumping buffer 0x{:016x} \n{}", addr, output);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, Some(Action::NOTIFY | Action::INSPECT), Access::WRITE).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("Attempt to write at 0x{:016X} - IGNORING", interception.address);
        colorize::info(&message);
        Action::STEALTH
    }));

    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);

    let v = common::dummy_vector(POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, v);
    debug!(logger, "{} bytes written", bytes);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);
    if v.iter().any(|&b| b != 0x00) {
        colorize::failed("STEALTH test result has FAILED.");
        let output = common::dump_vector(v);
        debug!(logger, "inspecting buffer 0x{:016x}", addr);
        colorize::warning(&output);
    } else {
        colorize::success("STEALTH test result has PASSED.");
    }

    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr);
}

// example of declared function as callback
#[allow(dead_code)]
fn callback_test(interception: Interception) -> Action {
    println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address, 
                                    interception.access);

    Action::CONTINUE
}

fn test_interception_callback(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);

    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    // guard.set_callback(Box::new(callback_test));
    guard.set_callback(Box::new(|interception| {
        println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address, 
                                        interception.access);

        Action::CONTINUE
    }));
    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);
    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);
    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr);
}

fn test_intercept_kernel_region(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);
    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    debug!(logger, "adding {} to {}", region, guard);

    guard.add(region);
    debug!(logger, "starting guard");
    guard.start();
    debug!(logger, "accessing memory 0x{:016x}", addr);

    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);

    debug!(logger, "stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr);
}

// fn test_intercept_region(_matches: &ArgMatches, logger: Logger) {
//     let mut v: Vec<u8> = Vec::new();
//     v.push(13);

//     let partition: Partition = Partition::root();
//     let mut guard = Guard::new(&partition, None);
//     let region = Sentinel::region(&partition, v.as_ptr() as u64, 10, None, Access::READ);
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

