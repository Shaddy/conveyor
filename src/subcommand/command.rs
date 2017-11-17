use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("subcommand_name").about("subcommand description")
    .subcommand(SubCommand::with_name("children_command_example1").about("description"))
    .subcommand(SubCommand::with_name("children_command_example2").about("description"))
    arg(Arg::with_name("INPUT")
        .help("Sets the input file to use")
        .required(true)
        .index(1))
    .arg(Arg::with_name("flag_name")
    .short("f")
    .multiple(true) // could be -f or -ffff
    .help("description of this flag"))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("command1")  => _not_implemented_command(logger),
        Some("command2")  => _not_implemented_command(logger),
        Some("command3")  => _not_implemented_command(logger),
        _             => println!("{}", matches.usage())
    }
}