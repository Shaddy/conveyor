
use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

use std::{thread};
use std::time::Duration;

use super::failure::Error;
use super::sentry::{io, misc};
use super::iochannel::{Device};
use super::rand::Rng;

use super::sentry::memguard::{Response,
                              Partition,
                              Region,
                              Guard,
                              Access,
                              Action,
                              Filter,
                              MatchType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memguard")
            .subcommand(SubCommand::with_name("fuzz")
                .subcommand(SubCommand::with_name("partition-process")))
            .subcommand(SubCommand::with_name("partition")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("delete")))
            .subcommand(SubCommand::with_name("regions")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("enumerate"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("regions-inside-guard")))
            .subcommand(SubCommand::with_name("guards")
                .subcommand(SubCommand::with_name("filter"))
                .subcommand(SubCommand::with_name("create-10"))
                .subcommand(SubCommand::with_name("create-and-start"))
                .subcommand(SubCommand::with_name("add-a-region")))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition",         Some(matches))  => partition_tests(matches, logger),
        ("guards",            Some(matches))  => guard_tests(matches, logger),
        ("regions",           Some(matches))  => region_tests(matches, logger),
        ("fuzz",              Some(matches))  => fuzz_tests(matches, logger),
        _                                     => Ok(println!("{}", matches.usage()))
    }
}

/////////////////////////////////////////////////////////////////////////
//
// FUZZ TESTS
//
pub fn fuzz_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition-process",   Some(_))       => test_fuzz_partition_process(logger),
        _                                      => Ok(println!("{}", matches.usage()))
    }
}


fn test_fuzz_partition_process(logger: &Logger) -> Result<(), Error> {
    let mut rng = super::rand::thread_rng();

    (0..1000).for_each(|round| {
        {
            debug!(logger, "exeuting round: {}", round);
            let elapse = rng.gen::<u8>();
            let duration = Duration::from_millis(u64::from(elapse));
            let _partition = Partition::root();
            let _device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
            if let Ok(process) = misc::Process::system() {
                let _ = process.to_string();
            }
            thread::sleep(duration);
        }
    });

    Ok(())
}



/////////////////////////////////////////////////////////////////////////
//
// PARTITION TESTS
//
pub fn partition_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("create",          Some(_))           => create_partition(logger),
        ("create-multiple", Some(_))           => create_multiple_partitions(logger),
        ("delete",          Some(_)) |
        ("getinfo",         Some(_)) |
        ("setinfo",         Some(_))           => super::common::_not_implemented_command(logger),
        _                                      => Ok(println!("{}", matches.usage()))
    }
}

fn create_multiple_partitions(logger: &Logger) -> Result<(), Error> {
    debug!(logger, "creating 3 partitions");
    let _partition1: Partition = Partition::new().unwrap();
    let _partition2: Partition = Partition::new().unwrap();
    let _partition3: Partition = Partition::new().unwrap();
    debug!(logger, "waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    debug!(logger, "done, destroying partitions");
    Ok(())
}

fn create_partition(logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    debug!(logger, "created partition: {:?}", partition);
    debug!(logger, "waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    debug!(logger, "done, destroying partition");

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// GUARD TESTS
//
fn guard_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("create-and-start", Some(matches))       => start_a_guard(matches, logger),
        ("create-10",        Some(matches))       => create_multiple_guards(matches, logger),
        ("filter",           Some(matches))       => test_guard_filters(matches, logger),
        _                                         => Ok(println!("{}", matches.usage()))
    }
}

fn test_guard_filters(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition = Partition::root();
    let filter = Filter::process(&partition.device, "notepad", MatchType::EQUAL)
                                .expect("can't find \"notepad\" process");

    // // this is totally a non recommended way
    // let pid = filter.filter.Conditions[0].Value.Value;

    let mut guard = Guard::new(&partition, Some(filter));

    let addr = misc::kernel_export_address(&partition.device, misc::get_kernel_base(), "ZwCreateKey")
                            .expect("can't find ZwCreateKey");

    let region = Region::new(&partition, addr,
                            1,
                            Some(Action::NOTIFY | Action::INSPECT),
                            Access::EXECUTE).unwrap();

    debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("executing 0x{:016x}", interception.address);
        Response::new(Some(message), Action::CONTINUE)
    }));

    debug!(logger, "starting guard");
    guard.start();

    let duration = Duration::from_secs(10);

    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);

    debug!(logger, "stoping guard");
    guard.stop();

    Ok(())
}

fn start_guard_a_second(guard: &Guard, logger: &Logger) -> Result<(), Error> {
    debug!(logger, "starting {}", guard);
    guard.start();

    let duration = Duration::from_secs(1);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);

    debug!(logger, "stopping {}", guard);
    guard.stop();

    Ok(())
}
fn start_a_guard(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let guard = Guard::new(&partition, None);

    start_guard_a_second(&guard, logger)?;

    Ok(())
}

fn create_multiple_guards(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _guard = Guard::new(&partition, None);

    let guards: Vec<Guard> = (0..10).map(|_| { Guard::new(&partition, None) }).collect();

    debug!(logger, "guards-created: {}", guards.len());

    debug!(logger, "enumerate-guards");

    // for guard in Guard::enumerate() {
    //     println!("guard: {}", guards);
    // }

    for guard in guards {
        debug!(logger, "{}", guard);
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// REGION TESTS
//
fn region_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("create",               Some(matches)) => test_create_region(matches, logger),
        ("enumerate",            Some(matches)) => test_enumerate_region(matches, logger),
        ("create-multiple",      Some(matches)) => test_create_multiple_regions(matches, logger),
        ("regions-inside-guard", Some(matches)) => test_regions_inside_guard(matches, logger),
        _                                       => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn test_enumerate_region(_matches: &ArgMatches, _logger: &Logger) -> Result<(), Error> {
    unimplemented!()
}

fn test_create_multiple_regions(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _regions: Vec<Region> = (0..10).map(|_| {
            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            debug!(logger, "{}", region);
            region
        }).collect();

    Ok(())
}

fn test_regions_inside_guard(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {

    let partition: Partition = Partition::root();

    let mut guard: Guard = Guard::new(&partition, None);

    let regions: Vec<Region> = (0..10).map(|_| {
            // let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();

            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            println!("{}", region);
            region
        }).collect();

    for region in regions {
        guard.add(region);
    }

    start_guard_a_second(&guard, logger)?;

    Ok(())
}

fn test_create_region(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
    debug!(logger, "{}", region);

    Ok(())
}
