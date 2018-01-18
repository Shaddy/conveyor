
use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use super::failure::Error;
use super::sentry::{io, search};
use super::iochannel::{Device};
use super::sentry::memguard::{ Partition};


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
            .subcommand(super::miscellaneous::bind())
            .subcommand(SubCommand::with_name("device")
                .subcommand(SubCommand::with_name("double-open")))
            .subcommand(super::mem::bind())
            .subcommand(super::interceptions::bind())
			.subcommand(super::patches::bind())
            .subcommand(super::errors::bind())
            .subcommand(super::memguard::bind())
}

pub fn parse(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("memguard",          Some(matches))  => super::memguard::tests(matches, logger),
        ("sentry",            Some(matches))  => super::kernel::tests(matches, logger),
        ("process",           Some(matches))  => super::process::tests(matches, logger),
        ("memory",            Some(matches))  => super::mem::tests(matches, logger),
        ("patches",           Some(matches))  => super::patches::tests(matches, logger),
        ("token",             Some(matches))  => super::token::tests(matches, logger),
        ("errors",            Some(matches))  => super::errors::tests(matches, logger),
        ("device",            Some(matches))  => device_tests(matches, logger),
        ("search-pattern",    Some(matches))  => test_search_pattern(matches, logger),
        ("misc",              Some(matches))  => super::miscellaneous::tests(matches, logger),
        ("interceptions",     Some(matches))  => super::interceptions::tests(matches, logger),
        _                                     => Ok(println!("{}", matches.usage()))
    }
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