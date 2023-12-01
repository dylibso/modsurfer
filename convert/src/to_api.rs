use crate::*;

use modsurfer_module::{Export, Import, Module, ValType};

pub fn source_language(src: SourceLanguage) -> api::SourceLanguage {
    match src {
        SourceLanguage::Unknown => api::SourceLanguage::Unknown,
        SourceLanguage::AssemblyScript => api::SourceLanguage::AssemblyScript,
        SourceLanguage::C => api::SourceLanguage::C,
        SourceLanguage::Cpp => api::SourceLanguage::Cpp,
        SourceLanguage::Go => api::SourceLanguage::Go,
        SourceLanguage::Rust => api::SourceLanguage::Rust,
    }
}

pub fn val_types(v: Vec<ValType>) -> Vec<protobuf::EnumOrUnknown<api::ValType>> {
    v.into_iter()
        .map(|x| protobuf::EnumOrUnknown::new(val_type(x)))
        .collect()
}

pub fn val_type(v: ValType) -> api::ValType {
    match v {
        ValType::I32 => api::ValType::I32,
        ValType::I64 => api::ValType::I64,
        ValType::F32 => api::ValType::F32,
        ValType::F64 => api::ValType::F64,
        ValType::V128 => api::ValType::V128,
        ValType::FuncRef => api::ValType::FuncRef,
        ValType::ExternRef => api::ValType::ExternRef,
        // the remainder of the types are only applicable for components
        _ => api::ValType::ExternRef, // todo: handle this better
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn module(module: Module, id: i64) -> api::Module {
    let mut dest = api::Module::new();
    dest.id = id;
    dest.location = module.location;
    dest.hash = module.hash;
    dest.metadata = module.metadata.unwrap_or_default();
    dest.size = module.size as u64;
    dest.source_language = protobuf::EnumOrUnknown::new(source_language(module.source_language));
    dest.inserted_at =
        protobuf::MessageField::some(protobuf::well_known_types::timestamp::Timestamp {
            seconds: module.inserted_at.timestamp(),
            nanos: module.inserted_at.timestamp_subsec_nanos() as i32,
            special_fields: protobuf::SpecialFields::new(),
        });

    dest.exports = exports(module.exports);
    dest.imports = imports(module.imports);
    dest.strings = module.strings;
    dest.complexity = module.complexity;
    dest.function_hashes = module.function_hashes;

    dest
}

#[cfg(target_arch = "wasm32")]
pub fn module(module: Module, id: i64) -> api::Module {
    let mut dest = api::Module::new();
    dest.id = id;
    dest.location = module.location;
    dest.hash = module.hash;
    dest.metadata = module.metadata.unwrap_or_default();
    dest.size = module.size as u64;
    dest.source_language = protobuf::EnumOrUnknown::new(source_language(module.source_language));
    dest.exports = exports(module.exports);
    dest.imports = imports(module.imports);
    dest.strings = module.strings;
    dest.complexity = module.complexity;
    dest.function_hashes = module.function_hashes;
    dest
}

pub fn exports(exports: Vec<Export>) -> Vec<api::Export> {
    exports
        .into_iter()
        .map(|e| {
            let func = api::Function {
                name: e.func.name,
                params: to_api::val_types(e.func.ty.params),
                results: to_api::val_types(e.func.ty.results),
                ..Default::default()
            };

            api::Export {
                func: protobuf::MessageField::some(func),
                ..Default::default()
            }
        })
        .collect()
}

pub fn imports(imports: Vec<Import>) -> Vec<api::Import> {
    imports
        .into_iter()
        .map(|i| {
            let func = api::Function {
                name: i.func.name,
                params: to_api::val_types(i.func.ty.params),
                results: to_api::val_types(i.func.ty.results),
                ..Default::default()
            };

            api::Import {
                func: protobuf::MessageField::some(func),
                module_name: i.module_name,
                ..Default::default()
            }
        })
        .collect()
}
