use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use anyhow::Context;
use xshell::{Cmd, Shell, cmd};

use crate::JOSH_VERSION;
use crate::refs::Refspec;
use crate::utils::{collect_args, install_dir};

/// Open a git repository.
pub fn open_repo(path: impl AsRef<Path>, verbose: bool) -> anyhow::Result<Repo> {
    let path = path.as_ref().canonicalize().context(format!(
        "cannot resolve repository at {}",
        path.as_ref().display()
    ))?;
    let sh = Shell::new()?;
    sh.change_dir(&path);
    let filter_bin = get_josh_filter_bin(&path, verbose)?;
    let repo = Repo {
        sh,
        filter_bin,
        path,
        verbose,
    };
    repo.read(["rev-parse", "--git-dir"])
        .context(format!("{} is not a Git repository", repo.path.display()))?;
    Ok(repo)
}

pub struct Repo {
    sh: Shell,
    filter_bin: JoshFilterBin,
    path: PathBuf,
    verbose: bool,
}

impl Repo {
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

    pub fn filter(&self, filter: &str, refspec: &str, update_to: &str) -> anyhow::Result<()> {
        self.filter_bin
            .run(&[filter, refspec, "--update", update_to])?;
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
        self.filter_bin.run(args)?;
        Ok(())
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

    fn command<'a, I, S>(&'a self, args: I) -> Cmd<'a>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args = collect_args(args);
        let mut cmd = cmd!(self.sh, "git {args...}");
        cmd.set_quiet(!self.verbose);
        cmd
    }
}

fn get_josh_filter_bin(
    working_path: impl AsRef<Path>,
    verbose: bool,
) -> anyhow::Result<JoshFilterBin> {
    let install_dir = install_dir();
    let josh_filter_path = install_dir.join("bin").join("josh-filter");
    let install_dir = install_dir.to_str().expect("the path is invalid unicode");
    println!(
        "Updating/installing josh-filter binary into `{}`...",
        josh_filter_path.display()
    );

    let sh = Shell::new()?;
    sh.change_dir(&working_path);
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
        sh,
        path: josh_filter_path,
        verbose,
    })
}

struct JoshFilterBin {
    sh: Shell,
    path: PathBuf,
    verbose: bool,
}

impl JoshFilterBin {
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
