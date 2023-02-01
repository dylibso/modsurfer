# Modsurfer

![Modsurfer](.github/img/modsurfer-logo.svg)

> ### Modsurfer provides ops & dev teams with a system of record + diagnostics application to search, browse, and investigate modules.

For developers, SRE, DevOps, and engineering leaders: understand what the WebAssembly in your system is all about. Modsurfer provides critical information about WebAssembly code through a handy GUI, or directly at your fingertips via our CLI.

Use Modsurfer for:
- **at-a-glance insights** into WebAssembly module data (code size & complexity, imports/exports & more)
- **search for details** about modules (hash, ID, function names, strings, namespaces, errors & more)
- **off-the-shelf** System of Record: easily audit and track all the WebAssembly code in your stack
- **debug & triage issues** otherwise difficult to pinpoint in opaque WebAssembly format

### Modsurfer Desktop Application

The desktop application is **free** and available to download from [Dylibso](https://dylib.so) (_COMING SOON_! For insider build, email us: [hello@dylib.so](mailto:hello@dylib.so)), and is a useful debugging, visibility, and diagnostics tool for anyone working with WebAssembly. You can try a [demo version](https://demo.modsurfer.app) before downloading Modsurfer desktop. 

The demo is fully in-memory and has no HTTP API - don't rely on storing modules, or try to use it with the [Modsurfer CLI](https://github.com/dylibso/modsurfer#cli).

Download the desktop application if you want module persistence and would like to use the HTTP API.

![Modsurfer](.github/img/modsurfer-desktop.png)

### Enterprise

If you're running WebAssmebly code in production, you may be interested in using the Enterprise version of Modsurfer. Please reach out to us at [hello@dylib.so](mailto:hello@dylib.so) and we'll be happy to get you more information.

--- 

## In this repository

This is a collection of Rust crates and other code that is used for a variety of Modsurfer-related purposes. You can find a list and description of these below.

## `cli`

The Modsurfer CLI provides two primary functions:
1. HTTP Client to interace with Modsurfer (either the desktop app, or your Enterprise deployment)
2. Validation to ensure that WebAssembly binaries:
  - are compatible with various host ABIs
  - have no known security issues / comply with your policy
  - meet the runtime requirements in terms of size and code complexity (RAM/CPU limits, etc)

### Download the CLI

Modsurfer CLI can be downloaded via: 
- the latest [GitHub Release](https://github.com/dylibso/modsurfer/releases/latest)
- the GitHub container registry (for Docker environments)
  - `docker pull ghcr.io/dylibso/modsurfer:latest`

### Validate WebAssembly Modules

Modsurfer CLI provides a novel feature to process and validate a WebAssembly module based on policy/properties of the module that you define. In a "checkfile" (see the `mod.yaml` below), declare certain properties of a WebAssembly module and Modsurfer will verify that they are true, or report which properties of your binary are invalid.

```yaml 
validate:
  # simply require that a module can have WASI functionality or not
  allow_wasi: false
  
  # ensure that various imports and exports are included/exlcuded such that a module
  # will run properly in any host environment
  imports:
    include:
      - http_get
      - log_message
      - proc_exit
    exclude: 
      - fd_write
    namespace:
      include:
        - env
      exclude:
        - some_future_deprecated_module_name
        # phasing out old APIs? exclude the from acceptable namespaces/module names
        - wasi_snapshot_preview1

  exports: 
    # secure your modules by ensuring that there is no superfluous functionality hidden inside a binary
    max: 100
    include:
      - _start
      - bar
    exclude:
      - main
      - foo

  # use a human-readable module size to prevent overly large binaries from running in your environment
  size:
    max: 4MB
  # our Cyclomatic Complexity analysis can help prevent risk of CPU exhaustion from deteriorating your user experience and slowing down your system
  complexity:
    max_risk: low

```

You can also point to a remote check file to track up-to-date requirements: 
```yaml
validate:
  url: https://raw.githubusercontent.com/extism/extism/main/mod.yaml
```

### Usage

To run validation, you can use our [GitHub Action](https://github.com/dylibso/modsurfer-validate-action), or call the `validate` command directly: 

```
modsurfer validate -p path/to/my.wasm -c path/to/mod.yaml
```

If any of the expectations declared in your checkfile are invalid, Modsurfer will report them: 

```
┌────────┬──────────────────────────────────────────────────┬──────────┬──────────┬───────────────────┬────────────┐
│ Status │ Property                                         │ Expected │ Actual   │ Classification    │ Severity   │
╞════════╪══════════════════════════════════════════════════╪══════════╪══════════╪═══════════════════╪════════════╡
│ FAIL   │ allow_wasi                                       │ false    │ true     │ ABI Compatibility │ |||||||||| │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ complexity.max_risk                              │ <= low   │ medium   │ Resource Limit    │ |          │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ exports.exclude.main                             │ excluded │ included │ Security          │ |||||      │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ exports.include.bar                              │ included │ excluded │ ABI Compatibility │ |||||||||| │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ exports.max                                      │ <= 100   │ 151      │ Security          │ ||||||     │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ imports.include.http_get                         │ included │ excluded │ ABI Compatibility │ ||||||||   │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ imports.include.log_message                      │ included │ excluded │ ABI Compatibility │ ||||||||   │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ imports.namespace.exclude.wasi_snapshot_preview1 │ excluded │ included │ ABI Compatibility │ |||||||||| │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ imports.namespace.include.env                    │ included │ excluded │ ABI Compatibility │ ||||||||   │
├────────┼──────────────────────────────────────────────────┼──────────┼──────────┼───────────────────┼────────────┤
│ FAIL   │ size.max                                         │ <= 4MB   │ 4.4 MiB  │ Resource Limit    │ |          │
└────────┴──────────────────────────────────────────────────┴──────────┴──────────┴───────────────────┴────────────┘
```

Find more information about the CLI in its dedicated [README](./cli/README.md), or download the tool and run `modsurfer -h`. 

### Testing the CLI

From the root of the repo, run the following to see a basic validation report:
- `make test-cli`
- `make empty-cli`
- `make unknown-cli`

`test/` contains a `mod.yaml`, which declares expected properties of a WebAssembly module, as well as a `spidermonkey.wasm` file to use as example input to use for the validation.

`wasm/` contains a set of WebAssembly binaries downloaded from the [`wapm`](https://wapm.io) package manager used for analysis and testing.

---

## `proto`

This directory contains the Protobuf definitions for the types used in the API. Messages have various levels of documentation as well as endpoints if they are request types. Use the `api.proto` to generate a language client if you'd like to interact with Modsurfer API programmatically from your application.