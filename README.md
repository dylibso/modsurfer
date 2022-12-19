# Modsurfer

Collection of crates related to Modsurfer. 

## `modsurfer-cli`

From the root of the repo, run the following to see a basic validation report:
- `make test-cli`

`test/` contains a `mod.yaml`, which declares expected properties of a .wasm module, as well as a
`spidermonkey.wasm` file to use as example input to use for the validation.