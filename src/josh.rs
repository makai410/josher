use std::{fmt::Display, marker::PhantomData};

use crate::tools::Repo;

/// Marker for filtered things.
pub struct Virtual;

// FIXME: is it really necessary to have a borrow??
pub struct JoshFilter<'repo, T> {
    inner: String,
    repo: &'repo Repo,
    _universe: PhantomData<T>,
}

impl<'repo, T> JoshFilter<'repo, T> {
    pub fn new(filter: String, repo: &'repo Repo) -> JoshFilter<'repo, T> {
        Self {
            inner: filter,
            repo,
            _universe: PhantomData,
        }
    }

    pub fn apply(&self, commit: &Commit<T>) -> Commit<Virtual> {
        // FIXME: check this read output
        // FIXME: this should take a ref instead of a commit
        // FIXME: seems that the exit code here is kinda weird
        // FIXME: We need a refspec system
        let oid = self
            .repo
            .filter(&self.inner, "todo", "todo")
            .expect("josh-filter failed to run");
        Commit::<Virtual>::from("todo".to_string())
    }

    pub fn reverse_apply(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        // FIXME: I'm 99% sure there's something buggy
        // FIXME: this should take a ref instead of a commit
        // FIXME: We need a refspec system
        let oid = self
            .repo
            .reverse_filter(&self.inner, "todo", "todo", true)
            .expect("josh-filter failed to run");
        Commit::<T>::from("todo".to_string())
    }
}

pub trait Projection<T> {
    fn project(&self, commit: &Commit<T>) -> Commit<Virtual>;
    fn reconstruct(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T>;
}

impl<'repo, T> Projection<T> for JoshFilter<'repo, T> {
    fn project(&self, commit: &Commit<T>) -> Commit<Virtual> {
        self.apply(commit)
    }
    fn reconstruct(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        self.reverse_apply(base, from)
    }
}

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
