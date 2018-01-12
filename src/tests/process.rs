// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::failure::Error;

use super::sentry::{misc};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("process")
                .subcommand(SubCommand::with_name("kernel-base"))
                .subcommand(SubCommand::with_name("system-process"))
                .subcommand(SubCommand::with_name("read-eprocess"))
                .subcommand(SubCommand::with_name("find-eprocess"))
                .subcommand(SubCommand::with_name("list-drivers"))
                .subcommand(SubCommand::with_name("walk-eprocess"))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("current",         Some(matches))  => test_read_eprocess(matches, logger),
        ("list",            Some(matches))  => test_walk_eprocess(matches, logger),
        ("find",            Some(matches))  => test_find_eprocess(matches, logger),
        ("system",          Some(matches))  => test_system_process(matches, logger),
        ("kernel-base",     Some(matches))  => test_kernel_base(matches, logger),
        ("list-drivers",    Some(matches))  => test_list_drivers(matches, logger),
        _                                   => Ok(println!("{}", matches.usage()))
    }
}

fn test_list_drivers(_matches: &ArgMatches, _logger: &Logger) -> Result<(), Error> {
    misc::list_kernel_drivers();
    Ok(())
}

fn test_kernel_base(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    debug!(logger, "base: 0x{:016x}", misc::get_kernel_base());
    Ok(())
}

fn test_system_process(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let system = misc::Process::system();
    debug!(logger, "system: 0x{:016x}", system.object());
    Ok(())
}

fn test_find_eprocess(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    debug!(logger, "{}", misc::WalkProcess::iter()
                                .find(|process| process.name().contains("svchost")).unwrap());
    Ok(())
}

fn test_walk_eprocess(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    misc::WalkProcess::iter().for_each(|process|
    {
            debug!(logger, "{}", process);
    });
    Ok(())
}

fn test_read_eprocess(_matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let current =  misc::WalkProcess::iter()
                            .find(|process| process.name().contains("conveyor")).unwrap();

    debug!(logger, "current-eprocess: 0x{:016x}", current.object());
    Ok(())
}

