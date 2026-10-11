use std::{ffi::OsStr, marker::PhantomData, path::PathBuf};

use anyhow::Context;
use xshell::{Cmd, Shell, cmd};

use crate::{
    JOSH_VERSION,
    utils::{collect_args, install_dir},
};

/// Marker for things from subtree repository.
pub struct Subtree;
/// Marker for things from mono repository.
pub struct Mono;
/// Marker for filtered things.
pub struct Virtual;

pub struct JoshFilterBin {
    sh: Shell,
    path: PathBuf,
    verbose: bool,
}

impl JoshFilterBin {
    fn read<I, S>(&self, args: I) -> anyhow::Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let command = self.command(args);
        command.read().context("cannot read command output")
    }

    fn run<I, S>(&self, args: I) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let command = self.command(args);
        command.run().context("cannot run command")
    }

    fn command<'a, I, S>(&'a self, args: I) -> Cmd<'a>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = collect_args(args);
        let mut cmd = self.sh.cmd(&self.path).args(args);
        cmd.set_quiet(!self.verbose);
        cmd
    }
}
pub struct JoshFilter<'a> {
    inner: String,
    bin: &'a JoshFilterBin,
}

impl<'a> JoshFilter<'a> {
    pub fn new(filter: String, bin: &'a JoshFilterBin) -> anyhow::Result<JoshFilter<'a>> {
        Ok(Self { inner: filter, bin })
    }

    pub fn apply<T>(&self, commit: &Commit<T>) -> Commit<Virtual> {
        // FIXME: check this read output
        let oid = self
            .bin
            .read(&[&self.inner, &commit.oid])
            .expect("josh-filter failed to run");
        Commit::<Virtual>::from(oid)
    }

    pub fn reverse_apply<T>(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T> {
        // FIXME: I'm 99% sure there's something buggy
        let oid = self
            .bin
            .read(&[&self.inner, &base.oid, "--update", &from.oid, "--reverse"])
            .expect("josh-filter failed to run");
        Commit::<T>::from(oid)
    }
}

pub struct Projection<'a, T> {
    filter: JoshFilter<'a>,
    _universe: PhantomData<T>,
}

impl<'a, T> Projection<'a, T> {
    pub fn new(filter: JoshFilter<'a>) -> Self {
        Self {
            filter,
            _universe: PhantomData,
        }
    }
}

impl<'a> Projection<'a, Subtree> {
    pub fn project(&self, commit: &Commit<Subtree>) -> Commit<Virtual> {
        self.filter.apply(commit)
    }

    pub fn reverse(&self, base: &Commit<Subtree>, from: &Commit<Virtual>) -> Commit<Subtree> {
        self.filter.reverse_apply(base, from)
    }
}

impl<'a> Projection<'a, Mono> {
    pub fn project(&self, commit: &Commit<Mono>) -> Commit<Virtual> {
        self.filter.apply(commit)
    }
    pub fn reverse(&self, base: &Commit<Mono>, from: &Commit<Virtual>) -> Commit<Mono> {
        self.filter.reverse_apply(base, from)
    }
}

pub struct Commit<U> {
    oid: String,
    _universes: PhantomData<U>,
}

impl<U> Commit<U> {
    pub fn from(oid: String) -> Self {
        Self {
            oid,
            _universes: PhantomData,
        }
    }
}

fn get_josh_filter_bin(verbose: bool) -> anyhow::Result<JoshFilterBin> {
    let install_dir = install_dir();
    let josh_filter_path = install_dir.join("bin").join("josh-filter");
    let install_dir = install_dir.to_str().expect("the path is invalid unicode");
    println!(
        "Updating/installing josh-filter binary into `{}`...",
        josh_filter_path.display()
    );

    let sh = Shell::new()?;
    let args = vec![
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
        sh,
        path: josh_filter_path,
        verbose,
    })
}
