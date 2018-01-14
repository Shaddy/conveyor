// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::cli::colorize;

use std::{thread};
use std::time::Duration;
use super::failure::Error;


use super::sentry::memguard::{Partition, Region, Guard, Access, Action};
use super::sentry::{misc, io, token};
use super::iochannel::{Device};

pub fn bind() -> App<'static, 'static> {
    let target = Arg::with_name("pid").short("p")
                            .required(true)
                            .value_name("PID")
                            .help("process pid target");

    SubCommand::with_name("token")
                    .subcommand(SubCommand::with_name("protect")
                                .arg(target.clone()))
                    .subcommand(SubCommand::with_name("duplicate")
                                .arg(target.clone()))
                    .subcommand(SubCommand::with_name("hijack")
                                .arg(target.clone()))
}

pub fn tests(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    match matches.subcommand() {
        ("protect",     Some(matches))        => protect_token(matches, logger),
        ("duplicate",   Some(matches))        => duplicate_token(matches, logger),
        ("hijack",      Some(matches))        => hijack_token(matches, logger),
        _                                     => Ok(println!("{}", matches.usage()))
    }
}

fn duplicate_token(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");
    

    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    debug!(logger, "elevating privilege of pid {}", pid);
    token::steal_token(&device, 0, pid, token::TokenType::DuplicateSource);
    debug!(logger, "success");
    Ok(())
}


fn hijack_token(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");
    

    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    debug!(logger, "elevating privilege of pid {}", pid);
    token::steal_token(&device, 0, pid, token::TokenType::HijackSystem);
    debug!(logger, "success");

    Ok(())
}

fn protect_token(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");

    let process = misc::WalkProcess::iter().find(|p| p.id() == pid)
           .expect("can't find client pid");

    let token = process.token() & !0xF;
    let token_offset = misc::get_offset("_EPROCESS.Token");

    debug!(logger, "protecting target pid {} with token 0x{:016x}", 
                        pid, token);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    // TODO: Do it in a stable way.
    // pointer to token (duplicateway)
    // let region = Region::new(&partition, token, 8, None, Access::WRITE);

    let region = Region::new(&partition, process.object() + u64::from(token_offset), 8, None, Access::WRITE).unwrap();
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("Attempt to write at 0x{:016X} - IGNORING", interception.address);
        colorize::info(&message);
        Action::STEALTH
    }));

    let duration = Duration::from_secs(20);
    debug!(logger, "waiting {:?}", duration);
    thread::sleep(duration);
    Ok(())
}