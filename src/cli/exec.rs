use std::{collections::HashMap, path::PathBuf};

use url;

pub type Id = i64;
pub type Hash = String;
pub type Offset = u32;
pub type Limit = u32;
pub type Version = String;

#[derive(Debug)]
pub struct Cli {
    cmd: clap::Command,
    help: String,
    host: url::Url,
}

#[derive(Debug, Default)]
pub enum Subcommand {
    #[default]
    Unknown,
    Create(PathBuf, HashMap<String, String>, Option<url::Url>),
    Delete(Vec<Id>),
    Get(Id),
    List(Offset, Limit),
    Search(
        Option<Id>,
        Option<Hash>,
        Option<String>,
        Option<Offset>,
        Option<Limit>,
    ),
    Yank(Id, Version),
}

impl Cli {
    pub fn new(mut cmd: clap::Command) -> Self {
        let help = cmd.render_long_help().to_string();
        let host = cmd
            .clone()
            .get_matches_from(["host"])
            .get_one::<url::Url>("host")
            .expect("host is present or default")
            .clone();

        Self { cmd, help, host }
    }

    pub fn execute(&self) {
        match self.cmd.clone().get_matches().subcommand() {
            Some(x) => self.run(x.into()),
            _ => println!("{}", self.help),
        }
    }

    fn run(&self, sub: Subcommand) {
        match sub {
            Subcommand::Unknown => println!("Unknown subcommand.\n\n{}", self.help),
            Subcommand::Create(_, _, _) => todo!(),
            Subcommand::Delete(_) => todo!(),
            Subcommand::Get(id) => todo!("make request for module ID: {}", id),
            Subcommand::List(_, _) => todo!(),
            Subcommand::Search(_, _, _, _, _) => todo!(),
            Subcommand::Yank(_, _) => todo!(),
        }
    }
}

impl From<(&str, &clap::ArgMatches)> for Subcommand {
    fn from(input: (&str, &clap::ArgMatches)) -> Self {
        match input {
            ("create", args) => todo!(),
            ("delete", args) => todo!(),
            ("get", args) => Subcommand::Get(*args.get_one("id").expect("valid moudle ID")),
            ("list", args) => todo!(),
            ("search", args) => todo!(),
            ("yank", args) => todo!(),
            _ => Subcommand::Unknown,
        }
    }
}
