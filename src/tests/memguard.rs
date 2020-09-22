use super::clap::{App, ArgMatches, SubCommand};

use std::thread;
use std::time::Duration;

use super::failure::Error;
use super::iochannel::Device;
use super::rand::Rng;
use super::sentry::{io, misc};

use super::sentry::memguard::{
    Access, Action, Filter, Guard, MatchType, Partition, Region, Response,
};

use super::cli::output::{MessageType, ShellMessage};
use super::console::style;
use std::sync::mpsc::Sender;

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memguard")
        .subcommand(
            SubCommand::with_name("fuzz").subcommand(SubCommand::with_name("partition-process")),
        )
        .subcommand(
            SubCommand::with_name("partition")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("delete")),
        )
        .subcommand(
            SubCommand::with_name("regions")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("enumerate"))
                .subcommand(SubCommand::with_name("create-multiple"))
                .subcommand(SubCommand::with_name("regions-inside-guard")),
        )
        .subcommand(
            SubCommand::with_name("guards")
                .subcommand(SubCommand::with_name("filter"))
                .subcommand(SubCommand::with_name("create-10"))
                .subcommand(SubCommand::with_name("create-and-start"))
                .subcommand(SubCommand::with_name("add-a-region")),
        )
}

pub fn tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition", Some(matches)) => partition_tests(matches, messenger),
        ("guards", Some(matches)) => guard_tests(matches, messenger),
        ("regions", Some(matches)) => region_tests(matches, messenger),
        ("fuzz", Some(matches)) => fuzz_tests(matches, messenger),
        _ => Ok(println!("{}", matches.usage())),
    }
}

/////////////////////////////////////////////////////////////////////////
//
// FUZZ TESTS
//
pub fn fuzz_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("partition-process", Some(_)) => test_fuzz_partition_process(messenger),
        _ => Ok(println!("{}", matches.usage())),
    }
}

fn test_fuzz_partition_process(messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let mut rng = super::rand::thread_rng();

    let bar = ShellMessage::new(
        messenger,
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}".to_string(),
        0,
        1000,
    );
    (0..1000).for_each(|_| {
        let elapse = rng.gen::<u8>();
        let duration = Duration::from_millis(u64::from(elapse));
        let _partition = Partition::root();
        let _device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
        if let Ok(process) = misc::Process::system() {
            let _ = process.to_string();
            bar.inc(messenger, 1);
        }

        thread::sleep(duration);
    });
    ShellMessage::send(
        messenger,
        "Executed all iterations.".to_string(),
        MessageType::Close,
        0,
    );
    bar.complete(messenger);

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// PARTITION TESTS
//
pub fn partition_tests(
    matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(_)) => create_partition(messenger),
        ("create-multiple", Some(_)) => create_multiple_partitions(messenger),
        ("delete", Some(_)) | ("getinfo", Some(_)) | ("setinfo", Some(_)) => {
            super::common::_not_implemented_command(messenger)
        }
        _ => Ok(println!("{}", matches.usage())),
    }
}

fn create_multiple_partitions(messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    ShellMessage::send(
        messenger,
        format!("Creating {} partitions...", style("3").bold().magenta()),
        MessageType::Spinner,
        0,
    );
    // debug!(logger, "creating 3 partitions");
    let _partition1: Partition = Partition::new().unwrap();
    let _partition2: Partition = Partition::new().unwrap();
    let _partition3: Partition = Partition::new().unwrap();
    ShellMessage::send(
        messenger,
        format!("Waiting {} seconds...", style("5").underlined().yellow()),
        MessageType::Spinner,
        0,
    );
    ShellMessage::sleep_bar(messenger, 5);
    // debug!(logger, "Waiting 5 seconds");
    // thread::sleep(Duration::from_secs(5));
    ShellMessage::send(
        messenger,
        format!("Partitions destroyed::{}", style("Done!").green()),
        MessageType::Close,
        0,
    );
    // debug!(logger, "done, destroying partitions");
    Ok(())
}

fn create_partition(messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();

    ShellMessage::send(
        messenger,
        format!("Created partition {:?}", style(partition).bold().magenta()),
        MessageType::Spinner,
        0,
    );
    // debug!(logger, "created partition: {:?}", partition);
    ShellMessage::send(
        messenger,
        format!("Waiting {} seconds...", style("5").underlined().yellow()),
        MessageType::Spinner,
        0,
    );
    // debug!(logger, "waiting 5 seconds");
    ShellMessage::sleep_bar(messenger, 5);
    // thread::sleep(Duration::from_secs(5));
    ShellMessage::send(
        messenger,
        format!("Partition destroyed::{}", style("Done!").green()),
        MessageType::Close,
        0,
    );
    // debug!(logger, "done, destroying partition");

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// GUARD TESTS
//
fn guard_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create-and-start", Some(matches)) => start_a_guard(matches, messenger),
        ("create-10", Some(matches)) => create_multiple_guards(matches, messenger),
        ("filter", Some(matches)) => test_guard_filters(matches, messenger),
        _ => Ok(println!("{}", matches.usage())),
    }
}

