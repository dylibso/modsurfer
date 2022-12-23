use crate::*;

use modsurfer::{Export, Import};

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

pub fn val_types(v: Vec<modsurfer::ValType>) -> Vec<protobuf::EnumOrUnknown<api::ValType>> {
    v.into_iter()
        .map(|x| protobuf::EnumOrUnknown::new(val_type(x)))
        .collect()
}

pub fn val_type(v: modsurfer::ValType) -> api::ValType {
    match v {
        modsurfer::ValType::I32 => api::ValType::I32,
        modsurfer::ValType::I64 => api::ValType::I64,
        modsurfer::ValType::F32 => api::ValType::F32,
        modsurfer::ValType::F64 => api::ValType::F64,
        modsurfer::ValType::V128 => api::ValType::V128,
        modsurfer::ValType::FuncRef => api::ValType::FuncRef,
        modsurfer::ValType::ExternRef => api::ValType::ExternRef,
    }
}

#[cfg(feature = "api")]
pub fn module(module: modsurfer::Module, id: i64) -> api::Module {
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

    dest
}

#[cfg(not(feature = "api"))]
pub fn module(module: modsurfer::Module, id: i64) -> api::Module {
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

    dest
}

pub fn exports(exports: Vec<Export>) -> Vec<api::Export> {
    exports
        .into_iter()
        .map(|e| {
            let func = api::Function {
                name: e.func.name,
                args: to_api::val_types(e.func.ty.args),
                returns: to_api::val_types(e.func.ty.returns),
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
                args: to_api::val_types(i.func.ty.args),
                returns: to_api::val_types(i.func.ty.returns),
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
