// Copyright © ByteHeed.  All rights reserved.

use super::clap::Subcommand;

use super::failure::Error;

use super::iochannel::Device;
use super::sentry::memguard::{Access, Action, Guard, Partition, Region, Response};
use super::sentry::{io, misc, token};

use super::cli::output::{MessageType, ShellMessage};
use super::console::style;
use std::sync::mpsc::Sender;

// pub fn bind() -> App<'static, 'static> {
//     let target = Arg::with_name("pid").short("p")
//                             .required(true)
//                             .value_name("PID")
//                             .help("process pid target");
//
//     SubCommand::with_name("token")
//                     .subcommand(SubCommand::with_name("protect")
//                                 .arg(target.clone()))
//                     .subcommand(SubCommand::with_name("duplicate")
//                                 .arg(target.clone()))
//                     .subcommand(SubCommand::with_name("hijack")
//                                 .arg(target.clone()))
// }
//
// pub fn parse(matches: &ArgMatches,  messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     match matches.subcommand() {
//         ("protect",     Some(matches))        => protect_token(matches, messenger),
//         ("duplicate",   Some(matches))        => duplicate_token(matches, messenger),
//         ("hijack",      Some(matches))        => hijack_token(matches, messenger),
//         _                                     => Ok(println!("{}", matches.usage()))
//     }
// }
//
// fn duplicate_token(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let pid: u64 = matches.value_of("pid")
//                      .expect("can't extract PID from arguments")
//                      .parse()
//                      .expect("error parsing pid");
//
//
//     let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
//     // debug!(logger, "elevating privilege of pid {}", pid);
//         ShellMessage::send(messenger, format!("Elevating privilege of pid {}", style(pid).on_blue()), MessageType::Close,0);
//     token::steal_token(&device, 0, pid, token::TokenType::DuplicateSource);
//     // debug!(logger, "success");
//         ShellMessage::send(messenger, format!("{}",style("Success!").bold().green()), MessageType::Close,0);
//     Ok(())
// }
//
//
// fn hijack_token(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let pid: u64 = matches.value_of("pid")
//                      .expect("can't extract PID from arguments")
//                      .parse()
//                      .expect("error parsing pid");
//
//
//     let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
//     // debug!(logger, "elevating privilege of pid {}", pid);
//         ShellMessage::send(messenger, format!("Elevating privilege of pid {}", style(pid).on_blue()), MessageType::Close,0);
//     token::steal_token(&device, 0, pid, token::TokenType::HijackSystem);
//     // debug!(logger, "success");
//         ShellMessage::send(messenger, format!("{}",style("Success!").bold().green()), MessageType::Close,0);
//
//     Ok(())
// }
//
// fn protect_token(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let pid: u64 = matches.value_of("pid")
//                      .expect("can't extract PID from arguments")
//                      .parse()
//                      .expect("error parsing pid");
//
//     let process = misc::WalkProcess::iter().find(|p| p.id() == pid)
//            .expect("can't find client pid");
//
//     let token = process.token() & !0xF;
//     let token_offset = misc::get_offset("_EPROCESS.Token").expect("Token offset");
//
//     ShellMessage::send(messenger, format!("Protecting target pid {} with token {}",
//                         style(pid).blue(), style(format!("0x{:016x}",token)).cyan()), MessageType::Spinner,0);
//
//     let partition: Partition = Partition::root();
//     let mut guard = Guard::new(&partition, None);
//
//     // TODO: Do it in a stable way.
//     // pointer to token (duplicateway)
//     let token_region = Region::new(&partition, token, 8, None, Access::WRITE).unwrap();
//     guard.add(token_region);
//     let pointer_region = Region::new(&partition, process.object() + u64::from(token_offset), 8, None, Access::WRITE).unwrap();
//     guard.add(pointer_region);
//
//     guard.set_callback(Box::new(|interception| {
//         let message = format!("0x{:016X} - IGNORING", interception.address);
//         Response::new(Some(message), Action::STEALTH)
//     }));
//
//     ShellMessage::send(messenger, format!("Waiting {} seconds...",style("20").underlined().yellow()), MessageType::Spinner,0);
//     guard.start();
//     ShellMessage::sleep_bar(messenger,20);
//     guard.stop();
//     ShellMessage::send(messenger, format!("{}",style("Done!").green()), MessageType::Close,0);
//     Ok(())
// }
