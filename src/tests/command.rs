use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("tests")
        .about("controls testing features")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("stealtoken").about("performs a privilege scalation"))
        .subcommand(SubCommand::with_name("cve-2017-6074").about("exploits DCCP protocol implementation"))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("steamtoken")     => _not_implemented_command(logger),
        Some("cve-2017-6074")  => _not_implemented_command(logger),
        _             => println!("{}", matches.usage())
    }
}