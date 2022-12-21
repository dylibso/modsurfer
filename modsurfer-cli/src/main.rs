use std::{env, path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use url::Url;

mod cli;
mod plugins;

use cli::{Cli, Hash, Id, Limit, Offset, Version};

const BASE_URL_ENV: &'static str = "MODSURFER_BASE_URL";
const DEFAULT_BASE_URL: &'static str = "http://localhost:1739";

#[tokio::main]
async fn main() -> Result<ExitCode> {
    // get MODSURFER_BASE_URL environment variable if set
    let base_url = Url::parse(
        env::var_os(BASE_URL_ENV)
            .unwrap_or_else(|| DEFAULT_BASE_URL.into())
            .to_str()
            .unwrap_or(DEFAULT_BASE_URL),
    )?;
    let cmd = Command::new("modsurfer")
        .about("Modsurfer CLI is used to interact with the HTTP API.")
        .before_help("Copyright Dylibso, Inc. <support@dylib.so>")
        .subcommands(make_subcommands());

    Cli::new(cmd, base_url).execute().await
}

fn make_subcommands() -> Vec<Command> {
    let create = clap::Command::new("create")
        .about("Create a new entry for a module.")
        .arg(
            Arg::new("path")
                .value_parser(clap::value_parser!(PathBuf))
                .long("path")
                .short('p')
                .help("a path on disk to a valid WebAssembly module"),
        )
        .arg(
            Arg::new("location")
                .value_parser(clap::value_parser!(url::Url))
                .long("location")
                .short('l')
                .help("a valid URL to where this module should be located"),
        )
        .arg(
            Arg::new("validate")
                .value_parser(clap::value_parser!(PathBuf))
                .long("validate")
                .help("a path on disk to a YAML file which declares validation requirements")
                .default_value("mod.yaml"),
        );

    let delete = clap::Command::new("delete")
        .about("Delete a module and its versions.")
        .arg(
            Arg::new("id")
                .value_parser(clap::value_parser!(i64))
                .long("id")
                .action(ArgAction::Append)
                .help("the numeric ID of a module entry in Modsurfer"),
        );

    let get = clap::Command::new("get")
        .about("Get a module by its ID.")
        .arg(
            Arg::new("id")
                .value_parser(clap::value_parser!(Id))
                .long("id")
                .help("the numeric ID of a module entry in Modsurfer"),
        );

    let list = clap::Command::new("list")
        .about(
            "List all modules, paginated by the `offset` and `limit` parameters or their defaults.",
        )
        .arg(
            Arg::new("offset")
                .value_parser(clap::value_parser!(Offset))
                .long("offset")
                .default_value("0")
                .help("the pagination offset by which modules are listed"),
        )
        .arg(
            Arg::new("limit")
                .value_parser(clap::value_parser!(Limit))
                .long("limit")
                .default_value("50")
                .help("the maximum number of modules in a list of results"),
        );

    let search = clap::Command::new("search")
        .about("Search for modules matching optional parameters.")
        .arg(
            Arg::new("function_name")
                .long("function_name")
                .help("adds a search parameter to match on `function_name"),
        )
        .arg(
            Arg::new("module_name")
                .long("module_name")
                .help("adds a search parameter to match on `module_name`"),
        )
        .arg(
            Arg::new("source_language")
                .long("source_language")
                .help("adds a search parameter to match on `source_language`"),
        )
        .arg(
            Arg::new("hash")
                .value_parser(clap::value_parser!(Hash))
                .long("hash")
                .help("adds a search parameter to match on `hash`"),
        )
        .arg(
            Arg::new("strings")
                .long("strings")
                .help("adds a search parameter to match on `strings`"),
        );

    let validate = clap::Command::new("validate")
        .about("Validate a module using a module requirement file.")
        .arg(
            Arg::new("path")
                .value_parser(clap::value_parser!(PathBuf))
                .long("path")
                .short('p')
                .help("a path on disk to a valid WebAssembly module"),
        )
        .arg(
            Arg::new("check")
                .value_parser(clap::value_parser!(PathBuf))
                .long("check")
                .short('c')
                .default_value("mod.yaml")
                .help("a path on disk to a YAML file which declares validation requirements"),
        );

    let yank = clap::Command::new("yank")
        .about("Mark a module version as yanked (unavailable).")
        .arg(
            Arg::new("id")
                .value_parser(clap::value_parser!(Id))
                .long("id")
                .help("the numeric ID of a module entry in Modsurfer"),
        )
        .arg(
            Arg::new("version")
                .value_parser(clap::value_parser!(Version))
                .long("version")
                .help("the version of a module entry in Modsurfer (if no version exists, this command has no effect)",
        ));

    vec![create, delete, get, list, search, validate, yank]
}
