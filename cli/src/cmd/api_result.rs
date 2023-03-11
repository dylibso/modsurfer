use std::fmt::Display;

use comfy_table::{modifiers::UTF8_SOLID_INNER_BORDERS, presets::UTF8_FULL, Row, Table};
use modsurfer_module::SourceLanguage;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResults<'a> {
    pub results: Vec<ApiResult<'a>>,
}

#[derive(Serialize)]
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

#[derive(Serialize)]
pub struct SimpleApiResult {
    pub module_id: i64,
    pub hash: String,
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
