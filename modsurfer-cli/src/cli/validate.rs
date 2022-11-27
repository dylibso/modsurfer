use std::{collections::BTreeMap, path::PathBuf};

use anyhow::Result;
use modsurfer;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize, Serialize)]
struct Validation {
    pub validate: Check,
}

#[derive(Debug, Deserialize, Serialize)]
struct Check {
    pub url: Option<String>,
    pub wasi: Option<bool>,
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
    println!("check: {:#?}", validation);

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

    println!("{:#?}", report);

    Ok(())
}
