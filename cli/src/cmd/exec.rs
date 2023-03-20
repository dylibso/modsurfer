#![allow(unused)]
use std::io::Write;
use std::process::ExitCode;
use std::{collections::HashMap, ffi::OsString, path::PathBuf};

use anyhow::Result;
use human_bytes::human_bytes;
use modsurfer_api::{ApiClient, Client, Persisted};
use modsurfer_convert::{Audit, AuditOutcome, Pagination};
use modsurfer_module::{Module, SourceLanguage};
use modsurfer_validation::validate_module;
use serde::Serialize;
use url::Url;

use super::api_result::{ApiResult, ApiResults, SimpleApiResult, SimpleApiResults};
use super::generate::checkfile_from_module;

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

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Json,
    Table,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
}

impl From<String> for OutputFormat {
    fn from(value: String) -> Self {
        match value.as_str() {
            "json" => Self::Json,
            _ => Self::Table,
        }
    }
}

impl From<OsString> for OutputFormat {
    fn from(value: OsString) -> Self {
        let s = value.into_string().unwrap_or_default();
        s.into()
    }
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
        &'a OutputFormat,
    ),
    Delete(Vec<Id>, &'a OutputFormat),
    Get(Id, &'a OutputFormat),
    List(Offset, Limit, &'a OutputFormat),
    Search(
        Option<&'a Hash>,
        Option<&'a ModuleName>,
        Option<&'a FunctionName>,
        Option<SourceLanguage>,
        Option<&'a TextSearch>,
        Offset,
        Limit,
        &'a OutputFormat,
    ),
    Generate(ModuleFile, CheckFile),
    Validate(ModuleFile, CheckFile, &'a OutputFormat),
    Yank(Id, Version, &'a OutputFormat),
    Audit(CheckFile, AuditOutcome, Offset, Limit, &'a OutputFormat),
}

impl Cli {
    pub fn new(mut cmd: clap::Command, host: Url) -> Self {
        let help = cmd.render_long_help().to_string();

        Self { cmd, help, host }
    }

    pub async fn execute(&self) -> Result<ExitCode> {
        match self.cmd.clone().get_matches().subcommand() {
            Some(x) => self.run(x).await,
            _ => {
                println!("{}", self.help);
                Ok(ExitCode::SUCCESS)
            }
        }
    }

    async fn run(&self, sub: impl Into<Subcommand<'_>>) -> Result<ExitCode> {
        match sub.into() {
            Subcommand::Unknown => unimplemented!("Unknown subcommand.\n\n{}", self.help),
            Subcommand::Create(module_path, checkfile_path, metadata, location, output_format) => {
                if let Some(check) = checkfile_path {
                    let report = validate_module(&module_path, check).await?;
                    if report.has_failures() {
                        println!(
                            "{}",
                            match output_format {
                                OutputFormat::Json => serde_json::to_string_pretty(&report)?,
                                OutputFormat::Table => report.to_string(),
                            }
                        );

                        return Ok(report.as_exit_code());
                    }
                }

                let wasm = tokio::fs::read(module_path).await?;
                let client = Client::new(self.host.as_str())?;
                let (id, hash) = client.create_module(wasm, Some(metadata), location).await?;

                let output = SimpleApiResults {
                    results: vec![SimpleApiResult {
                        module_id: id,
                        hash: hash.clone(),
                    }],
                };

                println!(
                    "{}",
                    match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&output)?,
                        OutputFormat::Table => output.to_string(),
                    }
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Delete(ids, output_format) => {
                let client = Client::new(self.host.as_str())?;
                let deleted_modules = client.delete_modules(ids).await?;

                let results = deleted_modules
                    .iter()
                    .map(|(id, hash)| SimpleApiResult {
                        module_id: *id,
                        hash: hash.clone(),
                    })
                    .collect();

                let output = SimpleApiResults { results };

                println!(
                    "{}",
                    match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&output)?,
                        OutputFormat::Table => output.to_string(),
                    }
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Get(id, output_format) => {
                let client = Client::new(self.host.as_str())?;
                let m = client.get_module(id).await?;
                let results = vec![to_api_result(&m)];
                let output = ApiResults { results };

                println!(
                    "{}",
                    match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&output)?,
                        OutputFormat::Table => output.to_string(),
                    }
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::List(offset, limit, output_format) => {
                let client = Client::new(self.host.as_str())?;
                let list = client.list_modules(offset, limit).await?;

                let results = list.vec().into_iter().map(to_api_result).collect();
                let output = ApiResults { results };

                println!(
                    "{}",
                    match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&output)?,
                        OutputFormat::Table => output.to_string(),
                    }
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Search(
                hash,
                mod_name,
                func_name,
                src_lang,
                text_search,
                offset,
                limit,
                output_format,
            ) => {
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
                        None,
                        None,
                    )
                    .await?;

                let results = modules.vec().into_iter().map(to_api_result).collect();
                let output = ApiResults { results };

                println!(
                    "{}",
                    match output_format {
                        OutputFormat::Json => serde_json::to_string_pretty(&output)?,
                        OutputFormat::Table => output.to_string(),
                    }
                );

                Ok(ExitCode::SUCCESS)
            }
            Subcommand::Generate(file, check) => match checkfile_from_module(&file, &check).await {
                Ok(_) => Ok(ExitCode::SUCCESS),
                Err(e) => {
                    println!("{:?}", e);
                    Ok(ExitCode::FAILURE)
                }
            },
            Subcommand::Validate(file, check, output_format) => {
                let report = validate_module(&file, &check).await?;
                match output_format {
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
                    OutputFormat::Table => {
                        if report.has_failures() {
                            println!("{report}")
                        }
                    }
                };
                Ok(report.as_exit_code())
            }
            Subcommand::Yank(_id, _version, _output_format) => {
                println!("`yank` is not yet supported. Reach out to support@dylib.so for more information!");

                Ok(ExitCode::FAILURE)
            }
            Subcommand::Audit(check, outcome, offset, limit, output_format) => {
                let checkfile = tokio::fs::read(&check).await?;
                let page = Pagination { offset, limit };
                let audit = Audit {
                    checkfile,
                    page,
                    outcome,
                };

                let client = Client::new(self.host.as_str())?;
                let reports = client.audit_modules(audit).await?;

                match output_format {
                    OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&reports)?),
                    OutputFormat::Table => {
                        let mut buf = vec![];
                        reports.iter().enumerate().for_each(|(i, (id, report))| {
                            if i != 0 {
                                writeln!(buf, "");
                            }
                            writeln!(buf, "Report for module: {id}");
                            writeln!(buf, "{}", report);
                        });

                        print!("{}", String::from_utf8(buf)?);
                    }
                };

                Ok(ExitCode::SUCCESS)
            }
        }
    }
}

fn to_api_result(m: &Persisted<Module>) -> ApiResult {
    ApiResult {
        module_id: m.get_id(),
        hash: m.get_inner().hash.clone(),
        file_name: m.get_inner().file_name(),
        exports: m.get_inner().exports.len(),
        imports: m.get_inner().imports.len(),
        namespaces: m.get_inner().get_import_namespaces(),
        source_language: m.get_inner().source_language.clone(),
        size: human_bytes(m.get_inner().size as f64),
    }
}

fn output_format(args: &clap::ArgMatches) -> &OutputFormat {
    args.get_one("output-format")
        .unwrap_or_else(|| &OutputFormat::Table)
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

                Subcommand::Create(
                    module_path,
                    checkfile_path,
                    metadata,
                    location.cloned(),
                    output_format(args),
                )
            }
            ("delete", args) => Subcommand::Delete(
                args.get_many("id")
                    .expect("module id(s) to delete")
                    .cloned()
                    .collect::<Vec<Id>>(),
                output_format(args),
            ),
            ("get", args) => Subcommand::Get(
                *args.get_one("id").expect("valid moudle ID"),
                output_format(args),
            ),
            ("list", args) => Subcommand::List(
                *args.get_one("offset").unwrap_or_else(|| &0),
                *args.get_one("limit").unwrap_or_else(|| &50),
                output_format(args),
            ),
            ("search", args) => {
                let hash: Option<&Hash> = args.get_one("hash");
                let mod_name: Option<&ModuleName> = args.get_one("module-name");
                let func_name: Option<&FunctionName> = args.get_one("function-name");
                let src_lang: Option<SourceLanguage> = args
                    .get_one("source-language")
                    .map(|s: &String| s.clone().into());
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
                    output_format(args),
                )
            }
            ("generate", args) => Subcommand::Generate(
                args.get_one::<PathBuf>("path")
                    .expect("valid module path")
                    .clone(),
                args.get_one::<PathBuf>("output")
                    .expect("valid checkfile output path")
                    .clone(),
            ),
            ("validate", args) => Subcommand::Validate(
                args.get_one::<PathBuf>("path")
                    .expect("valid module path")
                    .clone(),
                args.get_one::<PathBuf>("check")
                    .expect("valid checkfile path")
                    .clone(),
                output_format(args),
            ),
            ("yank", args) => Subcommand::Yank(
                *args.get_one::<Id>("id").expect("id is required"),
                args.get_one::<Version>("version")
                    .expect("version is required")
                    .clone(),
                output_format(args),
            ),
            ("audit", args) => {
                let offset: Offset = *args
                    .get_one("offset")
                    .expect("offset should have default value");
                let limit: Limit = *args
                    .get_one("limit")
                    .expect("limit should have default value");
                Subcommand::Audit(
                    args.get_one::<PathBuf>("check")
                        .expect("valid checkfile path")
                        .clone(),
                    args.get_one::<AuditOutcome>("outcome")
                        .expect("requires valid outcome ('pass' or 'fail')")
                        .clone(),
                    offset,
                    limit,
                    output_format(args),
                )
            }
            _ => Subcommand::Unknown,
        }
    }
}
