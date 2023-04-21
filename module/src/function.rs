#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Function {
    pub name: String,
    pub ty: FunctionType,
}

#[derive(Debug, Default, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FunctionType {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

/// Represents the types of values in a WebAssembly module.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ValType {
    /// The value type is i32.
    I32,
    /// The value type is i64.
    I64,
    /// The value type is f32.
    F32,
    /// The value type is f64.
    F64,
    /// The value type is v128.
    V128,
    /// The value type is a function reference.
    FuncRef,
    /// The value type is an extern reference.
    ExternRef,
}

impl ValType {
    /// Returns whether this value type is a "reference type".
    ///
    /// Only reference types are allowed in tables, for example, and with some
    /// instructions. Current reference types include `funcref` and `externref`.
    pub fn is_reference_type(&self) -> bool {
        matches!(self, ValType::FuncRef | ValType::ExternRef)
    }
}

impl From<wasmparser::ValType> for ValType {
    fn from(v: wasmparser::ValType) -> Self {
        use wasmparser::ValType as V;
        match v {
            V::I32 => ValType::I32,
            V::I64 => ValType::I64,
            V::F32 => ValType::F32,
            V::F64 => ValType::F64,
            V::V128 => ValType::V128,
            V::Ref(wasmparser::RefType::FUNCREF) => ValType::FuncRef,
            V::Ref(wasmparser::RefType::EXTERNREF) => ValType::FuncRef,
            V::Ref(r) => panic!("Unknown ref type: {:?}", r),
        }
    }
}

impl From<ValType> for wasmparser::ValType {
    fn from(v: ValType) -> Self {
        use wasmparser::ValType as V;
        match v {
            ValType::I32 => V::I32,
            ValType::I64 => V::I64,
            ValType::F32 => V::F32,
            ValType::F64 => V::F64,
            ValType::V128 => V::V128,
            ValType::FuncRef => V::FUNCREF,
            ValType::ExternRef => V::EXTERNREF,
        }
    }
}

impl<'a> From<&'a wasmparser::FuncType> for FunctionType {
    fn from(ft: &'a wasmparser::FuncType) -> Self {
        Self {
            params: ft.params().iter().cloned().map(ValType::from).collect(),
            results: ft.results().iter().cloned().map(ValType::from).collect(),
        }
    }
}
