
use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread};
use std::time::Duration;

use super::common;
use super::sentry::memguard::{ Partition, Sentinel, Guard, Access, Action, Filter, MatchType};
use super::sentry::{search, io, memory};
use super::iochannel::{Device};

/////////////////////////////////////////////////////////////////////////
// 
// DUMMY UNUSED COMMANDS
//
pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("tests")
            .subcommand(super::token::bind())
            .subcommand(super::kernel::bind())
            .subcommand(super::process::bind())
            .subcommand(SubCommand::with_name("search-pattern"))
            .subcommand(super::miscellaneous::bind())
            .subcommand(SubCommand::with_name("device")
                .subcommand(SubCommand::with_name("double-open")))
            .subcommand(super::mem::bind())
            .subcommand(super::interceptions::bind())
            .subcommand(SubCommand::with_name("partition")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("delete")))
            .subcommand(SubCommand::with_name("regions")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("intercept"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("regions-inside-guard"))
                .subcommand(SubCommand::with_name("delete"))
                .subcommand(SubCommand::with_name("enumerate"))
                .subcommand(SubCommand::with_name("info")))
            .subcommand(SubCommand::with_name("guards")
                .subcommand(SubCommand::with_name("filter"))
                .subcommand(SubCommand::with_name("create-10"))
                .subcommand(SubCommand::with_name("create-and-start"))
                .subcommand(SubCommand::with_name("add-a-region")))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("partition",         Some(matches))  => partition(matches, logger),
        ("guards",            Some(matches))  => guard_tests(matches, logger),
        ("regions",           Some(matches))  => region_tests(matches, logger),
        ("memory",            Some(matches))  => super::mem::tests(matches, logger),
        ("process",           Some(matches))  => super::process::tests(matches, logger),
        ("token",             Some(matches))  => super::token::tests(matches, logger),
        ("device",            Some(matches))  => device_tests(matches, logger),
        ("sentry",            Some(matches))  => super::kernel::tests(matches, logger),
        ("search-pattern",    Some(matches))  => test_search_pattern(matches, logger),
        ("misc",              Some(matches))  => super::miscellaneous::tests(matches, logger),
        ("interceptions",     Some(matches))  => super::interceptions::tests(matches, logger),
        _                             => println!("{}", matches.usage())
    }
}

/////////////////////////////////////////////////////////////////////////
// 
// DEVICE TESTS
//
fn device_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("double-open",  Some(matches))  => test_double_open(matches, logger),
        _                                => println!("{}", matches.usage())
    }
}

fn consume_device(device: Device) {
    println!("good bye - {:?}", device);
}

fn test_double_open(_matches: &ArgMatches, logger: Logger) {
        let partition = Partition::root();
        let device_one = Device::new(io::SE_NT_DEVICE_NAME);
        debug!(logger, "dropping: device_one");
        consume_device(device_one);
        debug!(logger, "dropped: device_one");
        debug!(logger, "creating a partition");

        if let Err(_) = io::delete_partition(&partition.device, partition.id) {
            colorize::failed("TEST HAS FAILED");
        } else {
            colorize::success("TEST IS SUCCESS");
        }
}

/////////////////////////////////////////////////////////////////////////
// 
// SEARCH PATTERN TEST
//

fn test_search_pattern(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(io::SE_NT_DEVICE_NAME);

    let switch_context_pattern: Vec<u8> = vec![0x89, 0x60, 0x18, 0x4C, 
                                               0x89, 0x68, 0x20, 0x4C, 
                                               0x89, 0x70, 0x28, 0x4C, 
                                               0x89, 0x78, 0x30, 0x65, 
                                               0x48, 0x8B, 0x1C, 0x25, 
                                               0x20, 0x00, 0x00, 0x00, 
                                               0x48, 0x8B, 0xF9];

    if let Some(offset) = search::pattern(&device, 
                                          "ntos", 
                                          &switch_context_pattern, 
                                          "KeSynchronizeExecution") {
        debug!(logger, "switch-context: 0x{:016x}", offset);
    }
}

