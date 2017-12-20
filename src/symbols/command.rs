use super::clap::{App, Arg, ArgMatches, SubCommand};
use super::slog::Logger;
use super::downloader::PdbDownloader;

pub fn _not_implemented_subcommand(_matches: &ArgMatches, _logger: Logger) {
    unimplemented!()
}

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("pdb")
        .about("offers pdb functionality")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("download")
                        .about("downloads the selected PDB")
                        .arg(Arg::with_name("target")
                        .short("t")
                        .long("target")
                        .value_name("TARGET")
                        .help("binary file to download PDB")
                        .takes_value(true)))
        .subcommand(SubCommand::with_name("analyze").about("analyze provided struct/function/class"))
        .subcommand(SubCommand::with_name("dump").about("dumps pdb information into console"))
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("download", Some(matches))  => download_pdb(matches, &logger),
        ("analyze", Some(_))         => _not_implemented_command(logger),
        _                            => println!("{}", matches.usage())
    }
}

fn download_pdb(matches: &ArgMatches, _logger: &Logger) {
    let target = matches.value_of("target").expect("target is not specified");
    let pdb = PdbDownloader::new(target.to_string());

    pdb.download().expect("Unable to download PDB");
}