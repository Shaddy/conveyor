// Copyright © ByteHeed.  All rights reserved.

use super::clap::{ArgMatches, Subcommand};

use super::common;
use super::failure::Error;
use super::iochannel::Device;
use super::sentry::memguard::Filter;
use super::sentry::memory::{Map, MapMode};
use super::sentry::{io, memory};
use std::fmt;

use super::cli::output::{MessageType, ShellMessage};
use super::console::style;
use std::sync::mpsc::Sender;

// pub fn bind() -> App<'static, 'static> {
//     SubCommand::with_name("memory")
//                 .subcommand(SubCommand::with_name("read"))
//                 .subcommand(SubCommand::with_name("fuzz-kernel-map-1"))
//                 .subcommand(SubCommand::with_name("virtual"))
//                 .subcommand(SubCommand::with_name("write"))
//                 .subcommand(SubCommand::with_name("kernel-map"))
//                 .subcommand(SubCommand::with_name("map"))
// }
//
// pub fn tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     match matches.subcommand() {
//         ("fuzz-kernel-map-1",      Some(matches))  => test_fuzz_memory(matches, messenger),
//         ("virtual",                Some(matches))  => test_virtual_memory(matches, messenger),
//         ("write",                  Some(matches))  => test_memory_write(matches, messenger),
//         ("map",                    Some(matches))  => test_memory_map(matches, messenger),
//         ("kernel-map",             Some(matches))  => test_kernel_map(matches, messenger),
//         _                                => Ok(println!("{}", matches.usage()))
//     }
// }
//
#[allow(unused_variables)]
fn test_fuzz_memory(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    let _filters: Vec<Filter> = (0..1000).map(|_| Filter::new(&device)).collect();
    // format!("{}", style("Done!").green());

    ShellMessage::send(
        messenger,
        format!("{}", style("Done!").green()),
        MessageType::Close,
        0,
    );
    Ok(())
}

fn test_kernel_map(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    struct TestStruct {
        first: u64,
        second: u64,
    }

    impl fmt::Display for TestStruct {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(
                f,
                "TestStruct {{ first: 0x{:016x}, second: 0x{:016x} }}",
                self.first, self.second
            )
        }
    }

    let map = memory::KernelAlloc::<TestStruct>::new(&device);

    ShellMessage::send(
        messenger,
        format!(
            "TestStruct: allocated {} bytes at:",
            format!("{}", style(map.size()).underlined().cyan())
        ),
        MessageType::Close,
        0,
    );
    ShellMessage::send(
        messenger,
        format!(
            "\t\tkernel: {}",
            style(format!("0x{:016x}", map.kernel_ptr())).yellow()
        ),
        MessageType::Close,
        0,
    );
    ShellMessage::send(
        messenger,
        format!(
            "\t\tuser:   {}",
            style(format!("0x{:016x}", map.as_ptr() as u64)).green()
        ),
        MessageType::Close,
        0,
    );

    unsafe {
        let test = &mut *map.as_mut_ptr();
        test.first = 0x1122_3344;
        test.second = 0x5566_7788;
    }
    // debug!(logger, "reading kernel pointer 0x{:016x}", map.kernel_ptr());
    ShellMessage::send(
        messenger,
        format!(
            "reading kernel pointer: {}",
            style(format!("0x{:016x}", map.kernel_ptr())).yellow()
        ),
        MessageType::Spinner,
        1,
    );

    let v = memory::read_virtual_memory(&device, map.kernel_ptr(), map.size())
        .expect("error reading memory");

    let u: &TestStruct = unsafe { &*map.as_ptr() };
    let k: &TestStruct = unsafe { &*(v.as_ptr() as *const TestStruct) };

    // debug!(logger, "from-user: {}", u);
    ShellMessage::send(
        messenger,
        format!("from-user: {}", style(u).green()),
        MessageType::Close,
        2,
    );
    // debug!(logger, "from-kernel: {}", k);
    ShellMessage::send(
        messenger,
        format!("from-kernel: {}", style(k).yellow()),
        MessageType::Close,
        3,
    );
    Ok(())
}

