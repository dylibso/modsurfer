#![allow(unused)]
use std::process::ExitCode;
use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use anyhow::Result;
use url::Url;

use super::get::get_module;
use super::validate::validate_module;

pub type Id = i64;
pub type Hash = String;
pub type Offset = u32;
pub type Limit = u32;
pub type Version = String;
pub type ModuleFile = PathBuf;
pub type ValidationFile = PathBuf;

#[derive(Debug)]
pub struct Cli {
    cmd: clap::Command,
    help: String,
    host: Url,
}

#[derive(Debug, Default)]
pub enum Subcommand {
    #[default]
    Unknown,
    Create(
        ModuleFile,
        ValidationFile,
        HashMap<String, String>,
        Option<Url>,
    ),
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
    Validate(ModuleFile, ValidationFile),
    Yank(Id, Version),
}

impl Cli {
    pub fn new(mut cmd: clap::Command, host: Url) -> Self {
        let help = cmd.render_long_help().to_string();

        Self { cmd, help, host }
    }

    pub async fn execute(&self) -> Result<ExitCode> {
        match self.cmd.clone().get_matches().subcommand() {
            Some(x) => self.run(x.into()).await,
            _ => anyhow::bail!("{}", self.help),
        }
    }

    async fn run(&self, sub: Subcommand) -> Result<ExitCode> {
        match sub {
            Subcommand::Unknown => unimplemented!("Unknown subcommand.\n\n{}", self.help),
            Subcommand::Create(_, _, _, _) => todo!(),
            Subcommand::Delete(_) => todo!(),
            Subcommand::Get(id) => {
                let module = get_module(&self.host, id).await?;
                Ok(ExitCode::SUCCESS)
            }
            Subcommand::List(_, _) => todo!(),
            Subcommand::Search(_, _, _, _, _) => todo!(),
            Subcommand::Validate(file, check) => {
                let report = validate_module(&file, &check).await?;
                println!("{report}");

                Ok(report.as_exit_code())
            }
            Subcommand::Yank(_, _) => todo!(),
        }
    }
}

impl From<(&str, &clap::ArgMatches)> for Subcommand {
    fn from(input: (&str, &clap::ArgMatches)) -> Self {
        match input {
            ("create", _args) => todo!(),
            ("delete", _args) => todo!(),
            ("get", args) => Subcommand::Get(*args.get_one("id").expect("valid moudle ID")),
            ("list", _args) => todo!(),
            ("search", _args) => todo!(),
            ("validate", args) => Subcommand::Validate(
                args.get_one::<PathBuf>("path")
                    .expect("valid module path")
                    .clone(),
                args.get_one::<PathBuf>("check")
                    .expect("valid check file path")
                    .clone(),
            ),
            ("yank", _args) => todo!(),
            _ => Subcommand::Unknown,
        }
    }
}
