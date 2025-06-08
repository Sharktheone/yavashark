use crate::scope::Module;
use std::collections::HashMap;
use std::path::PathBuf;
use temporal_rs::tzdb::FsTzdbProvider;

#[derive(Debug)]
pub struct Environment {
    pub modules: HashMap<PathBuf, Module>,
    pub tz_provider: FsTzdbProvider,
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            modules: self.modules.clone(),
            tz_provider: FsTzdbProvider::default(),
        }
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        self.modules == other.modules
    }
}

impl Eq for Environment {}
