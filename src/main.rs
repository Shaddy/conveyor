// Copyright © ByteHeed.  All rights reserved.
use conveyor::{iochannel, sentry, service, symbols, tests};

extern crate clap;
extern crate conveyor;
extern crate failure;
extern crate slog_term;
extern crate termcolor;

use failure::Error;

use std::process;
use clap::{App, Arg, ArgMatches};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender};
use conveyor::cli::output::{thread_printer, MessageType, ShellMessage};

// fn get_logger(matches: &ArgMatches) -> Logger {
//     let _level = match matches.occurrences_of("verbose") {
//         0 => slog::Level::Info,
//         1 => slog::Level::Debug,
//         2 | _ => slog::Level::Trace,
//     };
//
//     let plain = slog_term::PlainSyncDecorator::new(std::io::stdout());
//
//     Logger::root(slog_term::FullFormat::new(plain).build().fuse(), o!())
// }

fn run(app: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match app.subcommand() {
        ("device", Some(matches)) => iochannel::command::parse(matches, &messenger),
        ("pdb", Some(matches)) => symbols::command::parse(matches, &messenger),
        ("services", Some(matches)) => service::command::parse(matches, &messenger),
        ("tests", Some(matches)) => tests::command::parse(matches, &messenger),
        ("sentry", Some(matches)) => sentry::command::parse(matches, &messenger),
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

    let (messenger, rx) = channel();
    let (printer, writer) = thread_printer(rx, 20);

    if let Err(e) = run(&matches, &messenger) {
        ShellMessage::send(
            &messenger,
            format!("Application Error: {}", e),
            MessageType::Exit,
            0,
        );

        writer.join().expect("Unable to wait for writer");
        printer.join().expect("Unable to wait for printer");
        process::exit(1);
    }

    ShellMessage::send(
        &messenger,
        "".to_owned(),
        MessageType::Exit,
        0,
    );

    writer.join().expect("Unable to wait for writer");
    printer.join().expect("Unable to wait for printer");
}