fn test_guard_filters(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let partition = Partition::root();
    let filter = Filter::process(&partition.device, "notepad", MatchType::EQUAL)
        .expect("can't find \"notepad\" process");

    // // this is totally a non recommended way
    // let pid = filter.filter.Conditions[0].Value.Value;

    let mut guard = Guard::new(&partition, Some(filter));

    let addr =
        misc::kernel_export_address(&partition.device, misc::get_kernel_base(), "ZwCreateKey")
            .expect("can't find ZwCreateKey");

    let region = Region::new(
        &partition,
        addr,
        1,
        Some(Action::NOTIFY | Action::INSPECT),
        Access::EXECUTE,
    )
    .unwrap();

    ShellMessage::send(
        messenger,
        format!("Adding {} to {} ...", region, guard),
        MessageType::Spinner,
        0,
    );
    // debug!(logger, "adding {} to {}", region, guard);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("executing 0x{:016x}", interception.address);
        Response::new(Some(message), Action::CONTINUE)
    }));

    ShellMessage::send(
        messenger,
        "Starting guard...".to_string(),
        MessageType::Spinner,
        0,
    );
    // debug!(logger, "starting guard");
    guard.start();

    ShellMessage::send(
        messenger,
        format!("Waiting {}", style("10 secs").cyan()),
        MessageType::Spinner,
        0,
    );
    ShellMessage::sleep_bar(messenger, 10);
    // thread::sleep(duration);

    // debug!(logger, "stoping guard");
    ShellMessage::send(
        messenger,
        format!("{}", style("Stoping guard...").yellow()),
        MessageType::Spinner,
        0,
    );
    guard.stop();
    ShellMessage::send(
        messenger,
        "Guard Stopped.".to_string(),
        MessageType::Close,
        0,
    );
    ShellMessage::send(
        messenger,
        format!("{}", style("Done!").green()),
        MessageType::Close,
        1,
    );

    Ok(())
}

fn start_guard_a_second(guard: &Guard, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    // debug!(logger, "starting {}", guard);
    ShellMessage::send(
        messenger,
        format!("starting {}", guard),
        MessageType::Spinner,
        0,
    );
    guard.start();

    let duration = Duration::from_secs(1);
    // debug!(logger, "waiting {:?}", duration);
    ShellMessage::send(
        messenger,
        format!("waiting {:?}", duration),
        MessageType::Spinner,
        0,
    );
    thread::sleep(duration);

    // debug!(logger, "stopping {}", guard);
    ShellMessage::send(
        messenger,
        format!("stopping {}", guard),
        MessageType::Close,
        0,
    );
    guard.stop();

    Ok(())
}
fn start_a_guard(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let guard = Guard::new(&partition, None);

    start_guard_a_second(&guard, messenger)?;

    Ok(())
}

fn create_multiple_guards(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _guard = Guard::new(&partition, None);

    let guards: Vec<Guard> = (0..10).map(|_| Guard::new(&partition, None)).collect();

    // debug!(logger, "guards-created: {}", guards.len());
    ShellMessage::send(
        messenger,
        format!("{} {}", style("guards-created:").blue(), guards.len()),
        MessageType::Spinner,
        0,
    );

    // debug!(logger, "enumerate-guards");
    ShellMessage::send(
        messenger,
        "enumerate-guards".to_string(),
        MessageType::Spinner,
        0,
    );

    // for guard in Guard::enumerate() {
    //     println!("guard: {}", guards);
    // }

    for guard in guards {
        // debug!(logger, "{}", guard);
        ShellMessage::send(
            messenger,
            format!("{}", style(guard).on_blue()),
            MessageType::Close,
            0,
        );
    }

    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// REGION TESTS
//
fn region_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("create", Some(matches)) => test_create_region(matches, messenger),
        ("enumerate", Some(matches)) => test_enumerate_region(matches, messenger),
        ("create-multiple", Some(matches)) => test_create_multiple_regions(matches, messenger),
        ("regions-inside-guard", Some(matches)) => test_regions_inside_guard(matches, messenger),
        _ => {
            println!("{}", matches.usage());
            Ok(())
        }
    }
}

fn test_enumerate_region(
    _matches: &ArgMatches,
    _messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    unimplemented!()
}

fn test_create_multiple_regions(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let _regions: Vec<Region> = (0..10)
        .map(|_| {
            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            // debug!(logger, "{}", region);
            ShellMessage::send(
                messenger,
                format!("{}", style(&region).blue().dim()),
                MessageType::Close,
                0,
            );
            region
        })
        .collect();

    Ok(())
}

fn test_regions_inside_guard(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let partition: Partition = Partition::root();

    let mut guard: Guard = Guard::new(&partition, None);

    let regions: Vec<Region> = (0..10)
        .map(|_| {
            // let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();

            let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
            // println!("{}", region);
            ShellMessage::send(
                messenger,
                format!("{}", style(&region).blue().dim()),
                MessageType::Close,
                0,
            );
            region
        })
        .collect();

    for region in regions {
        guard.add(region);
    }

    start_guard_a_second(&guard, messenger)?;

    Ok(())
}

fn test_create_region(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let region = Region::new(&partition, 0xCAFE_BABE, 0x1000, None, Access::READ).unwrap();
    // debug!(logger, "{}", region);
    ShellMessage::send(
        messenger,
        format!("{}", style(&region).blue().dim()),
        MessageType::Close,
        0,
    );

    Ok(())
}
