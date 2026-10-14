use std::{ffi::OsStr, marker::PhantomData, path::PathBuf};

use anyhow::Context;
use xshell::{Cmd, Shell, cmd};

use crate::{Commit, JOSH_VERSION, sync::Projection, tools::Repo, utils::install_dir};

/// Marker for filtered things.
pub struct Virtual;

// FIXME: is it really necessary to have a borrow??
pub struct JoshFilter<'bin, 'repo, T> {
    inner: String,
    bin: &'bin JoshFilterBin<'repo>,
    _universe: PhantomData<T>,
}

struct JoshFilterBin<'repo> {
    repo: &'repo Repo,
    path: PathBuf,
}

impl<'bin, 'repo, T> JoshFilter<'bin, 'repo, T> {
    pub fn new(filter: String, bin: &'bin JoshFilterBin<'repo>) -> Self {
        Self {
            inner: filter,
            bin,
            _universe: PhantomData,
        }
    }

    pub fn apply(&self, commit: &Commit<T>) -> Commit<Virtual> {
        // FIXME: check this read output
        // FIXME: this should take a ref instead of a commit
        // FIXME: seems that the exit code here is kinda weird
        // FIXME: We need a refspec system
        let oid = self
            .bin
            .filter(&self.inner, "todo", "todo")
            .expect("josh-filter failed to run");
        Commit::<Virtual>::from("todo".to_string())
    }

    pub fn reverse_apply(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        // FIXME: I'm 99% sure there's something buggy
        // FIXME: this should take a ref instead of a commit
        // FIXME: We need a refspec system
        let oid = self
            .bin
            .reverse_filter(&self.inner, "todo", "todo", true)
            .expect("josh-filter failed to run");
        Commit::<T>::from("todo".to_string())
    }
}

impl<'bin, 'repo, T> Projection<T> for JoshFilter<'bin, 'repo, T> {
    fn project(&self, commit: &Commit<T>) -> Commit<Virtual> {
        self.apply(commit)
    }
    fn reconstruct(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        self.reverse_apply(base, from)
    }
}

impl<'repo> JoshFilterBin<'repo> {
    pub fn filter(&self, filter: &str, refspec: &str, update_to: &str) -> anyhow::Result<()> {
        self.run(&[filter, refspec, "--update", update_to])?;
        Ok(())
    }

    pub fn reverse_filter(
        &self,
        filter: &str,
        refspec: &str,
        update_to: &str,
        check_roundtrip: bool,
    ) -> anyhow::Result<()> {
        let mut args = vec![filter, refspec, "--update", update_to, "--reverse"];
        if check_roundtrip {
            args.push("--check-roundtrip");
        }
        self.run(args)?;
        Ok(())
    }

    fn run<I, S>(&self, args: I) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.repo.run_by_path(&self.path, args)
    }
}

pub fn get_josh_filter_bin<'repo>(
    repo: &'repo Repo,
    verbose: bool,
) -> anyhow::Result<JoshFilterBin> {
    let install_dir = install_dir();
    let path = install_dir.join("bin").join("josh-filter");
    let install_dir = install_dir.to_str().expect("the path is invalid unicode");
    println!(
        "Updating/installing josh-filter binary into `{}`...",
        path.display()
    );

    let sh = Shell::new()?;
    sh.change_dir(repo.path());
    let args = &[
        "+stable",
        "install",
        "--locked",
        "--git",
        "https://github.com/josh-project/josh",
        "--tag",
        JOSH_VERSION,
        "--root",
        install_dir,
        "josh-cli",
    ];
    let mut cmd = cmd!(sh, "cargo {args...}");
    cmd.set_quiet(!verbose);
    cmd.run().context("cannot install josh-filter binary")?;
    Ok(JoshFilterBin { repo, path })
}
