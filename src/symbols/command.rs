use super::clap::{Parser, Subcommand};
use super::cli::output::{MessageType, ShellMessage};
use super::console::style;
// use super::downloader::PdbDownloader;
use super::failure::Error;
// use super::parser;
use std::sync::mpsc::Sender;

#[derive(Subcommand)]
#[clap(name = "pdb", about = "PDB parser")]
pub enum PdbCommands {
    Download {
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "binary file to download PDB"
        )]
        target: String,
    },
    Parse {
        #[clap(
            short = 's',
            long = "struct",
            value_name = "STRUCT",
            help = "struct to find while parsing"
        )]
        struct_name: String,
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "binary file to download PDB"
        )]
        target: String,
    },
    Offset {
        #[clap(
            short = 's',
            long = "struct",
            value_name = "STRUCT",
            help = "struct to find while parsing"
        )]
        struct_name: String,
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "binary file to download PDB"
        )]
        target: String,
    },
    Analyze {
        #[clap(
            short = 's',
            long = "struct",
            value_name = "STRUCT",
            help = "struct to find while parsing"
        )]
        struct_name: String,
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "binary file to download PDB"
        )]
        target: String,
    },
    Dump {
        #[clap(
            short = 's',
            long = "struct",
            value_name = "STRUCT",
            help = "struct to find while parsing"
        )]
        struct_name: String,
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "binary file to download PDB"
        )]
        target: String,
    },
}

pub fn _not_implemented_subcommand(_messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    unimplemented!()
}

fn _not_implemented_command(_messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    unimplemented!()
}

pub fn parse(commands: &PdbCommands, messenger: &Sender<ShellMessage>) -> Result<(), Error> {
    match commands {
        PdbCommands::Download { target } => command_download_pdb(messenger, target),
        PdbCommands::Parse {
            target,
            struct_name,
        } => command_parse_pdb(messenger, &target, struct_name),
        PdbCommands::Offset {
            target,
            struct_name,
        } => command_find_offset(messenger, &target, struct_name),
        // PdbCommands::Analyze { target, struct_name } => analyze_pdb(&target, &struct_name, messenger),
        // PdbCommands::Dump { target, struct_name } => dump_pdb(&target, &struct_name, messenger),
        _ => unimplemented!("Command not implemented"),
    }
}

fn command_find_offset(
    messenger: &Sender<ShellMessage>,
    target: &str,
    struct_name: &str,
) -> Result<(), Error> {
    ShellMessage::send(
        messenger,
        format!(
            "parsing {} to find {} offset",
            style(target).cyan(),
            style(struct_name).magenta()
        ),
        MessageType::Close,
        0,
    );

    // TODO: Conversion between Pdb error
    // let _ = parser::find_offset(target, struct_name);

    Ok(())
}

fn command_parse_pdb(
    messenger: &Sender<ShellMessage>,
    target: &str,
    struct_name: &str,
) -> Result<(), Error> {
    ShellMessage::send(
        messenger,
        format!(
            "parsing {} searching {}",
            style(target).cyan(),
            style(struct_name).magenta()
        ),
        MessageType::Close,
        0,
    );

    // parser::pdb_to_c_struct(target, struct_name, messenger);

    Ok(())
}

fn command_download_pdb(messenger: &Sender<ShellMessage>, target: &str) -> Result<(), Error> {
    // let pdb = PdbDownloader::new(target.to_string());
    //
    // // TODO: Propagate the error
    // pdb.download(messenger).expect("Unable to download PDB");

    Ok(())
}
