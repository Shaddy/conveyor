use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

// partition functions
// use super::core::{create_partition,
//                   get_partition_option,
//                   set_partition_option,
//                   delete_partition};

use std::thread;
use std::time::Duration;
use super::{Partition, Sentinel, Guard, Access};

pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memguard")
        .about("enable protection features")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("features")
            .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
            .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
            .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis")))
        .subcommand(SubCommand::with_name("tests")
            .subcommand(SubCommand::with_name("partition")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("delete")))
                //     .arg(Arg::with_name("ID")
                //             .help("Sets the partition id to delete")
                //             .required(true)
                //             .value_name("PARTITION_ID")))
                // .subcommand(SubCommand::with_name("getinfo")
                //     .arg(Arg::with_name("ID")
                //             .help("Sets the partition id to delete")
                //             .required(true)
                //             .value_name("PARTITION_ID")
                //             .index(1))
                //     .arg(Arg::with_name("OPT")
                //             .help("Specifies the enumerator of the option")
                //             .required(true)
                //             .value_name("OPTION_NUMBER")
                //             .index(2)))
                // .subcommand(SubCommand::with_name("setinfo")
                //     .arg(Arg::with_name("ID")
                //             .help("Sets the partition id to get info")
                //             .required(true)
                //             .value_name("PARTITION_ID")
                //             .index(1))
                //     .arg(Arg::with_name("OPT")
                //             .help("Specifies the enumerator of the option")
                //             .required(true)
                //             .value_name("OPTION_NUMBER")
                //             .index(2))
                //     .arg(Arg::with_name("VAL")
                //             .help("Specifies the value of to set")
                //             .required(true)
                //             .value_name("VALUE")
                //             .index(3))))
            .subcommand(SubCommand::with_name("regions")
                .subcommand(SubCommand::with_name("create"))
                .subcommand(SubCommand::with_name("create_multiple"))
                .subcommand(SubCommand::with_name("regions_inside_guard"))
                .subcommand(SubCommand::with_name("delete"))
                .subcommand(SubCommand::with_name("info")))
            .subcommand(SubCommand::with_name("guards")
                .subcommand(SubCommand::with_name("create_10"))
                .subcommand(SubCommand::with_name("create_and_start"))
                .subcommand(SubCommand::with_name("add_a_region"))
            ))
}

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

    for guard in guards {
        println!("{}", guard);
    }
}

fn guard_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create_and_start", Some(matches))  => start_a_guard(matches, logger),
        ("create_10",        Some(matches))  => create_multiple_guards(matches, logger),
        _                                    => println!("{}", matches.usage())
    }
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

fn test_create_region(_matches: &ArgMatches, _logger: Logger) {
    let partition: Partition = Partition::root();
    let region = Sentinel::region(&partition, 0xCAFEBABE, 0x1000, Access::READ);
    println!("{}", region);
}

fn region_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create", Some(matches))  => test_create_region(matches, logger),
        ("create_multiple", Some(matches))  => test_create_multiple_regions(matches, logger),
        ("regions_inside_guard", Some(matches))  => test_regions_inside_guard(matches, logger),
        _                          => println!("{}", matches.usage())
    }
}

pub fn tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("partition", Some(matches))  => partition(matches, logger),
        ("guards",    Some(matches))  => guard_tests(matches, logger),
        ("regions",   Some(matches))  => region_tests(matches, logger),
        _                             => println!("{}", matches.usage())
    }
}

pub fn partition(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create",  Some(_))  => _not_implemented_command(logger),
        ("delete",  Some(_))  => _not_implemented_command(logger),
        ("getinfo", Some(_))  => _not_implemented_command(logger),
        ("setinfo", Some(_))  => _not_implemented_command(logger),
        _                     => println!("{}", matches.usage())
    }
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("features", Some(_))        => _not_implemented_command(logger),
        ("region", Some(_))          => _not_implemented_command(logger),
        ("tests", Some(matches))     => tests(matches, logger),
        _                            => println!("{}", matches.usage())
    }
}