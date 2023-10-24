// Copyright © ByteHeed.  All rights reserved.
use conveyor::{iochannel, sentry, service, symbols, tests};

extern crate conveyor;
extern crate failure;
extern crate slog_term;
extern crate termcolor;

use failure::Error;

use clap::Parser;
use conveyor::cli::commands::CliCommands;
use conveyor::cli::output::{create_messenger, MessageType, ShellMessage};
use std::process;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;

fn run(commands: CliCommands, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match commands {
        CliCommands::Pdb { command } => symbols::command::parse(&command, messenger),
        // CliCommands::Tests(commands) => tests::command::parse(&commands, messenger),
        // CliCommands::Sentry(commands) => sentry::command::parse(&commands, messenger),
        // CliCommands::Service(commands) => service::command::parse(&commands, messenger),
        CliCommands::Load { target } => {
            service::install(&target, messenger);
            service::start(&target, messenger);
            Ok(())
        }
        CliCommands::Unload { target } => {
            service::stop(&target, messenger);
            service::remove(&target, messenger);

            Ok(())
        }

        // CliCommands::IoChannel(commands) => iochannel::command::parse(&commands, messenger),
        _ => unimplemented!("Command not implemented"),
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
   ::;;;;;;;;;::

                                        :::::::::::::::::::::::::::::::::::
                                        :: www.byteheed.com/memoryguard  ::
                                        :::::::::::::::::::::::::::::::::::

Sherab G. <sherab.giovannini@byteheed.com>
A gate between humans and dragons.
___________________________________________________________________________\n\n"
    );

    let commands = conveyor::cli::commands::Cli::parse();

    let (messenger, receiver) = channel();
    let printer = create_messenger(receiver, None, 20);

    if let Err(e) = run(commands.command, &messenger) {
        ShellMessage::send(
            &messenger,
            format!("Application Error: {}", e),
            MessageType::Exit,
            0,
        );

        printer.join().expect("Unable to wait for printer");
        process::exit(1);
    }

    ShellMessage::send(&messenger, "".to_owned(), MessageType::Exit, 0);
    printer.join().expect("Unable to wait for printer");
}
