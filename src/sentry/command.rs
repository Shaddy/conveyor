use super::clap::Subcommand;
use super::failure::Error;

use super::cli::output::ShellMessage;
use std::sync::mpsc::Sender;

#[derive(Subcommand)]
pub enum SentryCommands {
    Features,
    Region,
}

// partition functions
// use super::core::{create_partition,
//                   get_partition_option,
//                   set_partition_option,
//                   delete_partition};

// pub fn bind() -> App<'static, 'static> {
//     SubCommand::with_name("sentry")
//         .about("enable protection features")
//         .version("0.1")
//         .author("Sherab G. <sherab.giovannini@byteheed.com>")
//         .subcommand(SubCommand::with_name("features")
//             .subcommand(SubCommand::with_name("tokenguard").about("activates privilege elevation protection tests"))
//             .subcommand(SubCommand::with_name("hotpatching").about("starts exploits patches protection"))
//             .subcommand(SubCommand::with_name("analyzer").about("activates kernel analysis")))
// }
//
//
//
// pub fn parse(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     match matches.subcommand() {
//         ("features", Some(_)) | ("region", Some(_))  => _not_implemented_command(messenger),
//         _                                            => println!("{}", matches.usage())
//     }
//
//     Ok(())
// }
