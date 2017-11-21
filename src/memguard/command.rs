use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::core::create_partition;


pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("memguard")
        .about("enable protection features")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("features")
            .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
            .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
            .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis")))
        .subcommand(SubCommand::with_name("partition")
            .subcommand(SubCommand::with_name("create"))
            .subcommand(SubCommand::with_name("delete"))
            .subcommand(SubCommand::with_name("info")))
        .subcommand(SubCommand::with_name("region")
            .subcommand(SubCommand::with_name("create"))
            .subcommand(SubCommand::with_name("delete"))
            .subcommand(SubCommand::with_name("info")))
}

fn delete_partition(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn create_partition_command(_matches: &ArgMatches, _logger: Logger) {
    match create_partition() {
        Ok(id) => {
            println!("partition_id: {}", id);
        },
        Err(err) => {

            println!("error: {}", err);
        }
    }
}


pub fn partition(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create", Some(matches))  => create_partition_command(matches, logger),
        ("delete", Some(matches))  => delete_partition(matches, logger),
        ("info", Some(matches))    => _not_implemented_subcommand(matches, logger),
        _                            => println!("{}", matches.usage())
    }
}
pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("features", Some(_))        => _not_implemented_command(logger),
        ("partition", Some(matches)) => partition(matches, logger),
        ("region", Some(_))          => _not_implemented_command(logger),
        _                            => println!("{}", matches.usage())
    }
}