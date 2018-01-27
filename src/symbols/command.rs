use super::parser;
use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::failure::Error;
use super::downloader::PdbDownloader;
use std::sync::mpsc::{Sender, channel};
use super::cli::output::{MessageType, ShellMessage};

pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: &Logger) -> Result<(), Error> {
    unimplemented!()
}

fn _not_implemented_command(_logger: &Logger) -> Result<(), Error> {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    let target = Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .value_name("TARGET")
                        .help("binary file to download PDB")
                        .takes_value(true);

    let struct_arg = Arg::with_name("struct")
                        .short("s")
                        .long("struct")
                        .value_name("STRUCT")
                        .help("struct to find while parsing")
                        .takes_value(true);
    SubCommand::with_name("pdb")
        .about("offers pdb functionality")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("download")
                        .about("downloads the selected PDB")
                        .arg(target.clone()))
        .subcommand(SubCommand::with_name("parse")
                        .about("downloads the selected PDB")
                        .arg(struct_arg.clone())
                        .arg(target.clone()))
        .subcommand(SubCommand::with_name("offset")
                        .about("finds the specified field offset from a struct")
                        .arg(struct_arg.clone())
                        .arg(target.clone()))
        .subcommand(SubCommand::with_name("analyze").about("analyze provided struct/function/class"))
        .subcommand(SubCommand::with_name("dump").about("dumps pdb information into console"))
}

pub fn parse(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("download", Some(matches))  => download_pdb(matches, logger, &tx),
        ("parse", Some(matches))     => parse_pdb(matches, logger),
        ("offset", Some(matches))    => find_offset(matches, logger),
        ("analyze", Some(_))         => _not_implemented_command(logger),
        _                            => Ok(println!("{}", matches.usage()))
    }
}

fn find_offset(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let target = matches.value_of("target").expect("target is not specified");
    let name = matches.value_of("struct").expect("target is not specified");

    debug!(logger, "parsing {} to find {} offset", target, name);
    let _ = parser::find_offset(target, name);
    Ok(())
}

fn parse_pdb(matches: &ArgMatches, logger: &Logger) -> Result<(), Error> {
    let target = matches.value_of("target").expect("target is not specified");
    let name = matches.value_of("struct").expect("target is not specified");

    debug!(logger, "parsing {} searching {}", target, name);
    parser::pdb_to_c_struct(target, name);
    Ok(())
}

fn download_pdb(matches: &ArgMatches, logger: &Logger, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let target = matches.value_of("target").expect("target is not specified");

    let pdb = PdbDownloader::new(target.to_string());

    pdb.download(&tx).expect("Unable to download PDB");

    Ok(())
}
