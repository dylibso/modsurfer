use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use human_bytes::human_bytes;
use parse_size::parse_size;
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
    pub exports: Option<Exports>,
    pub size: Option<Size>,
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

#[derive(Debug, Deserialize)]
struct Exports {
    pub contains: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub max: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Size {
    pub max: Option<String>,
}

#[derive(Debug)]
struct Report {
    // k/v pair of the dot-separated path to validation field and expectation info
    fails: BTreeMap<String, String>,
}

impl Report {
    fn new() -> Self {
        Self {
            fails: Default::default(),
        }
    }

    fn validate_fn(&mut self, name: &str, expects: &str, f: &dyn Fn() -> bool) {
        if !f() {
            self.fails.insert(name.to_string(), expects.to_string());
        }
    }
}

pub async fn validate_module(file: &PathBuf, check: &PathBuf) -> Result<()> {
    let module = modsurfer::Module::new_from_file(file).await?;

    let mut buf = vec![];
    tokio::fs::File::open(check)
        .await?
        .read_to_end(&mut buf)
        .await?;

    let validation: Validation = serde_yaml::from_slice(&buf)?;
    if let Some(url) = validation.validate.url {
        // fetch remote validation file
        println!("Fetching validation schema from URL: {}", url);

        // parse the file again & reassign `validation`
    }

    let mut report = Report::new();

    // WASI
    if let Some(wasi) = validation.validate.wasi {
        if module
            .get_import_namespaces()
            .contains(&"wasi_snapshot_preview1")
        {
            report.validate_fn("wasi", "true", &|| wasi == true);
        } else {
            report.validate_fn("wasi", "false", &|| wasi == false);
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
                report.validate_fn(&format!("imports.contains.{}", name), "exist", &|| {
                    import_func_names.contains(name)
                })
            })
        }

        if let Some(excludes) = imports.excludes {
            excludes.iter().for_each(|name| {
                report.validate_fn(&format!("imports.excludes.{}", name), "not exist", &|| {
                    !import_func_names.contains(name)
                })
            })
        }

        if let Some(namespace) = imports.namespace {
            if let Some(contains) = namespace.contains {
                contains.iter().for_each(|name| {
                    report.validate_fn(
                        &format!("imports.namespace.contains.{}", name),
                        "exist",
                        &|| import_module_names.contains(&name.as_str()),
                    );
                })
            }

            if let Some(excludes) = namespace.excludes {
                excludes.iter().for_each(|name| {
                    report.validate_fn(
                        &format!("imports.namespace.excludes.{}", name),
                        "not exist",
                        &|| !import_module_names.contains(&name.as_str()),
                    )
                })
            }
        }
    }

    // Exports
    if let Some(exports) = validation.validate.exports {
        let export_func_names = module
            .exports
            .iter()
            .map(|exp| exp.func.name.clone())
            .collect::<Vec<_>>();
        if let Some(max) = exports.max {
            let num = export_func_names.len() as u32;
            report.validate_fn("exports.max", &format!("{} > {}", num, max), &|| num <= max)
        }

        if let Some(contains) = exports.contains {
            contains.iter().for_each(|name| {
                report.validate_fn(&format!("exports.contains.{}", name), "exist", &|| {
                    export_func_names.contains(name)
                });
            })
        }

        if let Some(excludes) = exports.excludes {
            excludes.iter().for_each(|name| {
                report.validate_fn(&format!("exports.excludes.{}", name), "not exist", &|| {
                    !export_func_names.contains(name)
                })
            })
        }
    }

    // Size
    if let Some(size) = validation.validate.size {
        if let Some(max) = size.max {
            let parsed = parse_size(&max).unwrap();
            let human_actual = human_bytes(module.size as f64);
            report.validate_fn("size.max", &format!("{} > {}", human_actual, max), &|| {
                module.size <= parsed
            })
        }
    }

    println!("{:#?}", report);

    Ok(())
}
