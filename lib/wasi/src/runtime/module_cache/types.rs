use std::{fmt::Debug, ops::Deref};

use wasmer::{Engine, Module};

use crate::runtime::module_cache::AndThen;

/// A cache for compiled WebAssembly modules.
///
/// ## Assumptions
///
/// Implementations can assume that cache keys are unique and that using the
/// same key to load or save will always result in the "same" module.
///
/// Implementations can also assume that [`CompiledModuleCache::load()`] will
/// be called more often than [`CompiledModuleCache::save()`] and optimise
/// their caching strategy accordingly.
#[async_trait::async_trait]
pub trait ModuleCache: Debug {
    async fn load(&self, key: &str, engine: &Engine) -> Result<Module, CacheError>;

    async fn save(&self, key: &str, module: &Module) -> Result<(), CacheError>;

    /// Chain a second cache onto this one.
    ///
    /// The general assumption is that each subsequent cache in the chain will
    /// be significantly slower than the previous one.
    ///
    /// ```rust
    /// use wasmer_wasix::runtime::module_cache::{
    ///     CompiledModuleCache, ThreadLocalCache, OnDiskCache, SharedCache,
    /// };
    ///
    /// let cache = ThreadLocalCache::default()
    ///     .and_then(SharedCache::default())
    ///     .and_then(OnDiskCache::new("~/.local/cache"));
    /// ```
    fn and_then<C>(self, other: C) -> AndThen<Self, C>
    where
        Self: Sized,
        C: ModuleCache,
    {
        AndThen::new(self, other)
    }
}

#[async_trait::async_trait]
impl<D, C> ModuleCache for D
where
    D: Deref<Target = C> + Debug + Send + Sync,
    C: ModuleCache + Send + Sync + ?Sized,
{
    async fn load(&self, key: &str, engine: &Engine) -> Result<Module, CacheError> {
        (**self).load(key, engine).await
    }

    async fn save(&self, key: &str, module: &Module) -> Result<(), CacheError> {
        (**self).save(key, module).await
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// The item was not found.
    #[error("Not found")]
    NotFound,
    #[error(transparent)]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_object_safe() {
        let _: Option<Box<dyn ModuleCache>> = None;
    }
}
