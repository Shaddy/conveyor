
use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;
use super::console::{Term, style};
use super::indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time};

use self::time::Duration;

use super::failure::Error;
use super::sentry::{io, search};
use super::iochannel::{Device};
use super::sentry::memguard::{ Partition, ObjectFilter };

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};


/////////////////////////////////////////////////////////////////////////
//
// DUMMY UNUSED COMMANDS
//
pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: &Logger) -> Result<(), Error> {
    unimplemented!()
}

fn _not_implemented_command(_logger: &Logger) -> Result<(), Error> {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("tests")
            .subcommand(super::token::bind())
            .subcommand(super::kernel::bind())
            .subcommand(super::process::bind())
            .subcommand(SubCommand::with_name("search-pattern"))
            .subcommand(SubCommand::with_name("bars"))
            .subcommand(SubCommand::with_name("monitor"))
            .subcommand(super::miscellaneous::bind())
            .subcommand(SubCommand::with_name("device")
                .subcommand(SubCommand::with_name("double-open")))
            .subcommand(super::mem::bind())
            .subcommand(super::interceptions::bind())
			.subcommand(super::patches::bind())
            .subcommand(super::errors::bind())
            .subcommand(super::memguard::bind())
}

pub fn parse(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("memguard",          Some(matches))  => super::memguard::tests(matches, logger, &tx),
        ("sentry",            Some(matches))  => super::kernel::tests(matches, logger),
        ("process",           Some(matches))  => super::process::tests(matches, logger),
        ("memory",            Some(matches))  => super::mem::tests(matches, logger),
        ("patches",           Some(matches))  => super::patches::tests(matches, logger),
        ("token",             Some(matches))  => super::token::tests(matches, logger),
        ("errors",            Some(matches))  => super::errors::tests(matches, logger),
        ("device",            Some(matches))  => device_tests(matches, logger),
        ("monitor",           Some(matches))  => monitor_tests(matches, logger),
        ("bars",              Some(matches))  => bar_tests(matches, logger),
        ("search-pattern",    Some(matches))  => test_search_pattern(matches, logger),
        ("misc",              Some(matches))  => super::miscellaneous::tests(matches, logger),
        ("interceptions",     Some(matches))  => super::interceptions::tests(matches, logger),
        _                                     => Ok(println!("{}", matches.usage()))
    }
}

/////////////////////////////////////////////////////////////////////////
//
// BAR TESTS
//
#[allow(unused_variables)]
fn bar_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {

    let bar = ProgressBar::new(5);
    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        bar.inc(1);
        // ...
    }
    bar.finish();
    let bar = ProgressBar::new(5);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));
    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        bar.inc(1);
        // ...
    }
    bar.finish();

    Ok(())
}


/////////////////////////////////////////////////////////////////////////
//
// MONITOR TESTS
//
#[allow(unused_variables)]
fn monitor_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {

    let term = Term::stdout();

    println!("[*] creating {}.", style("ObjectMonitor").cyan());

    let bar = ProgressBar::new(30);

    for _ in 0..30 {
        thread::sleep(Duration::from_millis(50));
        bar.inc(1);
    }
    bar.finish();
    println!("[?] are you {}?", style("ready").red());
    let bar = ProgressBar::new(5);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));

    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        bar.inc(1);
    }
    bar.finish();

    (0..5).for_each(|n| {
        let mut msg = String::new();

        (0..n).for_each(|_| {
            msg.push_str("    ");
        });

        msg.push_str("GOOOO");

        println!("[!] {}", style(msg).red());
    });

    let bar = ProgressBar::new(5);

    for _ in 0..5 {
        thread::sleep(Duration::from_secs(1));
        bar.inc(1);
    }
    bar.finish();
    let filter = ObjectFilter::new()
                .expect("can't create object filter");

    println!("[!] {}.", style("starting").magenta());
    filter.start().expect("unable to start filter");


    let bar = ProgressBar::new(30);
    bar.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .progress_chars("##-"));

    for _ in 0..30 {
        thread::sleep(Duration::from_secs(1));
        bar.inc(1);
        // ...
    }
    bar.finish();

    println!("[!] {}.", style("stopping").magenta());
    filter.stop().expect("unable to start filter");
    Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// DEVICE TESTS
//
fn device_tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("double-open",  Some(matches))  => test_double_open(matches, logger),
        _                                => Ok(println!("{}", matches.usage()))
    }
}

fn consume_device(device: Device) {
    println!("good bye - {:?}", device);
}

fn test_double_open(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
        let partition = Partition::root();
        let device_one = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
        debug!(logger, "dropping: device_one");
        consume_device(device_one);
        debug!(logger, "dropped: device_one");
        debug!(logger, "creating a partition");

        if io::delete_partition(&partition.device, partition.id).is_err() {
            colorize::failed("TEST HAS FAILED");
        } else {
            colorize::success("TEST IS SUCCESS");
        }

        Ok(())
}

/////////////////////////////////////////////////////////////////////////
//
// SEARCH PATTERN TEST
//

fn test_search_pattern(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

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
                                          Some("KeSynchronizeExecution")) {
        debug!(logger, "switch-context: 0x{:016x}", offset);
    }

    Ok(())
}
