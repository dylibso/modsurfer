use modsurfer_convert::{api, from_api};
use modsurfer_module::Module;

/// A type returned from the API acting as a collection of results, with some additional data about
/// how the results were constructed (including the offset and limit from the caller).
#[derive(Default, PartialEq)]
pub struct List<T> {
    inner: Vec<T>,
    total: u32,
    offset: u32,
    limit: u32,
}

impl<T> List<T> {
    /// Construct a new List from a container of objects as well as some information about how the
    /// the inner container was created.
    pub fn new(inner: Vec<T>, total: u32, offset: u32, limit: u32) -> Self {
        Self {
            inner,
            total,
            offset,
            limit,
        }
    }

    /// Convert the inner container of objects into a `Vec<&T>`, obtaining references to the objects.
    pub fn vec(&self) -> Vec<&T> {
        self.inner.iter().map(|t| t).collect::<Vec<&T>>()
    }

    /// Separate the items from within the `List` to use independently.
    pub fn split(&self) -> (Vec<&T>, u32, u32) {
        let limit = self.limit;
        let offset = self.offset;

        (self.vec(), offset, limit)
    }
}

/// A helper type, returned from some API operations which contains the database-assigned ID of a
/// persisted object.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Persisted<T> {
    inner: T,
    source_id: i64,
}

impl<T> Persisted<T> {
    /// Get the database-assigned ID of the persisted object.
    pub fn get_id(&self) -> i64 {
        self.source_id
    }

    /// Get the actual persisted object, independent of its ID.
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
    /// Convert a non-persisted object into one usable by API operations which expect IDs. Only
    /// available with the `mock` feature enabled.
    pub fn from_module(id: i64, module: T) -> Self {
        Persisted {
            inner: module,
            source_id: id,
        }
    }
}
