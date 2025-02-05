//! TODO

use std::{any::TypeId, future::Future, path::PathBuf, sync::Arc};

use downcast_rs::DowncastSync;
pub use froglight_tool_macros::Dependency;
use hashbrown::HashMap;
use reqwest::Client;
use tokio::sync::RwLock;

/// A thread-safe container for shared dependencies.
///
/// This is cheaply cloneable and can be shared across threads.
#[derive(Default, Clone)]
pub struct SharedDependencies(Arc<RwLock<DependencyContainer>>);
impl std::ops::Deref for SharedDependencies {
    type Target = Arc<RwLock<DependencyContainer>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}

/// A container for shared dependencies.
#[derive(Default)]
pub struct DependencyContainer {
    /// A shared [`Client`] for making network requests.
    pub client: Client,
    /// A cache directory for storing dependencies.
    pub cache: PathBuf,
    dependencies: HashMap<TypeId, Box<dyn Dependency>>,
}

impl DependencyContainer {
    /// Create a new [`DependencyContainer`].
    #[inline]
    #[must_use]
    pub fn new(cache: PathBuf) -> Self { Self::from_client(cache, Client::default()) }

    /// Create a new [`DependencyContainer`] using a [`Client`].
    #[must_use]
    pub fn from_client(cache: PathBuf, client: Client) -> Self {
        Self { client, cache, dependencies: HashMap::new() }
    }
}

impl DependencyContainer {
    /// Get a regerence to a [`Dependency`].
    ///
    /// Returns `None` if the dependency is not found.
    #[must_use]
    pub fn get<T: Dependency>(&self) -> Option<&T> {
        self.dependencies
            .get(&TypeId::of::<T>())
            .and_then(|dep| dep.as_ref().as_any().downcast_ref())
    }
    /// Get a mutable reference to a [`Dependency`].
    ///
    /// Returns `None` if the dependency is not found.
    #[must_use]
    pub fn get_mut<T: Dependency>(&mut self) -> Option<&mut T> {
        self.dependencies
            .get_mut(&TypeId::of::<T>())
            .and_then(|dep| dep.as_mut().as_any_mut().downcast_mut())
    }

    /// Returns `true` if the [`DependencyContainer`] contains a [`Dependency`].
    #[must_use]
    pub fn contains<T: Dependency>(&self) -> bool {
        self.dependencies.contains_key(&TypeId::of::<T>())
    }

    /// Get a [`Dependency`] or retrieve it if it does not exist.
    ///
    /// # Errors
    /// Returns an error if the [`Dependency`] could not be retrieved.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_or_retrieve<T: Dependency + Retrievable>(&mut self) -> anyhow::Result<&T> {
        if !self.contains::<T>() {
            let dep = T::retrieve(self).await?;
            self.insert(dep);
        }
        Ok(self.get::<T>().unwrap())
    }
    /// Get a mutable [`Dependency`] or retrieve it if it does not exist.
    ///
    /// # Errors
    /// Returns an error if the [`Dependency`] could not be retrieved.
    #[expect(clippy::missing_panics_doc)]
    pub async fn get_or_retrieve_mut<T: Dependency + Retrievable>(
        &mut self,
    ) -> anyhow::Result<&mut T> {
        if !self.contains::<T>() {
            let dep = T::retrieve(self).await?;
            self.insert(dep);
        }
        Ok(self.get_mut::<T>().unwrap())
    }

    /// Insert a [`Dependency`] into the [`DependencyContainer`].
    pub fn insert<T: Dependency>(&mut self, dep: T) {
        self.dependencies.insert(TypeId::of::<T>(), Box::new(dep));
    }
    /// Take a [`Dependency`] from the [`DependencyContainer`].
    ///
    /// Returns `None` if the [`Dependency`] is not found.
    pub fn take<T: Dependency>(&mut self) -> Option<T> {
        self.dependencies.remove(&TypeId::of::<T>()).and_then(|dep| -> Option<T> {
            dep.into_any().downcast().map_or(None, |dep: Box<T>| Some(*dep))
        })
    }

    /// Remove a [`Dependency`] from the [`DependencyContainer`] and use it in a
    /// closure.
    ///
    /// Allows for accessing multiple dependencies mutably at once.
    ///
    /// # Panics
    /// Panics if the [`Dependency`] is not found.
    pub fn scoped<T: Dependency, Ret: Sized>(
        &mut self,
        f: impl FnOnce(&mut T, &mut Self) -> Ret,
    ) -> Ret {
        let mut dep = self.take::<T>().expect("Dependency not found to scope!");
        let result = f(&mut dep, self);
        self.insert(dep);
        result
    }
    /// Remove a [`Dependency`] from the [`DependencyContainer`] and use it in
    /// an async closure.
    ///
    /// Allows for accessing multiple dependencies mutably at once.
    ///
    /// # Panics
    /// Panics if the [`Dependency`] is not found.
    pub async fn scoped_fut<T: Dependency, Ret: Sized>(
        &mut self,
        f: impl AsyncFnOnce(&mut T, &mut Self) -> Ret,
    ) -> Ret {
        let mut dep = self.take::<T>().expect("Dependency not found to scope!");
        let result = f(&mut dep, self).await;
        self.insert(dep);
        result
    }
}

/// A trait for dependencies that can be stored in a [`DependencyContainer`].
pub trait Dependency: DowncastSync {}

/// A trait for dependencies that can be retrieved using
/// a [`DependencyContainer`].
pub trait Retrievable: Sized {
    /// Retrieve a [`Dependency`] with context from a [`DependencyContainer`].
    ///
    /// # Errors
    /// Returns an error if the [`Dependency`] could not be retrieved.
    fn retrieve(
        deps: &mut DependencyContainer,
    ) -> impl Future<Output = anyhow::Result<Self>> + Send + Sync;
}
// Blanket implementation for types that implement `Default`.
impl<T: Default> Retrievable for T {
    async fn retrieve(_: &mut DependencyContainer) -> anyhow::Result<Self> { Ok(Self::default()) }
}
