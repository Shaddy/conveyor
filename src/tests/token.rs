// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;

use std::{thread};
use std::time::Duration;
use super::failure::Error;


use super::sentry::memguard::{Response, Partition, Region, Guard, Access, Action};
use super::sentry::{misc, io, token};
use super::iochannel::{Device};


use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

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

pub fn tests(matches: &ArgMatches,  tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("protect",     Some(matches))        => protect_token(matches, &tx),
        ("duplicate",   Some(matches))        => duplicate_token(matches, &tx),
        ("hijack",      Some(matches))        => hijack_token(matches, &tx),
        _                                     => Ok(println!("{}", matches.usage()))
    }
}

fn duplicate_token(matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");


    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    // debug!(logger, "elevating privilege of pid {}", pid);
        ShellMessage::send(&tx, format!("Elevating privilege of pid {}", pid), MessageType::spinner,0);
    token::steal_token(&device, 0, pid, token::TokenType::DuplicateSource);
    // debug!(logger, "success");
        ShellMessage::send(&tx, format!("Success"), MessageType::close,0);
    Ok(())
}


fn hijack_token(matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");


    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    // debug!(logger, "elevating privilege of pid {}", pid);
        ShellMessage::send(&tx, format!("Elevating privilege of pid {}", pid), MessageType::spinner,0);
    token::steal_token(&device, 0, pid, token::TokenType::HijackSystem);
    // debug!(logger, "success");
        ShellMessage::send(&tx, format!("Success"), MessageType::close,0);

    Ok(())
}

fn protect_token(matches: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let pid: u64 = matches.value_of("pid")
                     .expect("can't extract PID from arguments")
                     .parse()
                     .expect("error parsing pid");

    let process = misc::WalkProcess::iter().find(|p| p.id() == pid)
           .expect("can't find client pid");

    let token = process.token() & !0xF;
    let token_offset = misc::get_offset("_EPROCESS.Token").expect("Token offset");

    // debug!(logger, "protecting target pid {} with token 0x{:016x}",
    //                     pid, token);
    ShellMessage::send(&tx, format!("Protecting target pid {} with token 0x{:016x}",
                        pid, token), MessageType::spinner,0);

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    // TODO: Do it in a stable way.
    // pointer to token (duplicateway)
    let token_region = Region::new(&partition, token, 8, None, Access::WRITE).unwrap();
    guard.add(token_region);
    let pointer_region = Region::new(&partition, process.object() + u64::from(token_offset), 8, None, Access::WRITE).unwrap();
    guard.add(pointer_region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("0x{:016X} - IGNORING", interception.address);
        Response::new(Some(message), Action::STEALTH)
    }));

    let duration = Duration::from_secs(20);
    // debug!(logger, "waiting {:?}", duration);
    ShellMessage::send(&tx, format!("Waiting {:?}...", duration), MessageType::spinner,0);
    thread::sleep(duration);
    ShellMessage::send(&tx, "Done!".to_string(), MessageType::close,0);
    Ok(())
}
