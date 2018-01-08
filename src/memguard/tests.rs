use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread, fmt};
use std::time::Duration;

use super::{misc, search};

use super::{Partition, Sentinel, Guard, Access, Action, Filter, MatchType};
use super::bucket::Interception;
use super::{core, memory, token};
use super::iochannel::{Device};
use super::memory::{Map, MapMode};

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
                .subcommand(SubCommand::with_name("kernel-base"))
                .subcommand(SubCommand::with_name("system-process"))
                .subcommand(SubCommand::with_name("read-eprocess"))
                .subcommand(SubCommand::with_name("find-eprocess"))
                .subcommand(SubCommand::with_name("list-drivers"))
                .subcommand(SubCommand::with_name("walk-eprocess")))
            .subcommand(SubCommand::with_name("search-pattern"))
            .subcommand(SubCommand::with_name("memory")
                .subcommand(SubCommand::with_name("read"))
                .subcommand(SubCommand::with_name("virtual"))
                .subcommand(SubCommand::with_name("write"))
                .subcommand(SubCommand::with_name("kernel-map"))
                .subcommand(SubCommand::with_name("map")))
            .subcommand(SubCommand::with_name("interceptions")
                .subcommand(SubCommand::with_name("kernel"))
                .subcommand(SubCommand::with_name("stealth"))
                .subcommand(SubCommand::with_name("analysis"))
                .subcommand(SubCommand::with_name("callback")))
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
        ("search-pattern",    Some(matches))  => test_search_pattern(matches, logger),
        ("interceptions",     Some(matches))  => interception_tests(matches, logger),
        _                             => println!("{}", matches.usage())
    }
}

/////////////////////////////////////////////////////////////////////////
// 
// SEARCH PATTERN TEST
//

fn test_search_pattern(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

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

/////////////////////////////////////////////////////////////////////////
// 
// CALLBACK INTERCEPTION TESTS
//
fn dummy_vector(size: usize) -> Vec<u8> {
    let mut v: Vec<u8> = Vec::new();

    (0..(size / 4)).for_each(|_| 
    {
        v.push(0xBE);
        v.push(0xBA);
        v.push(0xFE);
        v.push(0xCA);
    });

    v
}

fn dump_vector(v: Vec<u8>) -> String {
    v.iter().enumerate()
            .map(|(i, b)| 
            {
                    let mut s = format!("{:02X}", b);
                    if i > 1 && i % 16 == 0 { s += "\n"; }  else { s += " "};
                    s
            }).collect::<Vec<String>>().join("")
}


/////////////////////////////////////////////////////////////////////////
// 
// CALLBACK INTERCEPTION TESTS
//
fn interception_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("kernel",      Some(matches))  => test_intercept_kernel_region(matches, logger),
        ("stealth",     Some(matches))  => test_stealth_interception(matches, logger),
        ("analysis",    Some(matches))  => test_analysis_interception(matches, logger),
        ("callback",    Some(matches))  => test_interception_callback(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn test_analysis_interception(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();

    let mut guard = Guard::new(&partition, Filter::current_process(&partition.device, MatchType::NOT_EQUAL));
    let addr = misc::fixed_procedure_address(misc::get_kernel_base(), "ntoskrnl.exe", "ExAllocatePoolWithTag");
    let region = Sentinel::region(&partition, addr, 
                                  1, 
                                  Some(Action::NOTIFY | Action::INSPECT), 
                                  Access::EXECUTE);

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("0x{:016x} - ExAllocatePoolWithTag", interception.address);
        colorize::info(&message);
        Action::CONTINUE
    }));

    debug!(logger, "starting guard");
    guard.start();

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
    let output = dump_vector(v);
    debug!(logger, "dumping buffer 0x{:016x} \n{}", addr, output);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, Some(Action::NOTIFY | Action::INSPECT), Access::WRITE);

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

    let v = dummy_vector(POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, v);
    debug!(logger, "{} bytes written", bytes);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE);
    if v.iter().any(|&b| b != 0x00) {
        colorize::failed("STEALTH test result has FAILED.");
        let output = dump_vector(v);
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

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ);

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

/////////////////////////////////////////////////////////////////////////
// 
// TOKEN PROTECTION TESTS
//
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

    let process = misc::WalkProcess::iter().find(|p| p.id() == pid)
           .expect("can't find client pid");

    let token = process.token() & !0xF;

    debug!(logger, "protecting target pid {} with token 0x{:016x}", 
                        pid, token);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);
    let region = Sentinel::region(&partition, token, 8, None, Access::WRITE);
    guard.add(region);

    guard.set_callback(Box::new(move |_| {
        println!("{}: attempt to write 0x{:016x} token", pid, token);

        Action::CONTINUE
    }));

    let duration = Duration::from_secs(60);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);
}

/////////////////////////////////////////////////////////////////////////
// 
// MEMORY TESTS
//
fn memory_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("virtual",      Some(matches))  => test_virtual_memory(matches, logger),
        ("write",        Some(matches))  => test_memory_write(matches, logger),
        ("map",          Some(matches))  => test_memory_map(matches, logger),
        ("kernel-map",   Some(matches))  => test_kernel_map(matches, logger),
        _                                => println!("{}", matches.usage())
    }
}

