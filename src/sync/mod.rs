use crate::{
    git::Repo,
    sync::{
        core::SyncInput,
        josh::{JoshFilter, get_josh_filter_bin},
    },
};

pub mod core;
mod josh;

pub struct SyncDirection<Src, Dest> {
    _universe: std::marker::PhantomData<fn() -> (Src, Dest)>,
}

impl<Src, Dest> SyncDirection<Src, Dest> {
    pub fn run<'repo>(
        src_filter: String,
        dest_filter: String,
        src_ref: git2::Reference<'repo>,
        dest_ref: git2::Reference<'repo>,
        repo: &'repo Repo,
        integ: impl Integration,
        verbose: bool,
    ) -> anyhow::Result<SyncResult<'repo>> {
        let bin = get_josh_filter_bin(repo, verbose)?;
        let src_filter = JoshFilter::<Src>::new(src_filter, &bin);
        let dest_filter = JoshFilter::<Dest>::new(dest_filter, &bin);
        let src_input = SyncInput {
            projection: src_filter,
            reference: src_ref,
            _universe: std::marker::PhantomData,
        };
        let dest_input = SyncInput {
            projection: dest_filter,
            reference: dest_ref,
            _universe: std::marker::PhantomData,
        };
        SyncDirection::sync(src_input, dest_input, integ)
    }
}

/// Decides what virtual history should be reconstructed at the destination.
pub trait Integration {
    fn integrate<'repo>(
        &self,
        current: &git2::Reference<'repo>,
        incoming: &git2::Reference<'repo>,
    ) -> anyhow::Result<Option<git2::Reference<'repo>>>;
}

pub enum SyncResult<'repo> {
    UpToDate,
    Updated(git2::Reference<'repo>),
}
