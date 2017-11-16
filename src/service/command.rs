use super::clap::{App, ArgMatches, SubCommand};
use super::slog::Logger;
use super::process;


fn _not_implemented_command(_logger: Logger) {
    unimplemented!()
}

pub fn parse(matches: &ArgMatches, logger: Logger) {
    let mut services: Vec<&str> = "lynxv memguard sentry".split(" ").collect();

    let action: &Fn(&str, &Logger) = match matches.subcommand_name() {
        Some("install") => { &super::functions::install },
        Some("remove")  => { &super::functions::remove },
        Some("update")  => { &super::functions::update },
        Some("start")   => { &super::functions::start },
        Some("run")     => { &super::functions::run },
        Some("stop")    => { 
            // if an action is a stop, just reverse the order to proper unload services
            services = services.into_iter().rev().collect();
            &super::functions::stop 
        },
        Some("query")   => { &super::functions::query },
        _               => {
            println!("{}", matches.usage());
            process::exit(0);
        }

    };

    services.iter().for_each(|service| {
        action(service, &logger);
    });
}

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("services")
        .about("service controller for lynxvisor and memoryguard")
        .version("0.1")
        .author("Sherab G. <sherab.giovannini@byteheed.com>")
        .subcommand(SubCommand::with_name("install").about("installs lynxv.sys and memguard.sys"))
        .subcommand(SubCommand::with_name("run").about("stops, reinstalls and starts all services"))
        .subcommand(SubCommand::with_name("remove").about("deletes services"))
        .subcommand(SubCommand::with_name("update").about("reinstalls services"))
        .subcommand(SubCommand::with_name("start").about("starts services"))
        .subcommand(SubCommand::with_name("query").about("query services"))
        .subcommand(SubCommand::with_name("stop").about("stops services"))
}