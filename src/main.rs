// Copyright © ByteHeed.  All rights reserved.


use conveyor::{service, iochannel};

extern crate conveyor;
extern crate clap;
extern crate termcolor;
extern crate slog;
extern crate slog_term;

use slog::*;

// mod service;

use std::process;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::ptr;

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

fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

fn _not_implemented_subcommand(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        _ => {error!(logger, "not implemented!")},
    }
}


fn services(matches: &ArgMatches, logger: Logger) {
    let mut services: Vec<&str> = "lynxv memguard sentry".split(" ").collect();

    let action: &Fn(&str, &Logger) = match matches.subcommand_name() {
        Some("install") => { &service::install },
        Some("remove")  => { &service::remove },
        Some("update")  => { &service::update },
        Some("start")   => { &service::start },
        Some("run")     => { &service::run },
        Some("stop")    => { &service::stop },
        Some("query")   => { &service::query },
        _               => {
            println!("{}", matches.usage());
            std::process::exit(0);
        }

    };

    // if an action is a stop, just reverse the order to proper unload services
    if ptr::eq(action, &service::stop) {
        services = services.into_iter().rev().collect();
    } 

    services.iter().for_each(|service| {
        action(service, &logger);
    });
}


fn memoryguard_protect(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("tokenguard")    => _not_implemented_command(logger),
        Some("hotpatching")   => _not_implemented_command(logger),
        Some("analyzer")      => _not_implemented_command(logger),
        _                     => println!("{}", matches.usage())
    }
}

fn memoryguard_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("stealtoken", Some(matches))       => _not_implemented_command(logger),
        ("cve-2017-6074", Some(matches))    => _not_implemented_command(logger),
        ("device", Some(matches))           => iochannel::iochannel_tests(matches, logger),
        _                                   => println!("{}", matches.usage())
    }
}

fn test(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("lynxvisor", Some(subcommand))    => _not_implemented_subcommand(subcommand, logger),
        ("memoryguard", Some(subcommand))  => memoryguard_tests(subcommand, logger),
        _                                  => println!("{}", matches.usage())
    }
}

fn run(app: ArgMatches) -> Result<()> {
    let logger = get_logger(&app);

    match app.subcommand() {
        ("services", Some(matches))    => services(matches, logger),
        ("memoryguard", Some(matches)) => memoryguard_protect(matches, logger),
        ("test", Some(matches))        => test(matches, logger),
        _                              => println!("{}", app.usage())
    }

    Ok(())
}

fn main() {
    let matches = App::new("conveyor")
        .about("A gate between humans and dragons.")
        .version("1.0")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .arg(Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"))
        .subcommand(SubCommand::with_name("services")
                            .about("service controller for lynxvisor and memoryguard")
                            .version("0.1")
                            .author("Sherab G. <sherab.giovannini@byteheed.com>")
                            .subcommand(SubCommand::with_name("install").about("installs lynxv.sys and memguard.sys"))
                            .subcommand(SubCommand::with_name("run").about("stops, reinstalls and starts all services"))
                            .subcommand(SubCommand::with_name("remove").about("deletes services"))
                            .subcommand(SubCommand::with_name("update").about("reinstalls services"))
                            .subcommand(SubCommand::with_name("start").about("starts services"))
                            .subcommand(SubCommand::with_name("query").about("query services"))
                            .subcommand(SubCommand::with_name("stop").about("stops services")))
        .subcommand(SubCommand::with_name("tests")
                            .about("controls testing features")
                            .version("0.1")
                            .author("Sherab G. <sherab.giovannini@byteheed.com>")
                            .subcommand(SubCommand::with_name("stealtoken").about("performs a privilege scalation"))
                            .subcommand(SubCommand::with_name("cve-2017-6074").about("exploits DCCP protocol implementation"))
                            .subcommand(conveyor::iochannel::device_subcommand()))
        .subcommand(SubCommand::with_name("memoryguard")
                            .about("enable protection features")
                            .version("0.1")
                            .author("Sherab G. <sherab.giovannini@byteheed.com>")
                            .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
                            .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
                            .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis")))
        .get_matches();

    if let Err(e) = run(matches) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}
