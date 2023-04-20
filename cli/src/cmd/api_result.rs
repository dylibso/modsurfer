use std::fmt::Display;

use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Row, Table};
use modsurfer_module::SourceLanguage;
use serde::{ser::SerializeStruct, Serialize};

#[derive(Serialize)]
pub struct ApiResults<'a> {
    pub results: Vec<ApiResult<'a>>,
}

pub struct ApiResult<'a> {
    pub module_id: i64,
    pub hash: String,
    pub file_name: String,
    pub exports: usize,
    pub imports: usize,
    pub namespaces: Vec<&'a str>,
    pub source_language: SourceLanguage,
    pub size: String,
}

#[derive(Serialize)]
pub struct SimpleApiResults {
    pub results: Vec<SimpleApiResult>,
}

pub struct SimpleApiResult {
    pub module_id: i64,
    pub hash: String,
}

impl<'a> Serialize for ApiResult<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ApiResult", 8)?;
        state.serialize_field("module_id", &self.module_id.to_string())?;
        state.serialize_field("hash", &self.hash)?;
        state.serialize_field("file_name", &self.file_name)?;
        state.serialize_field("exports", &self.exports)?;
        state.serialize_field("imports", &self.imports)?;
        state.serialize_field("namespaces", &self.namespaces)?;
        state.serialize_field("source_language", &self.source_language)?;
        state.serialize_field("size", &self.size)?;
        state.end()
    }
}

impl Serialize for SimpleApiResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SimpleApiResult", 8)?;
        state.serialize_field("module_id", &self.module_id.to_string())?;
        state.serialize_field("hash", &self.hash)?;
        state.end()
    }
}

impl Display for SimpleApiResults {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        table.set_header(vec!["ID", "Hash"]);

        self.results.iter().for_each(|m| {
            table.add_row(Row::from(vec![m.module_id.to_string(), m.hash.clone()]));
        });

        f.write_str(table.to_string().as_str())
    }
}

impl Display for ApiResults<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table.load_preset(UTF8_FULL);
        table.apply_modifier(UTF8_SOLID_INNER_BORDERS);
        table.set_header(vec![
            "ID",
            "Hash",
            "Filename",
            "# Exports",
            "# Imports",
            "Namespaces",
            "Source",
            "Size",
        ]);

        if self.results.is_empty() {
            return f.write_str(table.to_string().as_str());
        }

        self.results.iter().for_each(|m| {
            table.add_row(Row::from(vec![
                m.module_id.to_string(),
                m.hash[0..6].to_string(),
                m.file_name.clone(),
                m.exports.to_string(),
                m.imports.to_string(),
                m.namespaces.join(", "),
                m.source_language.to_string(),
                m.size.clone(),
            ]));
        });

        f.write_str(table.to_string().as_str())
    }
}
