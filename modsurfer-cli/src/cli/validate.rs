use std::{collections::BTreeMap, fmt::Display, path::PathBuf, process::ExitCode};

use anyhow::Result;
use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Row, Table};
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
    pub allow_wasi: Option<bool>,
    pub imports: Option<Imports>,
    pub exports: Option<Exports>,
    pub size: Option<Size>,
}

#[derive(Debug, Deserialize)]
struct Namespace {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Imports {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub namespace: Option<Namespace>,
}

#[derive(Debug, Deserialize)]
struct Exports {
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub max: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct Size {
    pub max: Option<String>,
}

#[derive(Debug)]
enum Classification {
    AbiCompatibilty,
    ResourceLimit,
    Security,
}

impl Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Classification::AbiCompatibilty => f.write_str("ABI Compatibility")?,
            Classification::ResourceLimit => f.write_str("Resource Limit")?,
            Classification::Security => f.write_str("Security")?,
        }
        Ok(())
    }
}

#[derive(Debug)]
struct FailureDetail {
    actual: String,
    expected: String,
    severity: usize,
    classification: Classification,
}

#[derive(Debug)]
pub struct Report {
    /// k/v pair of the dot-separated path to validation field and expectation info
    fails: BTreeMap<String, FailureDetail>,
}

impl Report {
    pub fn as_exit_code(&self) -> ExitCode {
        match self.fails.len() {
            0 => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fails.is_empty() {
            f.write_str("All expectations met!\n")?;
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        table.set_header(vec![
            "Status",
            "Property",
            "Expected",
            "Actual",
            "Classification",
            "Severity",
        ]);

        self.fails.iter().for_each(|fail| {
            const SEVERITY_MAX: usize = 10;
            let severity = if fail.1.severity <= SEVERITY_MAX {
                fail.1.severity
            } else {
                SEVERITY_MAX
            };

            table.add_row(Row::from(vec![
                "FAIL",
                fail.0.as_str(),
                fail.1.expected.as_str(),
                fail.1.actual.as_str(),
                fail.1.classification.to_string().as_str(),
                "|".repeat(severity).as_str(),
            ]));
        });

        f.write_str(table.to_string().as_str())?;
        Ok(())
    }
}

impl Report {
    fn new() -> Self {
        Self {
            fails: Default::default(),
        }
    }

    fn validate_fn(
        &mut self,
        name: &str,
        expected: String,
        actual: String,
        valid: bool,
        severity: usize,
        classification: Classification,
    ) {
        if !valid {
            self.fails.insert(
                name.to_string(),
                FailureDetail {
                    actual,
                    expected,
                    severity,
                    classification,
                },
            );
        }
    }
}

struct Exist(bool);

impl Display for Exist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 {
            f.write_str("included")?;
        } else {
            f.write_str("excluded")?;
        }

        Ok(())
    }
}

pub async fn validate_module(file: &PathBuf, check: &PathBuf) -> Result<Report> {
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
    if let Some(allowed) = validation.validate.allow_wasi {
        let actual = module
            .get_import_namespaces()
            .contains(&"wasi_snapshot_preview1");
        report.validate_fn(
            "allow_wasi",
            allowed.to_string(),
            actual.to_string(),
            !(allowed == false && actual),
            10,
            Classification::AbiCompatibilty,
        );
    }

    // Imports
    if let Some(imports) = validation.validate.imports {
        let import_func_names = module
            .imports
            .iter()
            .map(|im| im.func.name.clone())
            .collect::<Vec<_>>();

        let import_module_names = module.get_import_namespaces();

        if let Some(include) = imports.include {
            include.iter().for_each(|name| {
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.include.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    8,
                    Classification::AbiCompatibilty,
                )
            });
        }

        if let Some(exclude) = imports.exclude {
            exclude.iter().for_each(|name| {
                let test = import_func_names.contains(name);
                report.validate_fn(
                    &format!("imports.exclude.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    test,
                    5,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(namespace) = imports.namespace {
            if let Some(include) = namespace.include {
                include.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.include.{}", name),
                        Exist(true).to_string(),
                        Exist(test).to_string(),
                        test,
                        8,
                        Classification::AbiCompatibilty,
                    );
                });
            }

            if let Some(exclude) = namespace.exclude {
                exclude.iter().for_each(|name| {
                    let test = import_module_names.contains(&name.as_str());
                    report.validate_fn(
                        &format!("imports.namespace.exclude.{}", name),
                        Exist(false).to_string(),
                        Exist(test).to_string(),
                        !test,
                        10,
                        Classification::AbiCompatibilty,
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
            let overage = num.saturating_sub(max);
            let max = if max == 0 { 1 } else { max };
            let severity = ((overage as f32 / max as f32) * 10.0).ceil() as usize;
            let test = num <= max;
            report.validate_fn(
                "exports.max",
                format!("<= {max}"),
                num.to_string(),
                test,
                severity,
                Classification::Security,
            );
        }

        if let Some(include) = exports.include {
            include.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.include.{}", name),
                    Exist(true).to_string(),
                    Exist(test).to_string(),
                    test,
                    10,
                    Classification::AbiCompatibilty,
                );
            });
        }

        if let Some(exclude) = exports.exclude {
            exclude.iter().for_each(|name| {
                let test = export_func_names.contains(name);
                report.validate_fn(
                    &format!("exports.exclude.{}", name),
                    Exist(false).to_string(),
                    Exist(test).to_string(),
                    !test,
                    5,
                    Classification::Security,
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
                (module.size / parsed) as usize,
                Classification::ResourceLimit,
            );
        }
    }

    // Complexity
    // TODO

    Ok(report)
}
