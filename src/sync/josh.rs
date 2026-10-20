use std::{ffi::OsStr, marker::PhantomData, path::PathBuf};

use anyhow::Context;
use xshell::{Shell, cmd};

use crate::{
    JOSH_VERSION,
    git::Repo,
    sync::core::Projection,
    utils::{collect_args, install_dir},
};

/// Marker for filtered things.
// FIXME: We'd need this when we have typed refs.
// pub struct Virtual;

pub struct JoshFilter<'bin, 'repo, T> {
    inner: String,
    bin: &'bin JoshFilterBin<'repo>,
    _universe: PhantomData<fn() -> T>,
}

impl<'bin, 'repo, T> JoshFilter<'bin, 'repo, T> {
    pub fn new(filter: String, bin: &'bin JoshFilterBin<'repo>) -> Self {
        Self {
            inner: filter,
            bin,
            _universe: PhantomData,
        }
    }

    fn apply(
        &self,
        from: &git2::Reference<'repo>,
        to: &git2::Reference<'repo>,
    ) -> anyhow::Result<()> {
        // FIXME: check the read output
        // FIXME: seems that the exit code here is kinda weird
        self.bin.filter(&self.inner, from, to)
    }

    fn reverse_apply(
        &self,
        base: &git2::Reference<'repo>,
        from: &git2::Reference<'repo>,
    ) -> anyhow::Result<()> {
        // FIXME: I'm 99% sure there's something buggy
        self.bin.reverse_filter(&self.inner, base, from, true)
    }
}

impl<'bin, 'repo, T> Projection<'repo, T> for JoshFilter<'bin, 'repo, T> {
    fn project(&self, ref_: &git2::Reference<'repo>) -> git2::Reference<'repo> {
        let update_to = self.bin.repo.reference_empty();
        self.apply(ref_, &update_to).expect("cannot apply filter");
        update_to
    }
    fn reconstruct(
        &self,
        base: &git2::Reference<'repo>,
        from: &git2::Reference<'repo>,
    ) -> git2::Reference<'repo> {
        self.reverse_apply(base, from)
            .expect("cannot reverse-apply filter");
        // FIXME: this does feel pretty damn weird.
        // The thing is that josh-filter will OVERWRITE the base reference here.
        // I kinda don't want to change the signature of this method, since
        // the current way is symmetric to `project`. I don't want to tweak it just
        // bacause of the behaviors of Josh.
        // I want to keep sync::core completely agnostic of Josh.
        base.clone()
    }
}

pub struct JoshFilterBin<'repo> {
    /// Repository we want the `josh-filter` binary to work in.
    repo: &'repo Repo,
    sh: Shell,
    path: PathBuf,
    verbose: bool,
}

impl<'repo> JoshFilterBin<'repo> {
    pub fn filter(
        &self,
        filter: &str,
        from: &git2::Reference<'repo>,
        update_to: &git2::Reference<'repo>,
    ) -> anyhow::Result<()> {
        self.run(&[filter, from.name()?, "--update", update_to.name()?])?;
        Ok(())
    }

    pub fn reverse_filter(
        &self,
        filter: &str,
        base: &git2::Reference<'repo>,
        update_to: &git2::Reference<'repo>,
        check_roundtrip: bool,
    ) -> anyhow::Result<()> {
        let mut args = vec![
            filter,
            base.name()?,
            "--update",
            update_to.name()?,
            "--reverse",
        ];
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
    repo: &'repo Repo,
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
