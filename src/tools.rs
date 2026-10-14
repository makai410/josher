use std::ffi::OsStr;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use anyhow::Context;
use xshell::{Cmd, Shell, cmd};

use crate::refs::Refspec;
use crate::utils::collect_args;

/// Open a git repository.
pub fn open_repo<T>(path: impl AsRef<Path>, verbose: bool) -> anyhow::Result<Repo<T>> {
    let path = path.as_ref().canonicalize().context(format!(
        "cannot resolve repository at {}",
        path.as_ref().display()
    ))?;
    let sh = Shell::new()?;
    sh.change_dir(&path);
    sh.set_var(
        "GIT_DIR",
        path.to_str()
            .expect(&format!("{} is invalid unicode", path.display())),
    );
    let repo = Repo {
        sh,
        path,
        verbose,
        _universe: PhantomData,
    };
    repo.read(&["rev-parse", "--git-dir"])
        .context(format!("{} is not a Git repository", repo.path.display()))?;
    Ok(repo)
}

pub fn open_or_create_bare_repo<T>(
    path: impl AsRef<Path>,
    verbose: bool,
) -> anyhow::Result<Repo<T>> {
    let path = path.as_ref().canonicalize().context(format!(
        "cannot resolve path at {}",
        path.as_ref().display()
    ))?;
    let sh = Shell::new()?;
    sh.change_dir(&path);
    sh.set_var(
        "GIT_DIR",
        path.to_str()
            .expect(&format!("{} is invalid unicode", path.display())),
    );
    let repo = Repo {
        sh,
        path,
        verbose,
        _universe: PhantomData,
    };

    // FIXME: eventually we need to fix this. this is looking terrible.
    match repo.read(&["rev-parse", "--is-bare-repository"]) {
        Ok(res) => {
            if res == "false" {
                anyhow::bail!(
                    "{} is a Git repository, but somehow is NOT a bare repository!",
                    repo.path.display()
                );
            }
            Ok(repo)
        }
        Err(_) => {
            repo.run(&["--bare", "init"])
                .context("cannot init bare repo")?;

            repo.read(["rev-parse", "--is-bare-repository"])
                .context(format!(
                    "{} is not a Git bare repository, `git --bare init` somehow failed",
                    repo.path.display()
                ))?;

            Ok(repo)
        }
    }
}

pub struct Repo<T> {
    sh: Shell,
    path: PathBuf,
    verbose: bool,
    _universe: PhantomData<T>,
}

impl<T> Repo<T> {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn rev_parse(&self, revision: &str) -> anyhow::Result<String> {
        self.read(&["rev-parse", "--verify", revision])
            .context(format!("cannot resolve Git revision `{revision}`"))
    }

    pub fn head(&self) -> anyhow::Result<String> {
        self.rev_parse("HEAD")
    }

    pub fn current_branch(&self) -> anyhow::Result<Option<String>> {
        let output = self
            .command(&["symbolic-ref", "--quiet", "--short", "HEAD"])
            .ignore_status()
            .output()?;
        match output.status.code() {
            Some(0) => Ok(Some(String::from_utf8(output.stdout)?.trim().to_owned())),
            Some(1) => Ok(None),
            _ => anyhow::bail!("git symbolic-ref failed with {}", output.status),
        }
    }

    pub fn is_clean(&self) -> anyhow::Result<bool> {
        Ok(self
            .read(&["status", "--porcelain", "--untracked-files=no"])?
            .is_empty())
    }

    pub fn ensure_clean(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.is_clean()?,
            "repository at {} has tracked changes",
            self.path.display()
        );
        Ok(())
    }

    pub fn fetch(&self, remote: &str, refspecs: &[&str]) -> anyhow::Result<()> {
        let mut args = vec!["fetch", "--no-tags", remote];
        args.extend_from_slice(refspecs);
        self.run(args)
            .context(format!("cannot fetch from `{remote}`"))
    }

    pub fn update_ref(
        &self,
        refspec: &Refspec,
        new_value: &str,
        expected_old: Option<&str>,
    ) -> anyhow::Result<()> {
        match expected_old {
            Some(old) => self.run(&["update-ref", refspec, new_value, old]),
            None => self.run(&["update-ref", refspec, new_value]),
        }
        .context(format!("cannot update `{refspec}`"))
    }

    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> anyhow::Result<bool> {
        let output = self
            .command(&["merge-base", "--is-ancestor", ancestor, descendant])
            .ignore_status()
            .output()?;
        match output.status.code() {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => anyhow::bail!("git merge-base failed with {}", output.status),
        }
    }

    pub fn run_by_path<I, S>(&self, path: impl AsRef<Path>, args: I) -> anyhow::Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let command = self.command_by_path(path, args);
        command.run().context("cannot run command")
    }

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

    fn command<I, S>(&self, args: I) -> Cmd<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = collect_args(args);
        let mut cmd = cmd!(self.sh, "git {args...}");
        cmd.set_quiet(!self.verbose);
        cmd
    }

    fn command_by_path<I, S>(&self, path: impl AsRef<Path>, args: I) -> Cmd<'_>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = collect_args(args);
        let mut cmd = self.sh.cmd(&path).args(args);
        cmd.set_quiet(!self.verbose);
        cmd
    }
}
