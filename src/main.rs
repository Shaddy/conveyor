extern crate clap;
extern crate termcolor;
extern crate conveyor;
extern crate slog;
extern crate slog_term;

use slog::*;

use std::process;
use clap::{App, Arg, ArgMatches, SubCommand};
use conveyor::WindowsService;

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

fn install_service(name: &str, logger: &Logger) {
    debug!(logger, "installing {}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).install();
}

fn remove_service(name: &str, logger: &Logger) {
    debug!(logger, "removing {}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).remove();
}

fn update_service(name: &str, logger: &Logger) {
    debug!(logger, "updating {}", name);

    let mut path = std::env::current_dir().expect("error getting current dir");
    path.push(format!("{}.sys", name));

    WindowsService::new(name, path.to_str().unwrap()).remove();
    WindowsService::new(name, path.to_str().unwrap()).install();
}

fn for_each_service(logger: Logger, service_action: &Fn(&str, &Logger)) {
    service_action("lynxv", &logger);
    service_action("memguard", &logger);
}

fn services(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("install") => { for_each_service(logger, &install_service) },
        Some("remove") => { for_each_service(logger, &remove_service) },
        Some("update") => { for_each_service(logger, &update_service) },
        _               => println!("{}", matches.usage())
    }
}


fn memoryguard_protect(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("tokenguard")          => _not_implemented_command(logger),
        Some("hotpatching")         => _not_implemented_command(logger),
        Some("analyzer")         => _not_implemented_command(logger),
        _                           => println!("{}", matches.usage())
    }
}

fn memoryguard_tests(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand_name() {
        Some("stealtoken")          => _not_implemented_command(logger),
        Some("cve-2017-6074")       => _not_implemented_command(logger),
        _                           => println!("{}", matches.usage())
    }
}

fn test(matches: &ArgMatches, logger: Logger) {
    match matches.subcommand() {
        ("lynxvisor", Some(subcommand))       => _not_implemented_subcommand(subcommand, logger),
        ("memoryguard", Some(subcommand))     => memoryguard_tests(subcommand, logger),
        _                                     => println!("{}", matches.usage())
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
                            .subcommand(SubCommand::with_name("remove").about("removes lynxv.sys and memguard.sys"))
                            .subcommand(SubCommand::with_name("update").about("updates lynxv.sys and memguard.sys")))
        .subcommand(SubCommand::with_name("tests")
                            .about("controls testing features")
                            .version("0.1")
                            .author("Sherab G. <sherab.giovannini@byteheed.com>")
                            .subcommand(SubCommand::with_name("stealtoken").about("performs a privilege scalation"))
                            .subcommand(SubCommand::with_name("cve-2017-6074").about("exploits DCCP protocol implementation")))
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
