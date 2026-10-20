use std::{ffi::OsStr, marker::PhantomData, path::PathBuf};

use anyhow::Context;
use xshell::{Shell, cmd};

use crate::{
    Commit, JOSH_VERSION,
    git::Repo,
    sync::Projection,
    utils::{collect_args, install_dir},
};

/// Marker for filtered things.
pub struct Virtual;

pub struct JoshFilter<'bin, 'repo, T> {
    inner: String,
    bin: &'bin JoshFilterBin<'repo>,
    _universe: PhantomData<fn() -> T>,
}

pub struct JoshFilterBin<'repo> {
    /// Repository we want the `josh-filter` binary to work in.
    /// `Repo<Virtual>` is right here because we only want it
    /// to work in the virtual repository.
    repo: &'repo Repo<Virtual>,
    sh: Shell,
    path: PathBuf,
    verbose: bool,
}

impl<'bin, 'repo, T> JoshFilter<'bin, 'repo, T> {
    pub fn new(filter: String, bin: &'bin JoshFilterBin<'repo>) -> Self {
        Self {
            inner: filter,
            bin,
            _universe: PhantomData,
        }
    }

    fn apply(&self, from: &Ref<T>, to: &Ref<Virtual>) -> anyhow::Result<()> {
        // FIXME: check this read output
        // FIXME: seems that the exit code here is kinda weird
        self.bin.filter(&self.inner, from, to)
    }

    fn reverse_apply(&self, base: &Ref<T>, from: &Ref<Virtual>) -> anyhow::Result<()> {
        // FIXME: I'm 99% sure there's something buggy
        // FIXME: this should take a ref instead of a commit
        // FIXME: We need a refspec system
        self.bin.reverse_filter(&self.inner, base, from, true)
    }
}

impl<'bin, 'repo, T> Projection<T> for JoshFilter<'bin, 'repo, T> {
    fn project(&self, ref_: &Ref<T>) -> Ref<Virtual> {
        self.apply(ref_)
    }
    fn reconstruct(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        self.reverse_apply(base, from)
    }
}

impl<'repo> JoshFilterBin<'repo> {
    pub fn filter(&self, filter: &str, from: &str, update_to: &str) -> anyhow::Result<()> {
        self.run(&[filter, from, "--update", update_to])?;
        Ok(())
    }

    pub fn reverse_filter(
        &self,
        filter: &str,
        base: &str,
        update_to: &str,
        check_roundtrip: bool,
    ) -> anyhow::Result<()> {
        let mut args = vec![filter, base, "--update", update_to, "--reverse"];
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
        let args = collect_args(args);
        let mut cmd = self.sh.cmd(&self.path).args(args);
        cmd.set_quiet(!self.verbose);
        cmd.run().context("cannot run command")
    }
}

pub fn get_josh_filter_bin<'repo>(
    repo: &'repo Repo<Virtual>,
    verbose: bool,
) -> anyhow::Result<JoshFilterBin<'repo>> {
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
    Ok(JoshFilterBin {
        repo,
        sh,
        path,
        verbose,
    })
}