fn test_virtual_memory(
    _matches: &ArgMatches,
    messenger: &Sender<ShellMessage>,
) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    // debug!(logger, "opened sentry: {:?}", device);
    ShellMessage::send(
        messenger,
        format!("opened sentry: {:?}", style(&device).cyan()),
        MessageType::Spinner,
        0,
    );

    let v = common::dummy_vector(0x200);

    let size = v.len();

    // debug!(logger, "write-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());
    ShellMessage::send(
        messenger,
        format!(
            "write-buffer({}) with size of {}",
            style(format!("0x{:016x}", v.as_ptr() as u64)).cyan(),
            // v.as_ptr() as u64,
            // v.len()),
            style(format!("0x{:08x}", v.len())).magenta()
        ),
        MessageType::Spinner,
        0,
    );

    let addr = memory::alloc_virtual_memory(&device, size).unwrap();

    // debug!(logger, "alloc_virtual_memory: 0x{:016x}", addr);
    ShellMessage::send(
        messenger,
        format!(
            "alloc_virtual_memory: {}",
            style(format!("0x{:016x}", addr)).cyan()
        ),
        MessageType::Spinner,
        0,
    );

    let written = memory::write_virtual_memory(&device, addr, v).unwrap();

    // debug!(logger, "write_virtual_memory: {} bytes written", written);
    ShellMessage::send(
        messenger,
        format!(
            "write_virtual_memory: {} bytes written",
            style(written).underlined().yellow()
        ),
        MessageType::Close,
        0,
    );

    let v = memory::read_virtual_memory(&device, addr, size).unwrap();

    // debug!(logger, "reading 0x{:08x} bytes from 0x{:016x}", addr, size);
    ShellMessage::send(
        messenger,
        format!(
            "reading {} bytes from {}",
            style(size).underlined().yellow(),
            style(format!("0x{:016x}", addr)).cyan()
        ),
        MessageType::Spinner,
        0,
    );

    // debug!(logger, "read-buffer(0x{:016x}) with size of 0x{:08x}", v.as_ptr() as u64, v.len());
    ShellMessage::send(
        messenger,
        format!(
            "read-buffer({}) with size of {}",
            style(format!("0x{:016x}", v.as_ptr() as u64)).cyan(),
            style(format!("0x{:08x}", v.len())).cyan()
        ),
        MessageType::Close,
        0,
    );

    let output = common::dump_vector(&v);

    // debug!(logger, "{}", output);
    // Here dumps binary as hex in lines
    for line in format!("{}", output).split("\n") {
        ShellMessage::send(
            messenger,
            format!("{}", style(&line.to_string()).blue()),
            MessageType::Spinner,
            0,
        );
    }

    // debug!(logger, "free_virtual_memory: 0x{:016x}", addr);
    ShellMessage::send(
        messenger,
        format!(
            "free_virtual_memory: {}",
            style(format!("0x{:016x}", addr)).underlined().green()
        ),
        MessageType::Close,
        0,
    );
    memory::free_virtual_memory(&device, addr).unwrap();
    Ok(())
}

fn test_memory_write(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");
    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    // debug!(logger, "reading virtual memory");
    ShellMessage::send(
        messenger,
        format!("[*] {} virtual memory...", style("reading").blue()),
        MessageType::Spinner,
        0,
    );
    let v = memory::read_virtual_memory(&device, addr, 0x200).unwrap();

    // debug!(logger, "writting virtual memory");
    ShellMessage::send(
        messenger,
        format!(
            "[/] {} virtual memory...",
            style("writing").underlined().yellow()
        ),
        MessageType::Spinner,
        0,
    );
    memory::write_virtual_memory(&device, addr, v).unwrap();
    memory::free_virtual_memory(&device, addr).unwrap();
    ShellMessage::send(
        messenger,
        format!(
            "[!] virtual memory written::{} ",
            style("Done!").underlined().green()
        ),
        MessageType::Close,
        0,
    );
    Ok(())
}

fn test_memory_map(_matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    let device = Device::new(io::SE_NT_DEVICE_NAME).expect("Can't open sentry");

    let addr = memory::alloc_virtual_memory(&device, 0x200).unwrap();
    let map = Map::new(&device, addr, 0x200, Some(MapMode::UserMode));

    // debug!(logger, "map: {:?}", map);
    ShellMessage::send(
        messenger,
        format!("[*] {}: {:?}", style("map").cyan(), map),
        MessageType::Close,
        0,
    );
    memory::free_virtual_memory(&device, addr).unwrap();

    Ok(())
}
