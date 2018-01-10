use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;

// partition functions
// use super::core::{create_partition,
//                   get_partition_option,
//                   set_partition_option,
//                   delete_partition};


pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("sentry")
        .about("enable protection features")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("features")
            .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
            .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
            .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis")))
}



pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("features", Some(_))        => _not_implemented_command(logger),
        ("region", Some(_))          => _not_implemented_command(logger),
        _                            => println!("{}", matches.usage())
    }
}