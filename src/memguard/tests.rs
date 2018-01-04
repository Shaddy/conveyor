use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;

use std::thread;
use std::time::Duration;

use std::sync::Arc;
use super::misc;

use super::{Partition, Sentinel, Guard, Access, Action};
use super::bucket::Interception;
use super::{core, memory, token};
use super::iochannel::{Device};
use super::memory::{Map};

pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    let target = Arg::with_name("pid").short("p")
                            .required(true)
                            .value_name("PID")
                            .help("process pid target");

    SubCommand::with_name("tests")
            .subcommand(SubCommand::with_name("token")
                .subcommand(SubCommand::with_name("protect")
                            .arg(target.clone()))
                .subcommand(SubCommand::with_name("steal")
                            .arg(target.clone())))
            .subcommand(SubCommand::with_name("process")
                .subcommand(SubCommand::with_name("read-eprocess"))
                .subcommand(SubCommand::with_name("find-eprocess"))
                .subcommand(SubCommand::with_name("walk-eprocess")))
            .subcommand(SubCommand::with_name("memory")
                .subcommand(SubCommand::with_name("read"))
                .subcommand(SubCommand::with_name("virtual"))
                .subcommand(SubCommand::with_name("write"))
                .subcommand(SubCommand::with_name("map")))
            .subcommand(SubCommand::with_name("interceptions")
                .subcommand(SubCommand::with_name("kernel-intercept"))
                .subcommand(SubCommand::with_name("callback-intercept")))
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
                .subcommand(SubCommand::with_name("create-10"))
                .subcommand(SubCommand::with_name("create-and-start"))
                .subcommand(SubCommand::with_name("add-a-region")))
}

pub fn tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("partition",         Some(matches))  => partition(matches, logger),
        ("guards",            Some(matches))  => guard_tests(matches, logger),
        ("regions",           Some(matches))  => region_tests(matches, logger),
        ("memory",            Some(matches))  => memory_tests(matches, logger),
        ("process",           Some(matches))  => process_tests(matches, logger),
        ("token",             Some(matches))  => token_tests(matches, logger),
        ("interceptions",     Some(matches))  => interception_tests(matches, logger),
        _                             => println!("{}", matches.usage())
    }
}

fn interception_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("kernel",      Some(matches))  => test_intercept_kernel_region(matches, logger),
        ("callback",    Some(matches))  => test_interception_callback(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn token_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("protect", Some(matches))        => protect_token(matches, logger),
        ("steal",   Some(matches))        => steal_token(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn steal_token(matches: &ArgMatches, logger: Logger) {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");
    

    let device = Device::new(core::SE_NT_DEVICE_NAME);
    debug!(logger, "elevating privilege of pid {}", pid);
    token::steal_token(&device, 0, pid, token::TokenType::DuplicateSource);
    debug!(logger, "success");
}

fn protect_token(matches: &ArgMatches, logger: Logger) {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");

    let mut process = misc::Process::system();
    let process = process.find(|p| p.id() == pid)
           .expect("can't find client pid");

    let token = process.token() & !0xF;

    debug!(logger, "protecting target pid {} with token 0x{:016x}", 
                        pid, token);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition);
    let region = Sentinel::region(&partition, token, 8, Access::WRITE);
    guard.add(region);

    guard.set_callback(Box::new(move |_| {
        println!("{}: attempt to write 0x{:016x} token", pid, token);

        Action::CONTINUE
    }));

    let duration = Duration::from_secs(60);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);
}

