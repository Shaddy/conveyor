use super::clap::Subcommand;
use super::console::style;
use std::{thread, time};

use self::time::Duration;

use super::failure::Error;
use super::iochannel::Device;
use super::sentry::memguard::Partition;
use super::sentry::{io, search};

use super::cli::output::{MessageType, ShellMessage};
use std::sync::mpsc::Sender;

#[derive(Subcommand)]
pub enum TestsCommands {
    NotDefinedYet,
}

// pub fn bind() -> App<'static, 'static> {
//     SubCommand::with_name("tests")
//             .subcommand(super::token::bind())
//             .subcommand(super::kernel::bind())
//             .subcommand(super::process::bind())
//             .subcommand(SubCommand::with_name("search-pattern"))
//             .subcommand(SubCommand::with_name("bars"))
//             .subcommand(super::miscellaneous::bind())
//             .subcommand(SubCommand::with_name("device")
//                 .subcommand(SubCommand::with_name("double-open")))
//             .subcommand(super::mem::bind())
//             .subcommand(super::interceptions::bind())
// 			.subcommand(super::patches::bind())
//             .subcommand(super::errors::bind())
//             .subcommand(super::memguard::bind())
// }
//
// pub fn parse(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     match matches.subcommand() {
//         ("memguard",          Some(matches))  => super::memguard::tests(matches, messenger),
//         ("sentry",            Some(matches))  => super::kernel::tests(matches, messenger),
//         ("process",           Some(matches))  => super::process::tests(matches, messenger),
//         ("memory",            Some(matches))  => super::mem::tests(matches, messenger),
//         ("errors",            Some(matches))  => super::errors::tests(matches, messenger),
//         ("device",            Some(matches))  => device_tests(matches, messenger),
//         ("bars",              Some(matches))  => bar_tests(matches, messenger),
//         ("search-pattern",    Some(matches))  => test_search_pattern(matches, messenger),
//         ("misc",              Some(matches))  => super::miscellaneous::tests(matches, messenger),
//         ("interceptions",     Some(matches))  => super::interceptions::tests(matches, messenger),
//         _                                     => Ok(println!("{}", matches.usage()))
//     }
// }
//
/////////////////////////////////////////////////////////////////////////
//
// BAR TESTS
//
// #[allow(unused_variables)]
// fn bar_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//
//     let bar = ProgressBar::new(5);
//     for _ in 0..5 {
//         thread::sleep(Duration::from_secs(1));
//         bar.inc(1);
//         // ...
//     }
//     bar.finish();
//     let bar = ProgressBar::new(5);
//     bar.set_style(ProgressStyle::default_bar()
//         .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//         .progress_chars("##-"));
//     for _ in 0..5 {
//         thread::sleep(Duration::from_secs(1));
//         bar.inc(1);
//         // ...
//     }
//     bar.finish();
//
//     Ok(())
// }
// #[allow(unused_variables)]
// fn bar_tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     ShellMessage::send(
//         messenger,
//         "Testing spinner".to_string(),
//         MessageType::Spinner,
//         0,
//     );
//     ShellMessage::send(
//         messenger,
//         "Testing spinner".to_string(),
//         MessageType::Spinner,
//         0,
//     );
//     thread::sleep(Duration::from_secs(2));
//     ShellMessage::send(
//         messenger,
//         format!("[*] creating {}.", style("ObjectMonitor").blue()),
//         MessageType::Spinner,
//         0,
//     );
//     thread::sleep(Duration::from_secs(2));
//     ShellMessage::send(
//         messenger,
//         format!("[*] creating {}.", style("ObjectMonitor").blue()),
//         MessageType::Spinner,
//         1,
//     );
//     thread::sleep(Duration::from_secs(1));
//     ShellMessage::send(
//         messenger,
//         format!("[*] protecting {}.", style("ObjectGuard").magenta()),
//         MessageType::Spinner,
//         0,
//     );
//     thread::sleep(Duration::from_secs(2));
//     ShellMessage::send(
//         messenger,
//         format!("[*] destroying {}.", style("ObjectShadow").cyan()),
//         MessageType::Close,
//         0,
//     );
//
//     thread::sleep(Duration::from_secs(2));
//     ShellMessage::send(
//         messenger,
//         format!("[*] protecting {}.", style("ObjectGuard").magenta()),
//         MessageType::Spinner,
//         1,
//     );
//     thread::sleep(Duration::from_secs(2));
//     ShellMessage::send(
//         messenger,
//         format!("[*] destroying {}.", style("ObjectShadow").cyan()),
//         MessageType::Close,
//         1,
//     );
//
//     let bar = ShellMessage::new(
//         messenger,
//         "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}".to_string(),
//         0,
//         10,
//     );
//     let bar1 = ShellMessage::new(
//         messenger,
//         "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}".to_string(),
//         1,
//         10,
//     );
//     for i in 1..10 {
//         thread::sleep(Duration::from_secs(1));
//         bar1.set_progress(messenger, i);
//         bar.set_progress(messenger, i);
//     }
//     bar.complete(messenger);
//     bar1.complete(messenger);
//
//     ShellMessage::sleep_bar(messenger, 5);
//     Ok(())
// }
//
// /////////////////////////////////////////////////////////////////////////
// //
// // DEVICE TESTS
// //
// // fn device_tests(matches: &ArgMatches,  messenger: &Sender<ShellMessage>) -> Result<(), Error> {
// //     match matches.subcommand() {
// //         ("double-open",  Some(matches))  => test_double_open(matches, messenger),
// //         _                                => Ok(println!("{}", matches.usage()))
// //     }
// // }
// //
// fn consume_device(device: Device, messenger: &Sender<ShellMessage>) {
//     // println!("good bye - {:?}", device);
//     ShellMessage::send(
//         messenger,
//         format!("good bye - {:?}", style(device).cyan()),
//         MessageType::Close,
//         0,
//     );
// }
//
// fn test_double_open(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let partition = Partition::root();
//     let device_one = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
//     // debug!(logger, "dropping: device_one");
//     ShellMessage::send(
//         messenger,
//         format!("{}", style("Dropping device_one").yellow()),
//         MessageType::Spinner,
//         0,
//     );
//     consume_device(device_one, messenger);
//     // debug!(logger, "device_one dropped");
//     ShellMessage::send(
//         messenger,
//         format!("{} dropped", style("device_one").cyan()),
//         MessageType::Spinner,
//         0,
//     );
//     // debug!(logger, "creating a partition");
//     ShellMessage::send(
//         messenger,
//         format!("{}", style("Creating partition").blue()),
//         MessageType::Spinner,
//         0,
//     );
//
//     if io::delete_partition(&partition.device, partition.id).is_err() {
//         ShellMessage::send(
//             messenger,
//             format!("{}", style("Test ended USUCCESSFULLY").red()),
//             MessageType::Close,
//             0,
//         );
//         // colorize::failed("TEST HAS FAILED");
//     } else {
//         ShellMessage::send(
//             messenger,
//             format!("{}", style("Test ended SUCCESSFULLY").green()),
//             MessageType::Close,
//             0,
//         );
//         // colorize::success("TEST IS SUCCESS");
//     }
//     // ShellMessage::send(messenger, "Test ended".to_string(), MessageType::Close, 0);
//
//     Ok(())
// }
//
// /////////////////////////////////////////////////////////////////////////
// //
// // SEARCH PATTERN TEST
// //
//
// fn test_search_pattern(
//     _matches: &ArgMatches,
//     messenger: &Sender<ShellMessage>,
// ) -> Result<(), Error> {
//     let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
//
//     let switch_context_pattern: Vec<u8> = vec![
//         0x89, 0x60, 0x18, 0x4C, 0x89, 0x68, 0x20, 0x4C, 0x89, 0x70, 0x28, 0x4C, 0x89, 0x78, 0x30,
//         0x65, 0x48, 0x8B, 0x1C, 0x25, 0x20, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xF9,
//     ];
//
//     if let Some(offset) = search::pattern(
//         &device,
//         "ntos",
//         &switch_context_pattern,
//         Some("KeSynchronizeExecution"),
//     ) {
//         ShellMessage::send(
//             messenger,
//             format!(
//                 "Switch-content: {}",
//                 style(format!("0x{:016x}", offset)).cyan()
//             ),
//             MessageType::Close,
//             0,
//         );
//     }
//
//     Ok(())
// }
