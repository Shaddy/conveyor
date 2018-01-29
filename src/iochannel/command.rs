use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::Device;
use super::failure::Error;


use std::sync::mpsc::{Sender};
use super::cli::output::{MessageType, ShellMessage};
use super::console::style;

fn _not_implemented_command(_messenger: &Sender<ShellMessage>) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("device")
        .about("tests all device related functionality")
        .subcommand(
            SubCommand::with_name("open").arg(
                Arg::with_name("name")
                    .short("n")
                    .required(true)
                    .value_name("DEVICENAME")
                    .help("name of target device"),
            ),
        )
        .subcommand(
            SubCommand::with_name("call").arg(
                Arg::with_name("ctl")
                    .short("c")
                    .required(true)
                    .value_name("IOCTL")
                    .help("specifies any IOCTL code"),
            ),
        )
}

fn device_call(_matches: &ArgMatches, _messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    unimplemented!()
}

pub fn device_open(
    matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let name = matches
        .value_of("name")
        .expect("argument `name` is not present");

    ShellMessage::send(
        messenger,
        format!("Opening device {}...", style(name).underlined().blue()),
        MessageType::Close,
        0,
    );


    let handle = Device::open(name)?;

    ShellMessage::send(
            messenger,
            format!("{} found, handle: {}", style(name).underlined().blue(), style(format!("0x{:x}",  handle as u64)).cyan()  ),
            MessageType::Close,
            1,
        );
    // debug!(logger, "handle: 0x{:x}", handle as u64);

    Ok(())
}

pub fn parse(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("open", Some(matches)) => device_open(matches, messenger),
        ("call", Some(matches)) => device_call(matches, messenger),
        _ => Ok(println!("{}", matches.usage())),
    }
}
