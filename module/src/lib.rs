pub use anyhow::Error;

mod function;
mod module;
mod source_language;

pub use function::{Function, FunctionType, ValType};
pub use module::{Export, Import, Module};
pub use source_language::SourceLanguage;
