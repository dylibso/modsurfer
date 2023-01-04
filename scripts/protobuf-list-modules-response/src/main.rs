use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use modsurfer_convert::{
    api::{ListModulesResponse, Module as ProtoModule},
    to_api,
};
use modsurfer_module::Module;
use modsurfer::ModuleParser;
use protobuf::Message;

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<String>>();
    let mut take = args[0].parse::<i32>().unwrap_or_else(|_| -1);
    println!("taking {} modules", take);
    // collect all the files in the wasm directory (expects these to be .wasm modules)
    let wasm_dir = PathBuf::new()
        .join(env!("CARGO_WORKSPACE_DIR"))
        .join("wasm");
    let dir = fs::read_dir(wasm_dir)?;
    let files = dir
        .filter_map(|entry| {
            entry.ok().and_then(|entry| {
                Some(entry).and_then(|entry| match entry.metadata() {
                    Ok(m) if !m.is_dir() => Some(entry),
                    _ => None,
                })
            })
        })
        .collect::<Vec<_>>();

    if take < 0 {
        take = files.len() as i32;
    }

    // parse .wasm into modsurfer Modules
    let mut modules = vec![];

    for file in files.iter().take(take as usize) {
        println!("reading: {:?}", file.path());
        let m = Module::new_from_file(file.path()).await?;
        modules.push(m);
    }

    let total = modules.len() as u64;

    // give each module an ID, and convert it to the protobuf Module message, collect all
    let modules = modules
        .into_iter()
        .zip(1i64..total as i64)
        .map(|(m, id)| to_api::module(m, id))
        .collect::<Vec<ProtoModule>>();

    // construct a protobuf ListModulesResponse message
    let output = ListModulesResponse {
        modules,
        total,
        ..Default::default()
    };

    // serialize the protobuf message and write to a file on disk
    let data = output.write_to_bytes()?;
    fs::write("ListModulesResponse.pb", &data)?;

    println!("wrote ListModulesResponse.pb with {} modules.", total);

    Ok(())
}
