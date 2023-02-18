use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use human_bytes::human_bytes;
use modsurfer_module::Module;
use serde_yaml;

use crate::cmd::validate::{
    Complexity, Exports, FunctionItem, ImportItem, Imports, Module as ModuleParser, Namespace,
    NamespaceItem, RiskLevel, Size, Validation,
};

pub async fn checkfile_from_module(wasm: &PathBuf, output: &PathBuf) -> Result<()> {
    let module_data = tokio::fs::read(wasm).await?;
    let module = ModuleParser::parse(&module_data)?;

    let validation = generate_checkfile(&module).await?;
    let mut file = File::create(output)?;
    writeln!(
        &mut file,
        "# For more information about other checkfile options, see the documentation at https://dev.dylib.so/docs/modsurfer/cli#checkfile"
    )?;
    serde_yaml::to_writer(&file, &validation)?;

    Ok(())
}

async fn generate_checkfile(module: &Module) -> Result<Validation> {
    let mut validation = Validation::default();
    let namespaces = module.get_import_namespaces();

    // allow_wasi
    if namespaces.contains(&"wasi_snapshot_preview1") {
        validation.validate.allow_wasi = Some(true);
    }

    // imports (add all to include + namespace)
    let mut imports = Imports::default();
    let mut include_imports = vec![];
    module.imports.iter().for_each(|imp| {
        include_imports.push(ImportItem::Item {
            namespace: Some(imp.module_name.clone()),
            name: imp.func.name.clone(),
            params: Some(imp.func.ty.args.clone()),
            results: Some(imp.func.ty.returns.clone()),
        });
    });
    imports.include = Some(include_imports);

    // imports.namespace (add all to imports)
    let mut namespace = Namespace::default();
    namespace.include = Some(
        namespaces
            .iter()
            .map(|name| NamespaceItem::Name(name.to_string()))
            .collect::<Vec<_>>(),
    );
    if !namespaces.is_empty() {
        imports.namespace = Some(namespace);
    }

    // exports (add all exports)
    let mut exports = Exports::default();
    let mut include_exports = vec![];
    module.exports.iter().for_each(|exp| {
        include_exports.push(FunctionItem::Item {
            name: exp.func.name.clone(),
            params: Some(exp.func.ty.args.clone()),
            results: Some(exp.func.ty.returns.clone()),
        });
    });
    let export_count = include_exports.len();
    exports.include = Some(include_exports);

    // exports.max (match number of exports)
    exports.max = Some(export_count as u32);

    // size.max (use size from module)
    let mut size = Size::default();
    size.max = Some(human_bytes(module.size as f64));

    // complexity.max_risk (use complexity)
    let mut complexity = Complexity::default();
    complexity.max_risk = Some(RiskLevel::from(module.complexity.unwrap_or_default()));

    validation.validate.url = None;
    validation.validate.imports = Some(imports);
    validation.validate.exports = Some(exports);
    validation.validate.size = Some(size);
    validation.validate.complexity = Some(complexity);

    Ok(validation)
}
