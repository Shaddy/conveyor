// Copyright © ByteHeed.  All rights reserved.

use super::cli::output::ShellMessage;
use std::sync::mpsc::Sender;

use super::failure::Error;

// pub fn bind() -> App<'static, 'static> {
//     SubCommand::with_name("misc")
//             .subcommand(SubCommand::with_name("dissasm"))
// }
//
//
// pub fn tests(matches: &ArgMatches, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
//     match matches.subcommand() {
//         ("dissasm",  Some(matches))  => test_disassembler(matches, messenger),
//         _                            => Ok(println!("{}", matches.usage()))
//     }
// }
//
// const CODE: &'static [u8] =
//     b"\x55\x48\x8b\x05\xb8\x13\x00\x00\xe8\x4a\xed\xff\xff\xe9\x14\x9e\x08\x00\x45\x31\xe4";

// /// Print register names
// fn reg_names<T, I>(cs: &Capstone, regs: T) -> String
// where
//     T: Iterator<Item = I>,
//     I: Into<u64>,
// {
//     let names: Vec<String> = regs.map(|x| cs.reg_name(x.into()).unwrap()).collect();
//     names.join(", ")
// }

// /// Print instruction group names
// fn group_names<T, I>(cs: &Capstone, regs: T) -> String
// where
//     T: Iterator<Item = I>,
//     I: Into<u64>,
// {
//     let names: Vec<String> = regs.map(|x| cs.group_name(x.into()).unwrap()).collect();
//     names.join(", ")
// }

// fn example() -> CsResult<()> {
//     let cs = Capstone::new()
//         .x86()
//         .mode(arch::x86::ArchMode::Mode64)
//         .syntax(arch::x86::ArchSyntax::Att)
//         .detail(true)
//         .build()?;

//     let insns = cs.disasm_all(CODE, 0x1000)?;
//     println!("Found {} instructions", insns.len());
//     for i in insns.iter() {
//         println!("");
//         println!("{}", i);
//         let output: &[(&str, String)] =
//             &[
//                 (
//                     "read regs:",
//                     reg_names(&cs, cs.read_register_ids(&i)?.iter().map(|x| *x)),
//                 ),
//                 (
//                     "write regs:",
//                     reg_names(&cs, cs.write_register_ids(&i)?.iter().map(|x| *x)),
//                 ),
//                 (
//                     "insn groups:",
//                     group_names(&cs, cs.insn_group_ids(&i)?.iter().map(|x| *x)),
//                 ),
//             ];
//         for &(ref name, ref message) in output.iter() {
//             println!("    {:12} {}", name, message);
//         }
//     }
//     Ok(())
// }

fn test_disassembler(_messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    // debug!(logger, "{}", ());
    // example().unwrap();
    unimplemented!()
}
