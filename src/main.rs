use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;

mod cli;
use cli::{Cli, Hash, Id, Limit, Offset, Version};

fn main() {
    let cmd = Command::new("modsurfer")
        .arg(
            Arg::new("host")
                .value_parser(clap::value_parser!(url::Url))
                .long("host")
                .help("Define the API host where Modsurfer CLI should connect.")
                .value_name("BASE_URL")
                .default_value("http://localhost:1739"),
        )
        .about("Modsurfer CLI is used to interact with the HTTP API.")
        .before_help("Copyright Dylibso, Inc. <support@dylib.so>")
        .subcommands(make_subcommands());

    Cli::new(cmd).execute();
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

    vec![create, delete, get, list, search, yank]
}
