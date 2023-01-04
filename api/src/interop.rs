use modsurfer_convert::{api, from_api};
use modsurfer_module::Module;

#[derive(Default, PartialEq)]
pub struct List<T> {
    inner: Vec<T>,
    total: u32,
    offset: u32,
    limit: u32,
}

impl<T> List<T> {
    pub fn new(inner: Vec<T>, total: u32, offset: u32, limit: u32) -> Self {
        Self {
            inner,
            total,
            offset,
            limit,
        }
    }

    pub fn vec(&self) -> Vec<&T> {
        self.inner.iter().map(|t| t).collect::<Vec<&T>>()
    }

    pub fn split(&self) -> (Vec<&T>, u32, u32) {
        let limit = self.limit;
        let offset = self.offset;

        (self.vec(), offset, limit)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Persisted<T> {
    inner: T,
    source_id: i64,
}

impl<T> Persisted<T> {
    pub fn get_id(&self) -> i64 {
        self.source_id
    }

    pub fn get_inner(&self) -> &T {
        &self.inner
    }
}

impl<T> AsRef<T> for Persisted<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl From<api::Module> for Persisted<Module> {
    fn from(a: api::Module) -> Self {
        let metadata = if a.metadata.is_empty() {
            None
        } else {
            Some(a.metadata)
        };

        let inserted_at: std::time::SystemTime = a
            .inserted_at
            .unwrap_or_else(|| protobuf::well_known_types::timestamp::Timestamp::new())
            .into();

        Persisted {
            inner: Module {
                hash: a.hash,
                imports: from_api::imports(a.imports),
                exports: from_api::exports(a.exports),
                size: a.size as u64,
                location: a.location,
                source_language: from_api::source_language(
                    a.source_language.enum_value_or_default(),
                ),
                metadata,
                inserted_at: inserted_at.into(),
                strings: a.strings,
                complexity: a.complexity,
                graph: a.graph,
            },
            source_id: a.id,
        }
    }
}

#[cfg(feature = "mock")]
impl<T> Persisted<T> {
    pub fn from_module(id: i64, module: T) -> Self {
        Persisted {
            inner: module,
            source_id: id,
        }
    }
}
