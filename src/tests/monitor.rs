use super::clap::{App, ArgMatches, SubCommand};

use std::{thread, time};
use self::time::Duration;

use super::console::{style};
use super::console::Term;
use std::sync::mpsc::Sender;
use super::failure::Error;
use super::cli::output::{ShellMessage, MessageType};
use super::sentry::memguard::{ ObjectFilter };

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("monitor")
                .subcommand(SubCommand::with_name("obfilter"))
}


pub fn parse(matches: &ArgMatches,  messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("obfilter",     Some(matches))  => monitor_tests(matches, messenger),
        _                                => Ok(println!("{}", matches.usage()))
    }
}


/////////////////////////////////////////////////////////////////////////
//
// MONITOR TESTS
//
#[allow(unused_variables)]
pub fn monitor_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {

    let term = Term::stdout();

    let filter = ObjectFilter::new()
                .expect("can't create object filter");

    ShellMessage::send(messenger, format!("[!] {}.", style("starting").magenta()), MessageType::Spinner,0);
    filter.start().expect("unable to start filter");

    thread::sleep(time::Duration::from_secs(20));

    ShellMessage::send(messenger, format!("[!] {}.", style("stopping").magenta()), MessageType::Close,0);
    filter.stop().expect("unable to start filter");
    Ok(())
}
