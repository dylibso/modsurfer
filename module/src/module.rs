use std::collections::{HashMap, HashSet};

use crate::{Function, SourceLanguage};

use serde;
use url;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Import {
    pub module_name: String,
    pub func: Function,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Export {
    pub func: Function,
}

/// A description of a wasm module extracted from the binary, encapsulating
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Module {
    /// sha256 hash of the modules raw bytes
    pub hash: String,
    /// function imports called by the module (see: <https://github.com/WebAssembly/design/blob/main/Modules.md#imports>)
    pub imports: Vec<Import>,
    /// function exports provided by the module (see: <https://github.com/WebAssembly/design/blob/main/Modules.md#exports>)
    pub exports: Vec<Export>,
    /// size in bytes of the module
    pub size: u64,
    /// path or locator to the module
    pub location: String,
    /// programming language used to produce this module
    pub source_language: SourceLanguage,
    /// arbitrary metadata provided by the operator of this module
    pub metadata: Option<HashMap<String, String>>,
    /// timestamp when this module was loaded and stored
    #[cfg(feature = "api")]
    pub inserted_at: chrono::DateTime<chrono::Utc>,
    #[cfg(not(feature = "api"))]
    pub inserted_at: u64,
    /// the interned strings stored in the wasm binary (panic/abort messages, etc.)
    pub strings: Vec<String>,
    /// cyclomatic complexity of the module
    pub complexity: Option<u32>,
    /// the graph in Dot format
    pub graph: Option<Vec<u8>>,
    /// function hashes
    pub function_hashes: HashMap<String, String>,
}

impl Module {
    // TODO: also add memory imports and other items of interest
    /// return the namespaces from which this module imports functions
    pub fn get_import_namespaces(&self) -> Vec<&str> {
        self.imports
            .iter()
            .fold(HashSet::new(), |mut acc, import| {
                acc.insert(import.module_name.as_str());
                acc
            })
            .into_iter()
            .collect()
    }
}

impl Default for Module {
    fn default() -> Self {
        Module {
            hash: String::new(),
            imports: vec![],
            exports: vec![],
            size: 0,
            location: String::new(),
            source_language: SourceLanguage::Unknown,
            metadata: None,
            #[cfg(feature = "chrono")]
            inserted_at: chrono::Utc::now(),
            #[cfg(not(feature = "chrono"))]
            inserted_at: 0,
            strings: vec![],
            complexity: None,
            graph: None,
            function_hashes: HashMap::new(),
        }
    }
}

impl Module {
    pub fn file_name(&self) -> String {
        std::path::Path::new(self.location_url().path())
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned()
    }

    pub fn location_url(&self) -> url::Url {
        url::Url::parse(self.location.as_str()).expect("Invalid location")
    }
}
