use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::core::Device;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("device").about("tests all device related functionality")
    .subcommand(SubCommand::with_name("example1").about("some command example"))
    .subcommand(SubCommand::with_name("example2").about("some command example"))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("open")  => open_device(logger),
        Some("close") => _not_implemented_command(logger),
        Some("whatever")  => _not_implemented_command(logger),
        _             => println!("{}", matches.usage())
    }
}

pub fn open_device(logger: Logger) {
    Device::new("/devices/memguard").call(11223344);
}