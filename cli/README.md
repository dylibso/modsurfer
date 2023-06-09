# Modsurfer CLI

Produces a binary called `modsurfer` which has the following usage:

```sh
Copyright Dylibso, Inc. <support@dylib.so>

Modsurfer CLI is used to interact with the HTTP API or validate modules offline.

Usage: modsurfer [COMMAND]

Commands:
  create
          Create a new entry for a module.
  delete
          Delete a module and its versions.
  get
          Get a module by its ID.
  list
          List all modules, paginated by the `offset` and `limit` parameters or their defaults.
  search
          Search for modules matching optional parameters.
  validate
          Validate a module using a module checkfile.
  yank
          Mark a module version as yanked (unavailable).
  audit
           Return a list of modules which violate requirements in the provided checkfile.
  generate
          Generate a starter checkfile from the given module.
  diff
          Compare two modules
  help
          Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help

  -V, --version
          Print version
```

## Output Format

Most commands that generate output can take an optional argument `--output-format` to instruct Modsurfer CLI to render `JSON` instead of a table. This can be very useful if Modsurfer CLI is part of a pipeline or script. 

e.g.

```sh
modsurfer get --id 4 | jq . | ...
```

## Examples:

```sh
# all commands will default to an API host of http://localhost:1739 unless otherwise set via 
# environment variable: `MODSURFER_BASE_URL`

modsurfer create \
        -p my.wasm \
        -c mod.yaml \ # optional - validate before creating an entry in Modsurfer
        -l file:///wasm/my.wasm \
        -m userid=12234 -m app=33 # optional - associate searchable key-value metadata with a module

modsurfer delete --id 3 --id 4 --id 5

modsurfer get --id 3

modsurfer list --offset 0 --limit 50 # (0 & 50 are defaults)

modsurfer search --function-name _start --module-name env --source-language Rust --text "Help me"

modsurfer generate -p spidermonkey.wasm -o mod.yaml

modsurfer diff a.wasm b.wasm # or diff using Modsurfer module IDs

modsurfer audit --outcome pass -c mod.yaml
```

> **NOTE:** when using the `search` command along with the `--source-language` argument, the value is case-sensitive, being one of `{Rust, Go, C, C++, AssemblyScript}`.