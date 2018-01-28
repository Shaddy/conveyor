// Copyright © ByteHeed.  All rights reserved.

use super::clap::{App, ArgMatches, SubCommand};
use super::cli::colorize;

use std::{thread, mem, fmt};
use std::time::Duration;

use super::failure::Error;
use super::common;
use super::iochannel::Device;
use super::sentry::{memory, search, io};
use super::sentry::memguard::{Response,
                              Interception,
                              Partition,
                              Region,
                              Guard,
                              Access,
                              Action,
                              Filter,
                              MatchType};

use super::ssdt::SSDT_FUNCTIONS;

use std::sync::mpsc::Sender;
use super::cli::output::{ShellMessage, MessageType};

pub fn bind() -> App<'static, 'static> {
    SubCommand::with_name("interceptions")
                .subcommand(SubCommand::with_name("kernel"))
                .subcommand(SubCommand::with_name("stealth"))
                .subcommand(SubCommand::with_name("analysis-normal"))
                .subcommand(SubCommand::with_name("analysis-no-message"))
                .subcommand(SubCommand::with_name("callback"))
                .subcommand(SubCommand::with_name("ssdt"))
}

pub fn tests(matches: &ArgMatches,  messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match matches.subcommand() {
        ("kernel",               Some(matches))  => test_intercept_kernel_region(matches, messenger),
        ("stealth",              Some(matches))  => test_stealth_interception(matches, messenger),
        ("analysis-normal",      Some(matches))  => test_analysis_normal(matches, messenger),
        ("analysis-no-message",  Some(matches))  => test_analysis_no_message(matches, messenger),
        ("callback",             Some(matches))  => test_interception_callback(matches, messenger),
        ("ssdt",                 Some(matches))  => test_ssdt_address(matches, messenger),
        _                                 => Ok(println!("{}", matches.usage()))
    }
}

fn test_ssdt_address(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    ShellMessage::send(messenger, format!("{:?}", find_ssdt_address()), MessageType::Spinner, 0);
    Ok(())
}

#[repr(C)]
struct ServiceTable {
    pub address: u64,
    pub tables: u64,
    pub count: u32,
    pub arguments: u64
}

impl fmt::Debug for ServiceTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ServiceTable {{
                    address:   0x{:016x},
                    tables:    0x{:016x},
                    count:            {},
                    arguments: 0x{:016x},
        }}", self.address,
             self.tables,
             self.count,
             self.arguments)
    }
}

fn find_ssdt_address() -> ServiceTable {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("sentry device");
    let pattern = vec![0x48, 0x89, 0xA3, 0xD8,
                       0x01, 0x00, 0x00, 0x8B,
                       0xF8, 0xC1, 0xEF, 0x07,
                       0x83, 0xE7, 0x20, 0x25];

    let address = search::pattern(&device,
                                  "ntoskrnl",
                                  &pattern,
                                  Some("ZwCreateResourceManager"))
                                  .expect("unable to find SSDT pattern");

    let instruction = pattern.len() as u64 + 7;

    let rva = memory::read_u32(&device, address + instruction).unwrap() as i32;

    let ssdt_reference = address.wrapping_add(rva as u64) + instruction + 4;

    println!("ref: 0x{:016x}", ssdt_reference);
    let data = memory::read_virtual_memory(&device, ssdt_reference, mem::size_of::<ServiceTable>())
                    .expect("error reading ServiceTable");

    let ssdt: ServiceTable = unsafe { mem::transmute_copy(&*data.as_ptr()) };

    ssdt
}

enum Print {
    Show,
    Hide
}

fn test_analysis_no_message(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    test_analysis_interception_messaged(Print::Hide, messenger)
}

fn test_analysis_normal(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    test_analysis_interception_messaged(Print::Show, messenger)
}


