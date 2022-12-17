use std::{collections::BTreeMap, fmt::Display, path::PathBuf};

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
struct FailureDetail {
    actual: String,
    expected: String,
}

#[derive(Debug)]
struct Report {
    // k/v pair of the dot-separated path to validation field and expectation info
    fails: BTreeMap<String, FailureDetail>,
}

// TODO: change this to implement some table view
impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = f.write_str("Modsurfer Validation Report\n");
        let _ = f.write_str("---------------------------\n");

        if self.fails.is_empty() {
            let _ = f.write_str("All expectations met!\n");
            return Ok(());
        }

        self.fails.iter().for_each(|fail| {
            let _ = f.write_str(&format!(
                "'{}': expected: '{}', actual: '{}'\n",
                fail.0, fail.1.expected, fail.1.actual
            ));
        });

        Ok(())
    }
}

impl Report {
    fn new() -> Self {
        Self {
            fails: Default::default(),
        }
    }

    fn validate_fn(&mut self, name: &str, expected: String, actual: String, valid: bool) {
        if !valid {
            self.fails
                .insert(name.to_string(), FailureDetail { actual, expected });
        }
    }
}

struct Exist(bool);

impl Display for Exist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 {
            let _ = f.write_str("exist");
        } else {
            let _ = f.write_str("not exist");
        }

        Ok(())
    }
}

pub async fn validate_module(file: &PathBuf, check: &PathBuf) -> Result<()> {
    let module = modsurfer::Module::new_from_file(file).await?;

    let mut buf = vec![];
    tokio::fs::File::open(check)
        .await?
        .read_to_end(&mut buf)
        .await?;

    let mut validation: Validation = serde_yaml::from_slice(&buf)?;
    if let Some(url) = validation.validate.url {
        // fetch remote validation file
        println!("Fetching validation schema from URL: {}", url);

        let resp = reqwest::get(&url).await?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "Failed to make request for remote validation schema: {}",
                url
            );
        }

        buf.clear();
        buf = resp.bytes().await?.into();

        // parse the file again & reassign `validation`
        validation = serde_yaml::from_slice(&buf)?;
    }

    let mut report = Report::new();

    // WASI
    if let Some(wasi) = validation.validate.wasi {
        let actual = module
            .get_import_namespaces()
            .contains(&"wasi_snapshot_preview1");
        // if module
        //     .get_import_namespaces()
        //     .contains(&"wasi_snapshot_preview1")
        // {
        //     report.validate_fn("wasi", "true", &|| wasi == true);
        // } else {
        //     report.validate_fn("wasi", "false", &|| wasi == false);
        // }
        report.validate_fn("wasi", wasi.to_string(), actual.to_string(), wasi == actual);
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
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.contains.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                )
            });
        }

        if let Some(excludes) = imports.excludes {
            excludes.iter().for_each(|name| {
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.excludes.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    test,
                );
            });
        }

        if let Some(namespace) = imports.namespace {
            if let Some(contains) = namespace.contains {
                contains.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.contains.{}", name),
                        Exist(true).to_string(),
                        Exist(test).to_string(),
                        test,
                    );
                });
            }

            if let Some(excludes) = namespace.excludes {
                excludes.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.excludes.{}", name),
                        Exist(false).to_string(),
                        Exist(test).to_string(),
                        !test,
                    )
                });
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
            let test = num <= max;
            report.validate_fn("exports.max", format!("<= {max}"), num.to_string(), test)
        }

        if let Some(contains) = exports.contains {
            contains.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.contains.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                );
            });
        }

        if let Some(excludes) = exports.excludes {
            excludes.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.excludes.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    !test,
                );
            });
        }
    }

    // Size
    if let Some(size) = validation.validate.size {
        if let Some(max) = size.max {
            let parsed = parse_size(&max).unwrap();
            let human_actual = human_bytes(module.size as f64);
            let test = module.size <= parsed;
            report.validate_fn(
                "size.max",
                format!("<= {max}"),
                human_actual.to_string(),
                test,
            );
        }
    }

    println!("{report}");

    Ok(())
}
