use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use serde_yaml;

use modsurfer_validation::{generate_checkfile, Module as ModuleParser};

pub async fn checkfile_from_module(wasm: &PathBuf, output: &PathBuf) -> Result<()> {
    let module_data = tokio::fs::read(wasm).await?;
    let module = ModuleParser::parse(&module_data)?;
    println!("{:?}", module);
    let validation = generate_checkfile(&module)?;
    let mut file = File::create(output)?;
    writeln!(
        &mut file,
        "# For more information about other checkfile options, see the documentation at https://dev.dylib.so/docs/modsurfer/cli#checkfile"
    )?;
    serde_yaml::to_writer(&file, &validation)?;

    Ok(())
}
