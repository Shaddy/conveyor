// Copyright © ByteHeed.  All rights reserved.
use conveyor::{iochannel, sentry, service, symbols, tests};

extern crate clap;
extern crate conveyor;
extern crate failure;
#[macro_use]
extern crate slog;
extern crate slog_term;
extern crate termcolor;

use failure::Error;
use slog::{Drain, Logger};
// mod service;

use std::process;
use clap::{App, Arg, ArgMatches};
use std::sync::mpsc::{channel, Receiver, Sender};
use conveyor::cli::output::{thread_printer, MessageType, ShellMessage};

fn get_logger(matches: &ArgMatches) -> Logger {
    let _level = match matches.occurrences_of("verbose") {
        0 => slog::Level::Info,
        1 => slog::Level::Debug,
        2 | _ => slog::Level::Trace,
    };

    let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());

    Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!())
}

fn run(app: &ArgMatches, tx: &Sender<ShellMessage>) -> Result<(), Error> {
    let logger = get_logger(app);

    match app.subcommand() {
        ("device", Some(matches)) => iochannel::command::parse(matches, &logger, &tx),
        ("pdb", Some(matches)) => symbols::command::parse(matches, &logger, &tx),
        ("services", Some(matches)) => service::command::parse(matches, &logger, &tx),
        ("tests", Some(matches)) => tests::command::parse(matches, &logger),
        ("sentry", Some(matches)) => sentry::command::parse(matches, &logger),
        _ => Ok(println!("{}", app.usage())),
    }
}

fn main() {
    print!(
        "\n      .;:
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
___________________________________________________________________________\n\n"
    );

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

    let (tx, rx) = channel();
    let (printer_thread, multi_progress) = thread_printer(rx, 20);

    if let Err(e) = run(&matches, &tx) {
        // println!("Application error: {}", e);
        ShellMessage::send(
            &tx,
            format!("Application Error: {}", e),
            MessageType::exit,
            0,
        );

        multi_progress.join();
        printer_thread.join();
        process::exit(1);
    } else {
        ShellMessage::send(
            &tx,
            "".to_owned(),
            MessageType::exit,
            0,
        );

        multi_progress.join();
        printer_thread.join().expect("Something fails.");
        // process::exit(0);
    }
}
