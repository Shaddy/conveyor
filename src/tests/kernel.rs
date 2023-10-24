// Copyright © ByteHeed. All rights reserved.

use super::iochannel::{Device, IoCtl};

use std::sync::mpsc::Sender;

use super::cli::output::ShellMessage;

use super::failure::Error;
use super::sentry::io;

use super::structs::{RawStruct, TestFlags, TestType, SE_RUN_TEST};

#[derive(Debug)]
struct SentryTest {
    pub kind: TestType,
    pub flags: TestFlags,
}

impl SentryTest {
    pub fn set_flag(&mut self, flag: TestFlags) {
        self.flags |= flag;
    }

    pub fn new(kind: TestType, flags: Option<TestFlags>) -> SentryTest {
        SentryTest {
            kind: kind,
            flags: flags.unwrap_or(TestFlags::INTERCEPT_NORMAL),
        }
    }
}

// pub fn bind() -> App<'static, 'static> {
//     SubCommand::with_name("sentry")
//                     .subcommand(SubCommand::with_name("guard"))
//                     .subcommand(SubCommand::with_name("region"))
//                     .subcommand(SubCommand::with_name("tracepoint"))
//                     .subcommand(SubCommand::with_name("intercept")
//                         .arg(Arg::with_name("stress").short("s")
//                                 .required(false)
//                                 .value_name("STRESS")
//                                 .help("interception stress affinity"))
//                         .subcommand(SubCommand::with_name("basic"))
//                         .subcommand(SubCommand::with_name("setevent"))
//                         .subcommand(SubCommand::with_name("delay"))
//                         .subcommand(SubCommand::with_name("timer"))
//                         .subcommand(SubCommand::with_name("priority"))
//                         .subcommand(SubCommand::with_name("pagefault")))
// }
//
// pub fn tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     let mut test = match matches.subcommand() {
//         ("guard",        Some(_))  => SentryTest::new(TestType::BasicGuard, None),
//         ("region",       Some(_))  => SentryTest::new(TestType::BasicGuardedRegion, None),
//         ("tracepoint",   Some(_))  => SentryTest::new(TestType::BasicTracePoint, None),
//         ("intercept",    Some(matches))  => parse_intercept(matches, messenger),
//         _                                => {
//                 let message = format!("{}", matches.usage());
//                 panic!(message);
//         }
//     };
//
//     if matches.is_present("stress") {
//         test.set_flag(TestFlags::INTERCEPT_STRESS_AFFINITY);
//     }
//
//     let device = Device::new(io::SE_NT_DEVICE_NAME)?;
//
//     sentry_run_test(&device, test)
// }
//
// fn parse_intercept(matches: &ArgMatches, _messenger: &Sender<ShellMessage>) -> SentryTest {
//     match matches.subcommand() {
//         ("basic",      Some(_))  => SentryTest::new(TestType::BasicIntercept, None),
//         ("delay",      Some(_))  => SentryTest::new(TestType::DelayIntercept, None),
//         ("setevent",   Some(_))  => SentryTest::new(TestType::SetEvent, None),
//         ("pagefault",  Some(_))  => SentryTest::new(TestType::PageFaultIntercept, None),
//         ("priority",   Some(_))  => SentryTest::new(TestType::PriorityIntercept, None),
//         ("timer",      Some(_))  => SentryTest::new(TestType::TimerIntercept, None),
//         _                              => {
//             let message = format!("{}", matches.usage());
//             panic!(message);
//         }
//     }
// }
//
// fn sentry_run_test(device: &Device, test: SentryTest) -> Result<(), Error> {
//     let control = IoCtl::new(Some("SE_RUN_TEST"), io::IOCTL_SENTRY_TYPE, 0x0A63, None, None);
//
//     let mut write = SE_RUN_TEST::init();
//
//     write.Kind = test.kind;
//     write.Flags = test.flags;
//
//     let (ptr, len) = (write.as_ptr(), write.size());
//
//     device.raw_call(control, ptr, len)?;
//
//     Ok(())
// }