fn process_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("read-eprocess", Some(matches))  => test_read_eprocess(matches, logger),
        ("walk-eprocess", Some(matches))  => test_walk_eprocess(matches, logger),
        ("find-eprocess", Some(matches))  => test_find_eprocess(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

// MEMORY TESTS
fn memory_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        // ("read",  Some(matches))  => test_memory_read(matches, logger),
        ("virtual",  Some(matches))       => test_virtual_memory(matches, logger),
        ("write", Some(matches))          => test_memory_write(matches, logger),
        ("map",   Some(matches))          => test_memory_map(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn test_find_eprocess(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);
    let addr = core::current_process(&device);

    let mut process = misc::Process::new(Arc::new(device), addr);

    process = process.forward();

    debug!(logger, "{}", process.find(|process| process.name().contains("svchost")).unwrap());
}

fn test_walk_eprocess(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);
    let addr = core::current_process(&device);

    let mut process = misc::Process::new(Arc::new(device), addr);

    process = process.forward();

    process.take(5).for_each(|process|
        {
            debug!(logger, "{}", process);
    });
}

fn test_read_eprocess(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    let addr = core::current_process(&device);

    debug!(logger, "current-eprocess: 0x{:016x}", addr);

}

// TODO: Find a more generic kernel pointer
const KERNEL_ADDR: u64 = 0xfffffa800231e9e0;

fn test_virtual_memory(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    debug!(logger, "opened sentry: {:?}", device);

    let mut v: Vec<u8> = Vec::new();

    (0..(0x200 / 4)).for_each(|_| 
    {
        v.push(0xBE);
        v.push(0xBA);
        v.push(0xFE);
        v.push(0xCA);
    });

    let size = v.len();

    debug!(logger, "write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let addr = memory::alloc_virtual_memory(&device, size);

    debug!(logger, "alloc_virtual_memory: 0x{:016x}", addr);

    let written = memory::write_virtual_memory(&device, addr, v);

    debug!(logger, "write_virtual_memory: {} bytes written", written);

    let v = memory::read_virtual_memory(&device, addr, size);

    debug!(logger, "reading 0x{:08x} bytes from 0x{:016x}", addr, size);

    debug!(logger, "read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let output: String = v.iter().enumerate()
                    .map(|(i, b)| 
                    {
                            let mut s = format!("{:02X}", b);
                            if i > 1 && i % 16 == 0 { s += "\n"; }  else { s += " "};
                            s
                    }).collect::<Vec<String>>().join("");;
    
    debug!(logger, "{}", output);

    debug!(logger, "free_virtual_memory: 0x{:016x}", addr);
    memory::free_virtual_memory(&device, addr);
}

fn test_memory_write(_matches: &ArgMatches, _logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);
    let v = memory::read_virtual_memory(&device, KERNEL_ADDR, 0x200);

    memory::write_virtual_memory(&device, KERNEL_ADDR, v);
}


fn test_memory_map(_matches: &ArgMatches, _logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    let map = Map::new(&device, KERNEL_ADDR, 0x200);

    println!("map: {:?}", map);

}

fn create_multiple_partitions(_logger: Logger) {
    println!("creating 3 partitions");
    let _partition1: Partition = Partition::new();
    let _partition2: Partition = Partition::new();
    let _partition3: Partition = Partition::new();
    println!("waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    println!("done, destroying partitions");
}

fn create_partition(_logger: Logger) {
    let partition: Partition = Partition::root();
    println!("created partition: {:?}", partition);
    println!("waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    println!("done, destroying partition");
}

pub fn partition(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create",  Some(_))  => create_partition(logger),
        ("create-multiple",  Some(_))  => create_multiple_partitions(logger),
        ("delete",  Some(_))  => _not_implemented_command(logger),
        ("getinfo", Some(_))  => _not_implemented_command(logger),
        ("setinfo", Some(_))  => _not_implemented_command(logger),
        _                     => println!("{}", matches.usage())
    }
}

// GUARD TESTS
fn guard_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create-and-start", Some(matches))       => start_a_guard(matches, logger),
        ("create-10",        Some(matches))       => create_multiple_guards(matches, logger),
        _                                         => println!("{}", matches.usage())
    }
}


// callback interception tests
// 
//
fn _callback_test(interception: Interception) -> Action {
    println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address, 
                                    interception.access);

    Action::CONTINUE
}

