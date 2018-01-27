
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

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

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

pub fn tests(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition",         Some(matches))  => partition_tests(matches, logger, &tx),
        ("guards",            Some(matches))  => guard_tests(matches, &tx),
        ("regions",           Some(matches))  => region_tests(matches, logger, &tx),
        ("fuzz",              Some(matches))  => fuzz_tests(matches, logger, &tx),
        _                                     => Ok(println!("{}", matches.usage()))
    }
}

/////////////////////////////////////////////////////////////////////////
//
// FUZZ TESTS
//
pub fn fuzz_tests(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition-process",   Some(_))       => test_fuzz_partition_process(logger, &tx),
        _                                      => Ok(println!("{}", matches.usage()))
    }
}


fn test_fuzz_partition_process(logger: &Logger,tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let mut rng = super::rand::thread_rng();


    (0..1000).for_each(|round| {
        {
            ShellMessage::send(&tx, format!("Executing round {}",round), MessageType::spinner, 0);
            // debug!(logger, "exeuting round: {}", round);
            let elapse = rng.gen::<u8>();
            let duration = Duration::from_millis(u64::from(elapse));
            let _partition = Partition::root();
            let _device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
            if let Ok(process) = misc::Process::system() {
                let _ = process.to_string();
            }
            thread::sleep(duration);
        }
        ShellMessage::send(&tx, "Executed all iterations.".to_string(), MessageType::close, 0);

    });

    Ok(())
}



/////////////////////////////////////////////////////////////////////////
//
// PARTITION TESTS
//
pub fn partition_tests(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create",          Some(_))           => create_partition(&tx),
        ("create-multiple", Some(_))           => create_multiple_partitions(&tx),
        ("delete",          Some(_)) |
        ("getinfo",         Some(_)) |
        ("setinfo",         Some(_))           => super::common::_not_implemented_command(logger),
        _                                      => Ok(println!("{}", matches.usage()))
    }
}

fn create_multiple_partitions(tx: &Sender<ShellMessage>) -> Result<(), Error> {
    ShellMessage::send(&tx, "Creating 3 partitions...".to_string(), MessageType::spinner, 0);
    // debug!(logger, "creating 3 partitions");
    let _partition1: Partition = Partition::new().unwrap();
    let _partition2: Partition = Partition::new().unwrap();
    let _partition3: Partition = Partition::new().unwrap();
    ShellMessage::send(&tx, "Waiting 5 seconds...".to_string(), MessageType::spinner, 0);
    // debug!(logger, "Waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    ShellMessage::send(&tx, "Done! Destroying partitions.".to_string(), MessageType::close, 0);
    // debug!(logger, "done, destroying partitions");
    Ok(())
}

fn create_partition(tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();

    ShellMessage::send(&tx, format!("Created partition {:?}",partition), MessageType::spinner, 0);
    // debug!(logger, "created partition: {:?}", partition);
    ShellMessage::send(&tx, "Waiting 5 seconds...".to_string(), MessageType::spinner, 0);
    // debug!(logger, "waiting 5 seconds");
    thread::sleep(Duration::from_secs(5));
    ShellMessage::send(&tx, "Done! Destroying partitions.".to_string(), MessageType::close, 0);
    // debug!(logger, "done, destroying partition");

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// GUARD TESTS
//
fn guard_tests(matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create-and-start", Some(matches))       => start_a_guard(matches, &tx),
        ("create-10",        Some(matches))       => create_multiple_guards(matches, &tx),
        ("filter",           Some(matches))       => test_guard_filters(matches, &tx),
        _                                         => Ok(println!("{}", matches.usage()))
    }
}

fn test_guard_filters(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
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

    ShellMessage::send(&tx, format!("Adding {} to {} ...",region, guard), MessageType::spinner, 0);
    // debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("executing 0x{:016x}", interception.address);
        Response::new(Some(message), Action::CONTINUE)
    }));

    ShellMessage::send(&tx, "Starting guard...".to_string(), MessageType::spinner, 0);
    // debug!(logger, "starting guard");
    guard.start();

    let duration = Duration::from_secs(10);

    // debug!(logger, "waiting {:?}", duration);
    ShellMessage::send(&tx, format!("Waiting {:?}", duration), MessageType::spinner, 0);
    thread::sleep(duration);

    // debug!(logger, "stoping guard");
    ShellMessage::send(&tx, "Stoping guard...".to_string(), MessageType::spinner, 0);
    guard.stop();
    ShellMessage::send(&tx, "Guard Stopped.".to_string(), MessageType::close, 0);
    ShellMessage::send(&tx, "Done!".to_string(), MessageType::close, 1);

    Ok(())
}

fn start_guard_a_second(guard: &Guard,tx: &Sender<ShellMessage>) -> Result<(), Error> {
    // debug!(logger, "starting {}", guard);
    ShellMessage::send(&tx, format!("starting {}", guard), MessageType::spinner, 0);
    guard.start();

    let duration = Duration::from_secs(1);
    // debug!(logger, "waiting {:?}", duration);
    ShellMessage::send(&tx, format!("waiting {:?}", duration), MessageType::spinner, 0);
    thread::sleep(duration);

    // debug!(logger, "stopping {}", guard);
    ShellMessage::send(&tx, format!("stopping {}", guard), MessageType::close, 0);
    guard.stop();

    Ok(())
}
fn start_a_guard(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let guard = Guard::new(&partition, None);

    start_guard_a_second(&guard, &tx)?;

    Ok(())
}

fn create_multiple_guards(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _guard = Guard::new(&partition, None);

    let guards: Vec<Guard> = (0..10).map(|_| { Guard::new(&partition, None) }).collect();

    // debug!(logger, "guards-created: {}", guards.len());
    ShellMessage::send(&tx, format!("guards-created: {}", guards.len()), MessageType::spinner, 0);

    // debug!(logger, "enumerate-guards");
    ShellMessage::send(&tx, "enumerate-guards".to_string(), MessageType::spinner, 0);

    // for guard in Guard::enumerate() {
    //     println!("guard: {}", guards);
    // }

    for guard in guards {
        // debug!(logger, "{}", guard);
        ShellMessage::send(&tx, format!("{}",guard), MessageType::spinner, 0);
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// REGION TESTS
//
fn region_tests(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create",               Some(matches)) => test_create_region(matches, logger, &tx),
        ("enumerate",            Some(matches)) => test_enumerate_region(matches, logger, &tx),
        ("create-multiple",      Some(matches)) => test_create_multiple_regions(matches, logger, &tx),
        ("regions-inside-guard", Some(matches)) => test_regions_inside_guard(matches, logger, &tx),
        _                                       => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn test_enumerate_region(_matches: &ArgMatches, _logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    unimplemented!()
}

fn test_create_multiple_regions(_matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _regions: Vec<Region> = (0..10).map(|_| {
            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            // debug!(logger, "{}", region);
            ShellMessage::send(&tx, format!("{}",region), MessageType::close, 0);
            region
        }).collect();

    Ok(())
}

fn test_regions_inside_guard(_matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {

    let partition: Partition = Partition::root();

    let mut guard: Guard = Guard::new(&partition, None);

    let regions: Vec<Region> = (0..10).map(|_| {
            // let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();

            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            // println!("{}", region);
            ShellMessage::send(&tx, format!("{}",region), MessageType::close, 0);
            region
        }).collect();

    for region in regions {
        guard.add(region);
    }

    start_guard_a_second(&guard, &tx)?;

    Ok(())
}

fn test_create_region(_matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
    // debug!(logger, "{}", region);
    ShellMessage::send(&tx, format!("{}",region), MessageType::close, 0);

    Ok(())
}
