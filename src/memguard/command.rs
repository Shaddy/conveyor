use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::core::{create_partition,
                  get_partition_option,
                  set_partition_option,
                  delete_partition};


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
            .subcommand(SubCommand::with_name("delete")
                .arg(Arg::with_name("ID")
                        .help("Sets the partition id to delete")
                        .required(true)
                        .value_name("PARTITION_ID")))
            .subcommand(SubCommand::with_name("getinfo")
                .arg(Arg::with_name("ID")
                        .help("Sets the partition id to delete")
                        .required(true)
                        .value_name("PARTITION_ID")
                        .index(1))
                .arg(Arg::with_name("OPT")
                        .help("Specifies the enumerator of the option")
                        .required(true)
                        .value_name("OPTION_NUMBER")
                        .index(2)))
            .subcommand(SubCommand::with_name("setinfo")
                .arg(Arg::with_name("ID")
                        .help("Sets the partition id to get info")
                        .required(true)
                        .value_name("PARTITION_ID")
                        .index(1))
                .arg(Arg::with_name("OPT")
                        .help("Specifies the enumerator of the option")
                        .required(true)
                        .value_name("OPTION_NUMBER")
                        .index(2))
                .arg(Arg::with_name("VAL")
                        .help("Specifies the value of to set")
                        .required(true)
                        .value_name("VALUE")
                        .index(3))))
        .subcommand(SubCommand::with_name("region")
            .subcommand(SubCommand::with_name("create"))
            .subcommand(SubCommand::with_name("delete"))
            .subcommand(SubCommand::with_name("info")))
}

fn delete_partition_command(matches: &ArgMatches, _logger: Logger) {
    let id: u64 = matches.value_of("ID")
                    .expect("Unable to retrieve partition id")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    delete_partition(id);
}

fn create_partition_command(_matches: &ArgMatches, _logger: Logger) {
    match create_partition() {
        Ok(partition) => {
            println!("partition_id: {:?}", partition);
        },
        Err(err) => {

            println!("error: {}", err);
        }
    }
}

fn get_partition_info_command(matches: &ArgMatches, _logger: Logger) {
    let id: u64 = matches.value_of("ID")
                    .expect("Unable to retrieve partition id")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    let option: u64 = matches.value_of("OPT")
                    .expect("Unable to retrieve partition option id")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    get_partition_option(id, option);
}

fn set_partition_info_command(matches: &ArgMatches, _logger: Logger) {
    let id: u64 = matches.value_of("ID")
                    .expect("Unable to retrieve partition id")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    let option: u64 = matches.value_of("OPT")
                    .expect("Unable to retrieve partition option id")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    let value: u64 = matches.value_of("VAL")
                    .expect("Unable to retrieve partition value")
                    .parse::<u64>()
                    .expect("Unable to convert partition id to u64");

    set_partition_option(id, option, value);
}

pub fn partition(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("create", Some(matches))  => create_partition_command(matches, logger),
        ("delete", Some(matches))  => delete_partition_command(matches, logger),
        ("getinfo", Some(matches)) => get_partition_info_command(matches, logger),
        ("setinfo", Some(matches)) => set_partition_info_command(matches, logger),
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