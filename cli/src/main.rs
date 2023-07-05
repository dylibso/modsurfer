use std::{env, path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Arg, ArgAction, Command};
use modsurfer_convert::AuditOutcome;
use url::Url;

mod cmd;

use cmd::{Cli, Hash, Id, Limit, MetadataEntry, Offset, OutputFormat, Version};

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
        .about("Modsurfer CLI is used to interact with the HTTP API or validate modules offline.")
        .version(env!("CARGO_PKG_VERSION"))
        .before_help("Copyright Dylibso, Inc. <support@dylib.so>")
        .subcommands(make_subcommands());

    Cli::new(cmd, base_url).execute().await
}

fn add_output_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("output-format")
            .value_parser(clap::value_parser!(OutputFormat))
            .long("output-format")
            .required(false)
            .help("set the output format of any command, supports `json` or `table` (default)"),
    )
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
            Arg::new("metadata")
                .value_parser(clap::value_parser!(MetadataEntry))
                .long("metadata")
                .short('m')
                .action(ArgAction::Append)
                .required(false)
                .help(
                    "a repeatable key=value metadata entry, to add arbitrary context to a module",
                ),
        )
        .arg(
            Arg::new("location")
                .value_parser(clap::value_parser!(url::Url))
                .long("location")
                .short('l')
                .required(false)
                .help("a valid URL to where this module should be located"),
        )
        .arg(
            Arg::new("check")
                .value_parser(clap::value_parser!(PathBuf))
                .long("check")
                .short('c')
                .required(false)
                .help("a path on disk to a YAML checkfile which declares validation requirements"),
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
            Arg::new("function-name")
                .long("function-name")
                .required(false)
                .help("adds a search parameter to match on `function-name"),
        )
        .arg(
            Arg::new("module-name")
                .long("module-name")
                .required(false)
                .help("adds a search parameter to match on `module-name`"),
        )
        .arg(
            Arg::new("source-language")
                .long("source-language")
                .required(false)
                .help("adds a search parameter to match on `source-language`"),
        )
        .arg(
            Arg::new("hash")
                .value_parser(clap::value_parser!(Hash))
                .long("hash")
                .required(false)
                .help("adds a search parameter to match on `hash`"),
        )
        .arg(
            Arg::new("text")
                .long("text")
                .required(false)
                .help("adds a search parameter to match on `strings` extracted from a module"),
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

    let generate = clap::Command::new("generate")
        .about("Generate a starter checkfile from the given module.")
        .arg(
            Arg::new("path")
                .value_parser(clap::value_parser!(PathBuf))
                .long("path")
                .short('p')
                .help("a path on disk to a valid WebAssembly module"),
        )
        .arg(
            Arg::new("output")
                .value_parser(clap::value_parser!(PathBuf))
                .long("output")
                .short('o')
                .default_value("mod.yaml")
                .help("a path on disk to write a generated YAML checkfile"),
        );
    let validate = clap::Command::new("validate")
        .about("Validate a module using a module checkfile.")
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

    let audit = clap::Command::new("audit")
        .about("Return a list of modules which violate requirements in the provided checkfile.")
        .arg(
            Arg::new("outcome")
                .value_parser(clap::value_parser!(AuditOutcome))
                .long("outcome")
                .default_value("fail")
                .help("which type of expected outcome the audit should verify ('pass' or 'fail')"),
        )
        .arg(
            Arg::new("check")
                .value_parser(clap::value_parser!(PathBuf))
                .long("check")
                .short('c')
                .default_value("mod.yaml")
                .help("a path on disk to a YAML file which declares validation requirements"),
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

    let diff = clap::Command::new("diff")
        .about("Compare two modules")
        .arg(
            Arg::new("with-context")
                .value_parser(clap::value_parser!(bool))
                .long("with-context")
                .default_value("false")
                .action(clap::ArgAction::SetTrue)
                .help("retain the surrounding unchnaged lines in the diff as context"),
        )
        .arg(Arg::new("module1").help("first module ID or path to .wasm"))
        .arg(Arg::new("module2").help("second module ID or path to .wasm"));

    let call_plugin = clap::Command::new("call")
        .about("Call a Modsurfer plugin.")
        .arg(
            Arg::new("identifier")
                .long("id")
                .help("the identifier of the registered plugin"),
        )
        .arg(
            Arg::new("function")
                .long("function")
                .short('f')
                .help("the function to be called"),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .short('i')
                .help("use @{path_to_file} to specify a file as input to your plugin's function. Otherwise, the value provided will used as input to your function as raw bytes"),
        )
        .arg(
            Arg::new("output")
                .value_parser(clap::value_parser!(PathBuf))
                .long("output")
                .short('o')
                .help("a location on disk to write the output.  The output of the call will be written to stdout if not specified"),
        );

    // TODO: allow specification of plugin "config"
    let install_plugin = clap::Command::new("install")
        .about("Install a Modsurfer plugin.")
        .arg(
            Arg::new("identifier")
                .long("id")
                .help("the identifier of the plugin to be installed"),
        )
        .arg(
            Arg::new("name")
                .required(false)
                .long("name")
                .short('n')
                .help("the human readable name of the plugin"),
        )
        .arg(
            Arg::new("location")
                .required_unless_present("wasm")
                .long("loc")
                .short('l')
                .help("the location of the wasm to install as a plugin. Note: currently unimplemented, please use --wasm instead for now."),
        )
        .arg(
            Arg::new("wasm")
                .required_unless_present("location")
                .value_parser(clap::value_parser!(PathBuf))
                .long("wasm")
                .short('w')
                .help("a path on disk to the plugin"),
        );

    let uninstall_plugin = clap::Command::new("uninstall")
        .about("Uninstall a Modsurfer plugin.")
        .arg(
            Arg::new("identifier")
                .long("id")
                .help("the identifier of the plugin to uninstall"),
        );

    let plugin = clap::Command::new("plugin")
        .about("Manage your Modsurfer plugins")
        .subcommand(call_plugin)
        .subcommand(install_plugin)
        .subcommand(uninstall_plugin);

    [
        create, delete, get, list, search, validate, yank, audit, plugin,
    ]
    .into_iter()
    .map(add_output_arg)
    .chain(vec![generate, diff])
    .collect()
}
