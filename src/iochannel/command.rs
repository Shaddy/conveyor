use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::Device;
use super::failure::Error;


use std::sync::mpsc::{Sender};
use super::cli::output::{MessageType, ShellMessage};

fn _not_implemented_command(_logger: &Logger) {
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

fn device_call(_matches: &ArgMatches,tx: &Sender<ShellMessage>) -> Result<(), Error> {
    unimplemented!()
}

pub fn device_open(
    matches: &ArgMatches,
    // logger: &Logger,
    tx: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let name = matches
        .value_of("name")
        .expect("argument `name` is not present");

    ShellMessage::send(
        &tx,
        format!("Opening device {}...", name),
        MessageType::close,
        0,
    );


    let handle = Device::open(name)?;

    ShellMessage::send(
            &tx,
            format!("{} found, handle: 0x{:x}",name, handle as u64),
            MessageType::close,
            1,
        );
    // debug!(logger, "handle: 0x{:x}", handle as u64);

    Ok(())
}

pub fn parse(
    matches: &ArgMatches,
    logger: &Logger,
    tx: &Sender<ShellMessage>,
) -> Result<(), Error> {
    match matches.subcommand() {
        ("open", Some(matches)) => device_open(matches, &tx),
        ("call", Some(matches)) => device_call(matches, &tx),
        _ => Ok(println!("{}", matches.usage())),
    }
}
