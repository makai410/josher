use std::path::Path;

use anyhow::Context;
use git2::Reference;

/// Open a git repository.
pub fn open_repo(path: impl AsRef<Path>) -> anyhow::Result<Repo> {
    let repo = git2::Repository::open_ext(
        &path,
        git2::RepositoryOpenFlags::NO_SEARCH,
        &[] as &[&std::ffi::OsStr],
    )?;
    let repo = Repo { repo };
    Ok(repo)
}

pub fn open_or_create_bare_repo(path: impl AsRef<Path>) -> anyhow::Result<Repo> {
    let repo = git2::Repository::open_bare(&path).or_else(|_| git2::Repository::init(&path))?;
    let repo = Repo { repo };
    Ok(repo)
}

pub struct Repo {
    repo: git2::Repository,
}

impl Repo {
    pub fn fetch(
        &self,
        remote_name: &str,
        url: &str,
        from: &str,
        to: &str,
    ) -> anyhow::Result<Reference<'_>> {
        let spec = &format!("+{}:{}", from, &*to);
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
}
