/// Detected from the `producers` section in the wasm binary, or from other implicit values within
/// the wasm binary.
/// See more: https://github.com/WebAssembly/tool-conventions/blob/main/ProducersSection.md
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SourceLanguage {
    Unknown,
    Rust,
    Go,
    C,
    Cpp,
    AssemblyScript,
}

impl From<String> for SourceLanguage {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Rust" => SourceLanguage::Rust,
            "Go" => SourceLanguage::Go,
            "C" => SourceLanguage::C,
            "C++" => SourceLanguage::Cpp,
            "AssemblyScript" => SourceLanguage::AssemblyScript,
            _ => SourceLanguage::Unknown,
        }
    }
}

impl std::fmt::Display for SourceLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SourceLanguage::Unknown => "Unknown",
            SourceLanguage::Rust => "Rust",
            SourceLanguage::Go => "Go",
            SourceLanguage::C => "C",
            SourceLanguage::Cpp => "C++",
            SourceLanguage::AssemblyScript => "AssemblyScript",
        };

        f.write_str(s)
    }
}

impl Default for SourceLanguage {
    fn default() -> Self {
        SourceLanguage::Unknown
    }
}