fn test_interception_callback(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition);

    const POOL_SIZE: usize = 0x100;

    println!("allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);

    println!("addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, Access::READ);

    println!("adding {} to {}", region, guard);
    guard.add(region);

    // guard.set_callback(Box::new(_callback_test));
    guard.set_callback(Box::new(|interception| {
        println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address, 
                                        interception.access);

        Action::CONTINUE
    }));
    println!("starting guard");
    guard.start();
    println!("accessing memory 0x{:016x}", addr);
    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);
    println!("stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr);
}

///////////////////////////////////////////////////////////////////////////////////////

fn start_guard_a_second(guard: &Guard) {
    println!("starting {}", guard);
    guard.start();

    let duration = Duration::from_secs(1);
    println!("waiting {:?}", duration);
    thread::sleep(duration);

    println!("stopping {}", guard);
    guard.stop();
}
fn start_a_guard(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let guard = Guard::new(&partition);

    start_guard_a_second(&guard);
}

fn create_multiple_guards(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let _guard = Guard::new(&partition);

    let guards: Vec<Guard> = (0..10).map(|_| {
            let guard = Guard::new(&partition);
            guard
        }).collect();

    println!("guards-created: {}", guards.len());

    println!("enumerate-guards");

    // for guard in Guard::enumerate() {
    //     println!("guard: {}", guards);
    // }

    for guard in guards {
        println!("{}", guard);
    }
}
// REGION TESTS
fn region_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create", Some(matches))  => test_create_region(matches, logger),
        ("enumerate", Some(matches))  => test_enumerate_region(matches, logger),
        ("create-multiple", Some(matches))  => test_create_multiple_regions(matches, logger),
        ("intercept", Some(matches))  => test_intercept_region(matches, logger),
        ("kernel-intercept", Some(matches))  => test_intercept_kernel_region(matches, logger),
        ("regions-inside-guard", Some(matches))  => test_regions_inside_guard(matches, logger),
        _                          => println!("{}", matches.usage())
    }
}

fn test_enumerate_region(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn test_create_multiple_regions(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let _regions: Vec<Sentinel> = (0..10).map(|_| {
            let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, Access::READ);
            println!("{}", region);
            region
        }).collect();
}

fn test_regions_inside_guard(_matches: &ArgMatches, _logger: Logger) {

    let partition: Partition = Partition::root();

    let mut guard: Guard = Guard::new(&partition);

    let regions: Vec<Sentinel> = (0..10).map(|_| {
            let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, Access::READ);
            println!("{}", region);
            region
        }).collect();

    for region in regions {
        guard.add(region);
    }


    start_guard_a_second(&guard);
}

fn test_intercept_kernel_region(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition);

    const POOL_SIZE: usize = 0x100;

    println!("allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);
    println!("addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, Access::READ);

    println!("adding {} to {}", region, guard);

    guard.add(region);
    println!("starting guard");
    guard.start();
    println!("accessing memory 0x{:016x}", addr);

    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);

    println!("stoping guard");
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr);
}

fn test_intercept_region(_matches: &ArgMatches, _logger: Logger) {
    let mut v: Vec<u8> = Vec::new();
    v.push(13);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition);
    let region = Sentinel::region(&partition, v.as_ptr() as u64, 10, Access::READ);
    println!("adding {} to {}", region, guard);
    guard.add(region);
    println!("starting guard, and sleeping 5 seconds");
    guard.start();
    thread::sleep(Duration::from_secs(5));

    // accessing memory
    println!("accessing memory {:?} 5 times", v.as_ptr());
    let _ = v[0];
    let _ = v[0];
    let _ = v[0];
    let _ = v[0];
    let value = v[0];

    println!("value: {}", value);
    println!("sleeping 5 secs");
    thread::sleep(Duration::from_secs(5));
    println!("stoping guard");
    guard.stop();
}

fn test_create_region(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, Access::READ);
    println!("{}", region);
}