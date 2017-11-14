extern crate clap;
extern crate termcolor;
extern crate conveyor;
extern crate slog;
extern crate slog_term;

use slog::*;

use std::process;
use clap::{App, Arg, ArgMatches};

    
fn get_logger(matches: &ArgMatches) -> Logger {
    let _level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };

    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());

    Logger::root(
        slog_term::FullFormat::new(plain)
        .build().fuse(), o!()
    )
}

fn run(matches: ArgMatches) -> Result<String> {
    let logger = get_logger(&matches);
    info!(logger, "Running application");
    println!("{:?}", matches.occurrences_of("install"));

    Ok(String::from("asdfasdasdf"))
}

fn main() {
    let matches = App::new("Convoyer")
        .version("0.1.0")
        .author("ByteHeed <dev@byteheed.com>")
        .about("Sentry application")
        .arg(Arg::with_name("Info|Debug|Error")
                .short("v")
                .long("verbose")
                .takes_value(true)
                .help("Logging level to display")
        )
        .arg(Arg::with_name("test")
                .short("t")
                .long("test")
                .takes_value(true)
                .help("Start test for drivers and services")
        )
        .arg(Arg::with_name("install")
                .short("i")
                .long("install")
                .takes_value(false)
                .help("Install services and drivers."),
        )
        .arg(Arg::with_name("remove")
                .short("r")
                .long("remove")
                .takes_value(false)
                .help("Delete installed services and drivers."),
        )
        .arg(Arg::with_name("update")
                .short("u")
                .long("update")
                .takes_value(false)
                .help("Update current services and drivers.")
        )
        .get_matches();

    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
