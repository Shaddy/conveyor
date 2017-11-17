use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::core::Device;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("device").about("tests all device related functionality")
    .subcommand(SubCommand::with_name("open")
        .arg(Arg::with_name("name").short("n").required(true).value_name("DEVICENAME").help("name of target device")))
    .subcommand(
        SubCommand::with_name("call")
        .arg(Arg::with_name("ctl").short("c").required(true).value_name("IOCTL").help("specifies any IOCTL code")))
}


fn device_call(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

pub fn device_open(matches: &ArgMatches, _logger: Logger) {
    let name = matches.value_of("name").expect("can't find name flag");
    match Device::open(name) {
        Ok(device) => println!("device: {:?}", device),
        Err(err) => println!("error: {:?}", err),
    }
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("open", Some(matches))  => device_open(matches, logger),
        ("call", Some(matches))  => device_call(matches, logger),
        _             => println!("{}", matches.usage())
    }
}
