use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author = "Sherab G. <sherab@reversingcode.com>", version = "2.0", about = "A gate between human and dragons", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: CliCommands,
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum CliCommands {
    Service {
        #[clap(subcommand)]
        command: crate::service::command::ServiceCommands,
    },
    Iochannel {
        #[clap(subcommand)]
        command: crate::iochannel::command::IoChannelCommands,
    },
    Pdb {
        #[clap(subcommand)]
        command: crate::symbols::command::PdbCommands,
    },
    Tests {
        #[clap(subcommand)]
        command: crate::tests::command::TestsCommands,
    },
    Sentry {
        #[clap(subcommand)]
        command: crate::sentry::command::SentryCommands,
    },
    // Patch {
    //     #[clap(subcommand)]
    //     command: crate::patch::command::PatchCommands,
    // },
    // Token {
    //     #[clap(subcommand)]
    //     command: crate::symbols::command::TokenCommands,
    // },
    Load {
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "service target"
        )]
        target: String,
    },
    Unload {
        #[clap(
            short = 't',
            long = "target",
            value_name = "TARGET",
            help = "service target"
        )]
        target: String,
    },
}

/*

   let target = Arg::with_name("target").short("t")
                           .required(true)
                           .value_name("TARGET")
                           .help("service target");



   let matches = App::new("conveyor")
       .about("A gate between humans and dragons.")
       .version("1.0")
       .author("Sherab G. <sherab.giovannini@byteheed.com>")
       .arg(Arg::with_name("v") .short("v") .multiple(true) .help("Sets the level of verbosity"))
       .subcommand(conveyor::service::command::bind())
       .subcommand(conveyor::iochannel::command::bind())
       .subcommand(conveyor::tests::command::bind())
       // .subcommand(conveyor::sentry::command::bind())
       .subcommand(conveyor::symbols::command::bind())
       .subcommand(conveyor::tests::patches::bind())
       .subcommand(conveyor::tests::token::bind())
       .subcommand(SubCommand::with_name("load")
                               .arg(target.clone()))
       .subcommand(SubCommand::with_name("unload")
                               .arg(target.clone()))
       .subcommand(conveyor::tests::monitor::bind())
       .get_matches();
 match app.subcommand() {
       ("device", Some(matches)) => iochannel::command::parse(matches, &messenger),
       ("pdb", Some(matches)) => symbols::command::parse(matches, &messenger),
       ("services", Some(matches)) => service::command::parse(matches, &messenger),
       ("tests", Some(matches)) => tests::command::parse(matches, &messenger),
       ("monitor", Some(matches)) => conveyor::tests::monitor::parse(matches, &messenger),
       ("patch", Some(matches)) => conveyor::tests::patches::parse(matches, &messenger),
       ("token", Some(matches)) => conveyor::tests::token::parse(matches, &messenger),
       ("sentry", Some(matches)) => sentry::command::parse(matches, &messenger),
       _ => Ok(println!("{}", app.usage())),
   }
*/
