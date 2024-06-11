use std::sync::Arc;
use bevy::prelude::Resource;

use parking_lot::RwLock;
#[derive(Resource, Default)]
pub struct RegistryResource<T>(Arc<RwLock<T>>);

impl<T> RegistryResource<T> {
    pub fn clone_registry(&self) -> Arc<RwLock<T>> {
        Arc::clone(&self.0)
    }

    pub fn new(t: T) -> Self {
        RegistryResource(Arc::new(RwLock::new(t)))
    }
}