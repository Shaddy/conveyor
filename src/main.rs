// Copyright © ByteHeed.  All rights reserved.
use conveyor::{service, iochannel, sentry, tests, symbols};

extern crate failure;
extern crate conveyor;
extern crate clap;
extern crate termcolor;
#[macro_use]
extern crate slog;
extern crate slog_term;

use failure::Error;
use slog::{Logger, Drain};
// mod service;

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

fn run(app: &ArgMatches) -> Result<(), Error> {
    let logger = get_logger(app);

    match app.subcommand() {
        ("device",   Some(matches)) => iochannel::command::parse(matches, &logger),
        ("pdb",      Some(matches)) => symbols::command::parse(matches, &logger),
        ("services", Some(matches)) => service::command::parse(matches, &logger),
        ("tests",    Some(matches)) => tests::command::parse(matches, &logger),
        ("sentry",   Some(matches)) => sentry::command::parse(matches, &logger),
        _                           => Ok(println!("{}", app.usage()))
    }
}

fn main() {

    print!("\n      .;:
     ::::
     ; ;:
       ;:
   ::: ;::::::::  :##### ##  ## ###### #####  ;,   #  ####@ ##### ;####
   ::  ;:     ::  :#   #  #  #    ##   ##     ;,   #  #     #     ;.   #
   ::  ;:     ::  :#   #  ####    ##   ##     ;.   #  #     #     ;.   #
   ::  ;:     ::  :#####  `##`    ##   #####  ;#####  ####; ####@ ;.   #
   ::  ;:     ::  :#   #   ##     ##   ##     ;,   #  #     #     ;.   #
   ::         ::  :#   #   ##     ##   ##     ;,   #  #     #     ;.   #
   ::         ::  :#####   ##     ##   #####  ;,   #  ##### ##### ;####
   ::;;;;;;;;;::                        :::::::::::::::::::::::::::::::::::
                                        :: www.byteheed.com/memoryguard  ::
                                        :::::::::::::::::::::::::::::::::::

Sherab G. <sherab.giovannini@byteheed.com>
A gate between humans and dragons.
___________________________________________________________________________\n\n");



    // print!("{:?}", &head_message);

    let matches = App::new("conveyor")
        .about("A gate between humans and dragons.")
        .version("1.0")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .arg(Arg::with_name("v") .short("v") .multiple(true) .help("Sets the level of verbosity"))
        .subcommand(conveyor::service::command::bind())
        .subcommand(conveyor::iochannel::command::bind())
        .subcommand(conveyor::tests::command::bind())
        // .subcommand(conveyor::sentry::command::bind())
        .subcommand(conveyor::symbols::command::bind())
        .get_matches();

    if let Err(e) = run(&matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
