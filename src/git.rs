use std::path::Path;

use anyhow::Context;
use git2::Reference;

use crate::utils::{gen_ref_name, gen_runid};

/// Open a git repository.
pub fn open_repo(path: impl AsRef<Path>) -> anyhow::Result<Repo> {
    let repo = git2::Repository::open_ext(
        &path,
        git2::RepositoryOpenFlags::NO_SEARCH,
        &[] as &[&std::ffi::OsStr],
    )?;
    let repo = Repo {
        repo,
        run_id: gen_runid(),
    };
    Ok(repo)
}

pub fn open_or_create_bare_repo(path: impl AsRef<Path>) -> anyhow::Result<Repo> {
    let repo = git2::Repository::open_bare(&path).or_else(|_| git2::Repository::init(&path))?;
    let repo = Repo {
        repo,
        run_id: gen_runid(),
    };
    Ok(repo)
}

pub struct Repo {
    repo: git2::Repository,
    run_id: String,
}

impl Repo {
    pub fn fetch(&self, remote_name: &str, url: &str, from: &str) -> anyhow::Result<Reference<'_>> {
        let to = self.new_ref_name();
        let spec = &format!("+{}:{}", from, &to);
        let mut remote = self.repo.remote_with_fetch(remote_name, url, spec)?;
        remote.fetch(&[] as &[&str], None, None)?;
        let ref_ = self
            .repo
            .find_reference(&*to)
            .context("cannot find reference")?;
        Ok(ref_)
    }

    pub fn path(&self) -> &Path {
        self.repo.path()
    }

    pub fn is_ancestor(
        &self,
        ancestor: &Reference<'_>,
        descendant: &Reference<'_>,
    ) -> anyhow::Result<bool> {
        // TODO: check eq
        self.repo
            .graph_descendant_of(descendant.target().unwrap(), ancestor.target().unwrap())
            .context("cannot determine")
    }

    /// We actually don't need to care about the exact name here.
    pub fn reference_empty(&self) -> Reference<'_> {
        let name = self.new_ref_name();
        self.repo
            .reference(&name, git2::Oid::ZERO_SHA1, false, "")
            .context("cannot create reference")
            .expect("something wrong")
    }

    fn new_ref_name(&self) -> String {
        format!("refs/josher/runs/{}/{}", &self.run_id, &gen_ref_name())
    }
}
