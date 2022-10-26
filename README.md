# Modsurfer CLI

Produces a binary called `modsurfer` which has the following usage:

```sh
Modsurfer CLI is used to interact with the HTTP API.

Usage: modsurfer [OPTIONS] [COMMAND]

Commands:
  create  Create a new entry for a module.
  delete  Delete a module and its versions.
  get     Get a module by its ID.
  list    List all modules, paginated by the `offset` and `limit` parameters or their defaults.
  search  Search for modules matching optional parameters.
  yank    Mark a module version as unavailable.
  help    Print this message or the help of the given subcommand(s)

Options:
      --host <BASE_URL>  Define the API host where Modsurfer CLI should connect. [default: http://localhost:1739]
  -h, --help             Print help information

```


## Examples:

```sh
# all commands will default to a `--host` param of http://localhost:1739 unless otherwise set via:
# modsurfer --host <BASE_URL> {create|delete|...}

modsurfer create \
        --path my.wasm \
        --validate mod.yaml \
        --location file:///wasm/my.wasm \
        --metadata userid=12234 --metadata app=33

modsurfer delete --id 3 --id 4 --id 5

modsurfer yank --id 3 --version 0.4.1

modsurfer get --id 3

modsurfer list --offset 0 --limit 50 # (0 & 50 are defaults)

modsurfer search --function_name _start --module_name env --source_language Rust --hash 121eee... --strings "Help me"

```