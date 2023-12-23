use std::ffi::OsString;

/// Detected from the `producers` section in the wasm binary, or from other implicit values within
/// the wasm binary.
/// See more: <https://github.com/WebAssembly/tool-conventions/blob/main/ProducersSection.md>
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SourceLanguage {
    Unknown,
    Rust,
    Go,
    C,
    Cpp,
    AssemblyScript,
    Swift,
    JavaScript,
    Haskell,
    Zig,
}

impl From<String> for SourceLanguage {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Rust" => SourceLanguage::Rust,
            "Go" => SourceLanguage::Go,
            "C" => SourceLanguage::C,
            "C++" => SourceLanguage::Cpp,
            "AssemblyScript" => SourceLanguage::AssemblyScript,
            "Swift" => SourceLanguage::Swift,
            "JavaScript" => SourceLanguage::JavaScript,
            "Haskell" => SourceLanguage::Haskell,
            "Zig" => SourceLanguage::Zig,
            _ => SourceLanguage::Unknown,
        }
    }
}

impl From<OsString> for SourceLanguage {
    fn from(value: OsString) -> Self {
        let s = value.into_string().unwrap_or_default();
        s.into()
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
            SourceLanguage::Swift => "Swift",
            SourceLanguage::JavaScript => "JavaScript",
            SourceLanguage::Haskell => "Haskell",
            SourceLanguage::Zig => "Zig",
        };

        f.write_str(s)
    }
}

impl Default for SourceLanguage {
    fn default() -> Self {
        SourceLanguage::Unknown
    }
}
