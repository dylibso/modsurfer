use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use modsurfer;
use serde::Deserialize;
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize)]
struct Validation {
    pub validate: Check,
}

#[derive(Debug, Deserialize)]
struct Check {
    pub url: Option<String>,
    pub wasi: Option<bool>,
    pub imports: Option<Imports>,
}

#[derive(Debug, Deserialize)]
struct Namespace {
    pub contains: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Imports {
    pub contains: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub namespace: Option<Namespace>,
}

#[derive(Debug)]
struct Report {
    // k/v pair of the innermost validation field and failing value
    fails: BTreeMap<String, String>,
}

impl Report {
    fn new() -> Self {
        Self {
            fails: Default::default(),
        }
    }

    fn validate<T: PartialEq + ToString>(&mut self, name: &str, expected: T, actual: T) {
        if expected != actual {
            self.fails.insert(name.to_string(), actual.to_string());
        }
    }
}

pub async fn validate_module(file: &PathBuf, check: &PathBuf) -> Result<()> {
    let module = modsurfer::Module::new_from_file(file).await?;
    println!("module # exports: {}", module.exports.len());

    let mut buf = vec![];
    tokio::fs::File::open(check)
        .await?
        .read_to_end(&mut buf)
        .await?;

    let validation: Validation = serde_yaml::from_slice(&buf)?;

    let mut report = Report::new();

    // WASI
    if let Some(wasi) = validation.validate.wasi {
        if module
            .get_import_namespaces()
            .contains(&"wasi_snapshot_preview1")
        {
            report.validate("wasi", wasi, true)
        }
    }

    // Imports
    if let Some(imports) = validation.validate.imports {
        let import_func_names = module
            .imports
            .iter()
            .map(|im| im.func.name.clone())
            .collect::<Vec<_>>();

        let import_module_names = module.get_import_namespaces();

        if let Some(contains) = imports.contains {
            contains.iter().for_each(|name| {
                report.validate(
                    &format!("imports.contains.{}", name),
                    true,
                    import_func_names.contains(name),
                )
            })
        }

        if let Some(excludes) = imports.excludes {
            excludes.iter().for_each(|name| {
                report.validate(
                    &format!("imports.excludes.{}", name),
                    false,
                    import_func_names.contains(name),
                )
            })
        }

        if let Some(namespace) = imports.namespace {
            if let Some(contains) = namespace.contains {
                contains.iter().for_each(|name| {
                    report.validate(
                        &format!("imports.namespace.contains.{}", name),
                        true,
                        import_module_names.contains(&name.as_str()),
                    );
                })
            }

            if let Some(excludes) = namespace.excludes {
                excludes.iter().for_each(|name| {
                    report.validate(
                        &format!("imports.namespace.excludes.{}", name),
                        false,
                        import_module_names.contains(&name.as_str()),
                    )
                })
            }
        }
    }

    println!("{:#?}", report);

    Ok(())
}
