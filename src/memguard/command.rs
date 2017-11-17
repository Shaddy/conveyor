use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memguard")
        .about("enable protection features")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
        .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
        .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis"))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("tokenguard")    => _not_implemented_command(logger),
        Some("hotpatching")   => _not_implemented_command(logger),
        Some("analyzer")      => _not_implemented_command(logger),
        _                     => println!("{}", matches.usage())
    }
}