fn test_kernel_map(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    struct TestStruct {
        first:  u64,
        second: u64, 
    }

    impl fmt::Display for TestStruct {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TestStruct {{ first: 0x{:016x}, second: 0x{:016x} }}", self.first, self.second)
        }
    }

    let map = memory::KernelAlloc::<TestStruct>::new(&device);

    debug!(logger, "TestStruct: allocated {} bytes at:
                    kernel: 0x{:016x}
                    user:   0x{:016x}", map.size(), map.kernel_ptr(), map.as_ptr() as u64);

    unsafe {
        let test = &mut *map.as_mut_ptr();
        test.first = 0x11223344;
        test.second = 0x55667788;
    }

    debug!(logger, "reading kernel pointer 0x{:016x}", map.kernel_ptr());

    let v = memory::read_virtual_memory(&device, map.kernel_ptr(), map.size());


    let u: &TestStruct = unsafe{ &*map.as_ptr() };
    let k: &TestStruct = unsafe { &*(v.as_ptr() as *const TestStruct) };

    debug!(logger, "from-user: {}", u);
    debug!(logger, "from-kernel: {}", k);
}

fn test_virtual_memory(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    debug!(logger, "opened sentry: {:?}", device);

    let v = dummy_vector(0x200);

    let size = v.len();

    debug!(logger, "write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let addr = memory::alloc_virtual_memory(&device, size);

    debug!(logger, "alloc_virtual_memory: 0x{:016x}", addr);

    let written = memory::write_virtual_memory(&device, addr, v);

    debug!(logger, "write_virtual_memory: {} bytes written", written);

    let v = memory::read_virtual_memory(&device, addr, size);

    debug!(logger, "reading 0x{:08x} bytes from 0x{:016x}", addr, size);

    debug!(logger, "read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());

    let output = dump_vector(v);
    
    debug!(logger, "{}", output);

    debug!(logger, "free_virtual_memory: 0x{:016x}", addr);
    memory::free_virtual_memory(&device, addr);
}

fn test_memory_write(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);
    debug!(logger, "reading virtual memory");
    let v = memory::read_virtual_memory(&device, KERNEL_ADDR, 0x200);

    debug!(logger, "writting virtual memory");
    memory::write_virtual_memory(&device, KERNEL_ADDR, v);
}


fn test_memory_map(_matches: &ArgMatches, logger: Logger) {
    let device = Device::new(core::SE_NT_DEVICE_NAME);

    let map = Map::new(&device, KERNEL_ADDR, 0x200, Some(MapMode::UserMode));

    debug!(logger, "map: {:?}", map);

}

/////////////////////////////////////////////////////////////////////////
// 
// PROCESS TESTS
//
fn process_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("read-eprocess",   Some(matches))  => test_read_eprocess(matches, logger),
        ("walk-eprocess",   Some(matches))  => test_walk_eprocess(matches, logger),
        ("find-eprocess",   Some(matches))  => test_find_eprocess(matches, logger),
        ("kernel-base",     Some(matches))  => test_kernel_base(matches, logger),
        ("system-process",  Some(matches))  => test_system_process(matches, logger),
        ("list-drivers",    Some(matches))  => test_list_drivers(matches, logger),
        _                                 => println!("{}", matches.usage())
    }
}

fn test_list_drivers(_matches: &ArgMatches, _logger: Logger) {
    misc::list_kernel_drivers();
}


fn test_kernel_base(_matches: &ArgMatches, logger: Logger) {
    debug!(logger, "base: 0x{:016x}", misc::get_kernel_base());
}

fn test_system_process(_matches: &ArgMatches, logger: Logger) {
    let system = misc::Process::system();
    debug!(logger, "system: 0x{:016x}", system.object());
}

fn test_find_eprocess(_matches: &ArgMatches, logger: Logger) {
    debug!(logger, "{}", misc::WalkProcess::iter()
                                .find(|process| process.name().contains("svchost")).unwrap());
}

fn test_walk_eprocess(_matches: &ArgMatches, logger: Logger) {
    misc::WalkProcess::iter().for_each(|process|
    {
            debug!(logger, "{}", process);
    });
}

/////////////////////////////////////////////////////////////////////////
// 
// MEMORY TESTS
//
fn test_read_eprocess(_matches: &ArgMatches, logger: Logger) {
    let current =  misc::WalkProcess::iter()
                            .find(|process| process.name().contains("conveyor")).unwrap();

    debug!(logger, "current-eprocess: 0x{:016x}", current.object());

}

// TODO: Find a more generic kernel pointer
const KERNEL_ADDR: u64 = 0xfffffa800231e9e0;

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
        _                                         => println!("{}", matches.usage())
    }
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
        ("intercept",            Some(matches)) => test_intercept_region(matches, logger),
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

fn test_intercept_kernel_region(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    debug!(logger, "allocating pool");
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE);
    debug!(logger, "addr: 0x{:016x}", addr);

    let region = Sentinel::region(&partition, addr, POOL_SIZE as u64, None, Access::READ);

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

fn test_intercept_region(_matches: &ArgMatches, logger: Logger) {
    let mut v: Vec<u8> = Vec::new();
    v.push(13);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);
    let region = Sentinel::region(&partition, v.as_ptr() as u64, 10, None, Access::READ);
    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);
    debug!(logger, "starting guard, and sleeping 5 seconds");
    guard.start();
    thread::sleep(Duration::from_secs(5));

    // accessing memory
    debug!(logger, "accessing memory {:?} 5 times", v.as_ptr());
    let _ = v[0];
    let _ = v[0];
    let _ = v[0];
    let _ = v[0];
    let value = v[0];

    debug!(logger, "value: {}", value);
    debug!(logger, "sleeping 5 secs");
    thread::sleep(Duration::from_secs(5));
    debug!(logger, "stoping guard");
    guard.stop();
}

fn test_create_region(_matches: &ArgMatches, logger: Logger) {
    let partition: Partition = Partition::root();
    let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, None, Access::READ);
    debug!(logger, "{}", region);
}