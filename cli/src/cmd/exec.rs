#![allow(unused)]
use std::process::ExitCode;
use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use anyhow::Result;
use modsurfer_api::{ApiClient, Client};
use modsurfer_module::SourceLanguage;
use url::Url;

use super::validate::validate_module;

pub type Id = i64;
pub type Hash = String;
pub type ModuleName = String;
pub type FunctionName = String;
pub type TextSearch = String;
pub type Offset = u32;
pub type Limit = u32;
pub type Version = String;
pub type ModuleFile = PathBuf;
pub type CheckFile = PathBuf;
pub type MetadataEntry = String;

#[derive(Debug)]
pub struct Cli {
    cmd: clap::Command,
    help: String,
    host: Url,
}

#[derive(Debug, Default)]
pub enum Subcommand<'a> {
    #[default]
    Unknown,
    Create(
        &'a ModuleFile,
        Option<&'a CheckFile>,
        HashMap<String, String>,
        Option<Url>,
    ),
    Delete(Vec<Id>),
    Get(Id),
    List(Offset, Limit),
    Search(
        Option<&'a Hash>,
        Option<&'a ModuleName>,
        Option<&'a FunctionName>,
        Option<&'a SourceLanguage>,
        Option<&'a TextSearch>,
        Offset,
        Limit,
    ),
    Validate(ModuleFile, CheckFile),
    Yank(Id, Version),
}

impl Cli {
    pub fn new(mut cmd: clap::Command, host: Url) -> Self {
        let help = cmd.render_long_help().to_string();

        Self { cmd, help, host }
    }

    pub async fn execute(&self) -> Result<ExitCode> {
        match self.cmd.clone().get_matches().subcommand() {
            Some(x) => self.run(x).await,
            _ => anyhow::bail!("{}", self.help),
        }
    }

    async fn run(&self, sub: impl Into<Subcommand<'_>>) -> Result<ExitCode> {
        match sub.into() {
            Subcommand::Unknown => unimplemented!("Unknown subcommand.\n\n{}", self.help),
            Subcommand::Create(module_path, checkfile_path, metadata, location) => {
                if let Some(check) = checkfile_path {
                    validate_module(&module_path, check).await?;
                }

                let wasm = tokio::fs::read(module_path).await?;
                let client = Client::new(self.host.as_str())?;
                let (id, hash) = client.create_module(wasm, Some(metadata), location).await?;

                println!("Module {} ({}) created", id, hash);

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Delete(ids) => {
                let client = Client::new(self.host.as_str())?;
                let deleted_modules = client.delete_modules(ids).await?;
                println!("Deleted: {:#?}", deleted_modules);
                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Get(id) => {
                let client = Client::new(self.host.as_str())?;
                let module = client.get_module(id).await?.get_inner().clone();
                println!("Module: ({}) {} {}", id, module.location, module.size);
                Ok(ExitCode::SUCCESS)
            }
            Subcommand::List(offset, limit) => {
                let client = Client::new(self.host.as_str())?;
                let list = client.list_modules(offset, limit).await?;
                println!("List length: {}", list.vec().len());
                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Search(hash, mod_name, func_name, src_lang, text_search, offset, limit) => {
                let client = Client::new(self.host.as_str())?;
                let modules = client
                    .search_modules(
                        None,
                        hash.map(String::clone),
                        func_name.map(String::clone),
                        mod_name.map(String::clone),
                        None,
                        None,
                        None,
                        None,
                        None,
                        src_lang.map(|lang| lang.to_string()),
                        None,
                        None,
                        None,
                        text_search.map(|s| vec![s.clone()]),
                        offset,
                        limit,
                    )
                    .await?;

                println!(
                    "{:#?}",
                    modules
                        .vec()
                        .iter()
                        .map(|m| format!(
                            "{} ({}) {}",
                            m.get_id(),
                            m.get_inner().hash,
                            m.get_inner().file_name()
                        ))
                        .collect::<Vec<String>>()
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Validate(file, check) => {
                let report = validate_module(&file, &check).await?;
                println!("{report}");
                Ok(report.as_exit_code())
            }
            Subcommand::Yank(_, _) => todo!(),
        }
    }
}

impl<'a> From<(&'a str, &'a clap::ArgMatches)> for Subcommand<'a> {
    fn from(input: (&'a str, &'a clap::ArgMatches)) -> Self {
        match input {
            ("create", args) => {
                let module_path = args
                    .get_one::<PathBuf>("path")
                    .expect("must provide a --path to the module on disk");
                let checkfile_path: Option<&PathBuf> = args.get_one("check");
                let raw_metadata = args
                    .get_many("metadata")
                    .unwrap_or_default()
                    .cloned()
                    .collect::<Vec<String>>();

                let metadata: HashMap<String, String> = raw_metadata
                    .into_iter()
                    .map(|raw| {
                        let parts = raw.split("=").collect::<Vec<_>>();
                        (parts[0].to_string(), parts[1].to_string())
                    })
                    .collect();

                let location: Option<&Url> = args.get_one("location");

                Subcommand::Create(module_path, checkfile_path, metadata, location.cloned())
            }
            ("delete", args) => Subcommand::Delete(
                args.get_many("id")
                    .expect("module id(s) to delete")
                    .cloned()
                    .collect::<Vec<Id>>(),
            ),
            ("get", args) => Subcommand::Get(*args.get_one("id").expect("valid moudle ID")),
            ("list", args) => Subcommand::List(
                *args.get_one("offset").unwrap_or_else(|| &0),
                *args.get_one("limit").unwrap_or_else(|| &50),
            ),
            ("search", args) => {
                // hash, mod_name, func_name, src_lang, text_search, offset, limit
                let hash: Option<&Hash> = args.get_one("hash");
                let mod_name: Option<&ModuleName> = args.get_one("module_name");
                let func_name: Option<&FunctionName> = args.get_one("function_name");
                let src_lang: Option<&SourceLanguage> = args.get_one("source_language");
                let text_search: Option<&TextSearch> = args.get_one("text");
                let offset: Offset = *args
                    .get_one("offset")
                    .expect("offset should have default value");
                let limit: Limit = *args
                    .get_one("limit")
                    .expect("limit should have default value");

                Subcommand::Search(
                    hash,
                    mod_name,
                    func_name,
                    src_lang,
                    text_search,
                    offset,
                    limit,
                )
            }
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
