use std::{fmt::Display, marker::PhantomData};

pub mod cli;
pub mod config;
pub mod exec;
pub mod josh;
pub mod refs;
pub mod sync;
pub mod tools;
pub mod utils;

const JOSH_VERSION: &str = "r26.08.28";

pub struct Commit<U> {
    oid: String,
    _universe: PhantomData<U>,
}

impl<U> Display for Commit<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.oid)
    }
}

impl<U> PartialEq for Commit<U> {
    fn eq(&self, other: &Self) -> bool {
        self.oid == other.oid
    }
}

impl<U> From<String> for Commit<U> {
    fn from(value: String) -> Self {
        Self {
            oid: value,
            _universe: PhantomData,
        }
    }
}