fn create_multiple_partitions(logger: Logger) {
    debug!(logger, "creating 3 partitions");
    let _partition1: Partition = Partition::new();
    let _partition2: Partition = Partition::new();
    let _partition3: Partition = Partition::new();
    debug!(logger, "waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    debug!(logger, "done, destroying partitions");
}

fn create_partition(logger: Logger) {
    let partition: Partition = Partition::root();
    debug!(logger, "created partition: {:?}", partition);
    debug!(logger, "waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    debug!(logger, "done, destroying partition");
}

pub fn partition(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create",  Some(_))           => create_partition(logger),
        ("create-multiple",  Some(_))  => create_multiple_partitions(logger),
        ("delete",  Some(_))           => _not_implemented_command(logger),
        ("getinfo", Some(_))           => _not_implemented_command(logger),
        ("setinfo", Some(_))           => _not_implemented_command(logger),
        _                              => println!("{}", matches.usage())
    }
}

/////////////////////////////////////////////////////////////////////////
// 
// GUARD TESTS
//
fn guard_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create-and-start", Some(matches))       => start_a_guard(matches, logger),
        ("create-10",        Some(matches))       => create_multiple_guards(matches, logger),
        ("filter",           Some(matches))       => test_guard_filters(matches, &logger),
        _                                         => println!("{}", matches.usage())
    }
}

fn test_guard_filters(_matches: &ArgMatches, logger: &Logger) {
    let partition = Partition::root();
    let filter = Filter::process(&partition.device, "notepad", MatchType::EQUAL).unwrap();

    debug!(logger, "alloc: {:?}", filter.alloc);

    let before = filter.alloc.as_slice().iter()
                  .map(|&b| b)
                  .collect::<Vec<u8>>();

    debug!(logger, "{}", common::dump_vector(before));

    let mut guard = Guard::new(&partition, Some(filter));

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);

    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ);

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

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

fn start_guard_a_second(guard: &Guard, logger: &Logger) {
    debug!(logger, "starting {}", guard);
    guard.start();

    let duration = Duration::from_secs(1);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);

    debug!(logger, "stopping {}", guard);
    guard.stop();
}
fn start_a_guard(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let guard = Guard::new(&partition, None);

    start_guard_a_second(&guard, &logger);
}

fn create_multiple_guards(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let _guard = Guard::new(&partition, None);

    let guards: Vec<Guard> = (0..10).map(|_| {
            let guard = Guard::new(&partition, None);
            guard
        }).collect();

    debug!(logger, "guards-created: {}", guards.len());

    debug!(logger, "enumerate-guards");

    // for guard in Guard::enumerate() {
    //     println!("guard: {}", guards);
    // }

    for guard in guards {
        debug!(logger, "{}", guard);
    }
}

/////////////////////////////////////////////////////////////////////////
// 
// REGION TESTS
//
fn region_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create",               Some(matches)) => test_create_region(matches, logger),
        ("enumerate",            Some(matches)) => test_enumerate_region(matches, logger),
        ("create-multiple",      Some(matches)) => test_create_multiple_regions(matches, logger),
        ("regions-inside-guard", Some(matches)) => test_regions_inside_guard(matches, logger),
        _                                       => println!("{}", matches.usage())
    }
}

fn test_enumerate_region(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn test_create_multiple_regions(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let _regions: Vec<Sentinel> = (0..10).map(|_| {
            let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, None, Access::READ);
            debug!(logger, "{}", region);
            region
        }).collect();
}

fn test_regions_inside_guard(_matches: &ArgMatches, logger: Logger) {

    let partition: Partition = Partition::root();

    let mut guard: Guard = Guard::new(&partition, None);

    let regions: Vec<Sentinel> = (0..10).map(|_| {
            let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, None, Access::READ);
            println!("{}", region);
            region
        }).collect();

    for region in regions {
        guard.add(region);
    }

    start_guard_a_second(&guard, &logger);
}

fn test_create_region(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, None, Access::READ);
    debug!(logger, "{}", region);
}
