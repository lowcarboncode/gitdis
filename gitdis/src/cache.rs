use quickleaf::{Cache, Value};

pub type CacheRepo = Cache<Value>;
pub type ArcCache = std::sync::Arc<std::sync::RwLock<CacheRepo>>;
