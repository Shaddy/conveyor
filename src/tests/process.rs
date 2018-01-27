// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::failure::Error;

use super::sentry::misc;

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("process")
        .subcommand(SubCommand::with_name("kernel-base"))
        .subcommand(SubCommand::with_name("system"))
        .subcommand(SubCommand::with_name("current"))
        .subcommand(SubCommand::with_name("find"))
        .subcommand(SubCommand::with_name("list-drivers"))
        .subcommand(SubCommand::with_name("list"))
}

pub fn tests(
    matches: &ArgMatches,
    logger: &Logger,
    tx: &Sender<ShellMessage>,
) -> Result<(), Error> {
    match matches.subcommand() {
        ("current", Some(matches)) => test_read_eprocess(matches, &tx),
        ("list", Some(matches)) => test_walk_eprocess(matches, &tx),
        ("find", Some(matches)) => test_find_eprocess(matches, &tx),
        ("system", Some(matches)) => test_system_process(matches, &tx),
        ("kernel-base", Some(matches)) => test_kernel_base(matches, &tx),
        ("list-drivers", Some(matches)) => test_list_drivers(matches, &tx),
        _ => Ok(println!("{}", matches.usage())),
    }
}

fn test_list_drivers(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    misc::list_kernel_drivers();
    Ok(())
}

fn test_kernel_base(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    // debug!(logger, "base: 0x{:016x}", misc::get_kernel_base());
    ShellMessage::send(
        &tx,
        format!("base: 0x{:016x}", misc::get_kernel_base()),
        MessageType::close,
        0,
    );
    Ok(())
}

fn test_system_process(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let system = misc::Process::system().expect("system process");
    // debug!(logger, "system: 0x{:016x}", system.object());
    ShellMessage::send(
        &tx,
        format!("base: 0x{:016x}", system.object()),
        MessageType::close,
        0,
    );
    Ok(())
}

fn test_find_eprocess(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    ShellMessage::send(
        &tx,
        format!(
            "{}",
            misc::WalkProcess::iter()
                .find(|process| process.name().contains("svchost"))
                .unwrap()
        ),
        MessageType::close,
        0,
    );
    // debug!(logger, "{}", misc::WalkProcess::iter()
    //                             .find(|process| process.name().contains("svchost")).unwrap());
    Ok(())
}

fn test_walk_eprocess(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    misc::WalkProcess::iter().for_each(|process| {
        // debug!(logger, "{}", process);
        ShellMessage::send(
            &tx,
            format!("{}", process),
            MessageType::close,
            0,
        );
    });
    Ok(())
}

fn test_read_eprocess(_matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let current = misc::WalkProcess::iter()
        .find(|process| process.name().contains("conveyor"))
        .unwrap();

    // debug!(logger, "current-eprocess: 0x{:016x}", current.object());
    ShellMessage::send(
        &tx,
        format!("Current-eprocess: 0x{:016x}", current.object()),
        MessageType::close,
        0,
    );
    Ok(())
}