fn test_analysis_interception_messaged(show: Print, messenger: &Sender<ShellMessage>) -> Result<(), Error> {

    ShellMessage::send(messenger,"discovering SSDT".to_string(), MessageType::Spinner, 0);
    let ssdt = find_ssdt_address();

    ShellMessage::send(messenger,format!("{:?}", ssdt), MessageType::Spinner, 0);

    let address = ssdt.address;

    ShellMessage::send(messenger,format!("found at 0x{:16x}", address), MessageType::Spinner, 0);

    let partition = Partition::root();

    let filter = Filter::process(&partition.device, "notepad", MatchType::EQUAL)
                            .expect("can't find notepad process");

    let mut guard = Guard::new(&partition, Some(filter));

    let region = Region::new(&partition, address,
                              ssdt.count as u64 * 4,
                              Some(Action::NOTIFY | Action::INSPECT),
                              Access::READ)
                            .expect("can't create region");

    ShellMessage::send(messenger,format!("adding {} to {}", region, guard), MessageType::Spinner, 0);
    guard.add(region);

    guard.set_callback(Box::new(move |interception| {
        let index = interception.address.wrapping_sub(address) / 4;
        let name = SSDT_FUNCTIONS.get(index as usize).unwrap_or(&"<invalid-index>");

        let message = format!("{}", name);
        match show {
            Print::Show => Response::new(Some(message), Action::CONTINUE),
            Print::Hide => Response::new(None, Action::CONTINUE),
        }
    }));

    ShellMessage::send(messenger, "starting guard".to_string(), MessageType::Spinner, 0);
    guard.start();

    let duration = Duration::from_secs(60);
    ShellMessage::send(messenger, format!("waiting {:?}", duration), MessageType::Spinner, 0);
    thread::sleep(duration);

    ShellMessage::send(messenger, "stoping guard".to_string(), MessageType::Spinner, 0);
    guard.stop();
    Ok(())
}

//
// This test aims to demostrate that we are able to ignore any write to any memory address
//
fn test_stealth_interception(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {

    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x10;

    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();
    ShellMessage::send(messenger,format!("new pool: 0x{:016x} ({} bytes)", addr, POOL_SIZE),MessageType::Spinner,0);

    let bytes = memory::write_virtual_memory(&partition.device, addr, vec![0; POOL_SIZE]).unwrap();
    ShellMessage::send(messenger,format!("zeroed {} bytes", bytes),MessageType::Spinner,0);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    let output = common::dump_vector(&v);
    ShellMessage::send(messenger,format!("dumping buffer 0x{:016x} \n{}", addr, output),MessageType::Spinner,0);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, Some(Action::NOTIFY | Action::INSPECT), Access::WRITE).unwrap();

    ShellMessage::send(messenger,format!("adding {} to {}", region, guard),MessageType::Spinner,0);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("Attempt to write at 0x{:016X} - IGNORING", interception.address);
        // TODO: set optional color for response
        // colorize::info(&message);
        Response::new(Some(message), Action::STEALTH)
    }));

    ShellMessage::send(messenger,"starting guard".to_string(),MessageType::Spinner,0);
    guard.start();
    ShellMessage::send(messenger,format!("accessing memory 0x{:016x}", addr),MessageType::Spinner,0);

    let v = common::dummy_vector(POOL_SIZE);

    let bytes = memory::write_virtual_memory(&partition.device, addr, v).unwrap();
    ShellMessage::send(messenger,format!("{} bytes written", bytes),MessageType::Spinner,0);

    let v = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    if v.iter().any(|&b| b != 0x00) {
        colorize::failed("STEALTH test result has FAILED.");
        let output = common::dump_vector(&v);
        ShellMessage::send(messenger, format!("inspecting buffer 0x{:016x}", addr), MessageType::Spinner, 0);
        colorize::warning(&output);
    } else {
        colorize::success("STEALTH test result has PASSED.");
    }

    ShellMessage::send(messenger, "stoping guard".to_string(), MessageType::Spinner, 0);
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();
    Ok(())
}

// example of declared function as callback
#[allow(dead_code)]
fn callback_test(interception: Interception) -> Action {
    println!("The offensive address is 0x{:016X} (accessing {:?})", interception.address,
                                    interception.access);
    // ShellMessage::send(messenger,format!("The offensive address is 0x{:016X} (accessing {:?})", interception.address,
    //                                 interception.access),MessageType::Spinner,0);
    //
    Action::CONTINUE
}

fn test_interception_callback(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    ShellMessage::send(messenger, "Allocating pool".to_string(), MessageType::Spinner, 0);
    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();

    ShellMessage::send(messenger, format!("Addr: 0x{:016x}", addr), MessageType::Spinner, 0);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    ShellMessage::send(messenger,
            format!("Adding {} to {}", region, guard), MessageType::Spinner, 0);
    guard.add(region);

    guard.set_callback(Box::new(|interception| {
        let message = format!("The offensive address is 0x{:016X} (accessing {:?})", interception.address,
                                        interception.access);

        Response::new(Some(message), Action::CONTINUE)
    }));

    ShellMessage::send(messenger, "Starting guard".to_string(), MessageType::Spinner, 0);
    guard.start();

    ShellMessage::send(messenger,
        format!("Accessing memory 0x{:016x}", addr),MessageType::Spinner, 0);

    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    ShellMessage::send(messenger,"Stoping guard".to_string(),MessageType::Spinner, 0);
    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();
    Ok(())
}

fn test_intercept_kernel_region(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let partition: Partition = Partition::root();
    let mut guard = Guard::new(&partition, None);

    const POOL_SIZE: usize = 0x100;

    ShellMessage::send(messenger,"Allocating pool...".to_string(),MessageType::Spinner,0);

    let addr = memory::alloc_virtual_memory(&partition.device, POOL_SIZE).unwrap();
    ShellMessage::send(messenger,format!("Addr: 0x{:016x}",addr),MessageType::Spinner,0);

    let region = Region::new(&partition, addr, POOL_SIZE as u64, None, Access::READ).unwrap();

    ShellMessage::send(messenger,format!("Adding {} to {}",region , guard),MessageType::Spinner,0);

    guard.add(region);
    ShellMessage::send(messenger,"Starting guard...".to_string(),MessageType::Spinner,0);

    guard.start();
    ShellMessage::send(messenger,format!("Accesing memory 0x{:016x}",addr),MessageType::Spinner,0);

    let _ = memory::read_virtual_memory(&partition.device, addr, POOL_SIZE).unwrap();
    ShellMessage::send(messenger,"Stoping guard".to_string(),MessageType::Spinner,0);

    guard.stop();

    memory::free_virtual_memory(&partition.device, addr).unwrap();
    ShellMessage::send(messenger,"Done!".to_string(),MessageType::Close,0);

    Ok(())
}

// fn test_intercept_region(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let mut v: Vec<u8> = Vec::new();
//     v.push(13);

//     let partition: Partition = Partition::root();
//     let mut guard = Guard::new(&partition, None);
//     let region = Region::new(&partition, v.as_ptr() as u64, 10, None, Access::READ);
//     debug!(logger, "adding {} to {}", region, guard);
//     guard.add(region);
//     debug!(logger, "starting guard, and sleeping 5 seconds");
//     guard.start();
//     thread::sleep(Duration::from_secs(5));

//     // accessing memory
//     debug!(logger, "accessing memory {:?} 5 times", v.as_ptr());
//     let _ = v[0];
//     let _ = v[0];
//     let _ = v[0];
//     let _ = v[0];
//     let value = v[0];

//     debug!(logger, "value: {}", value);
//     debug!(logger, "sleeping 5 secs");
//     thread::sleep(Duration::from_secs(5));
//     debug!(logger, "stoping guard");
//     guard.stop();
// }
