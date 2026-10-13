use anyhow::Context;

use crate::{
    Commit,
    config::Config,
    josh::{JoshFilter, Virtual},
    sync::Integration,
    tools::{self, Repo},
    utils::{gen_runid, workspace_dir},
};

struct ExecCtxt {
    config: Config,
    workspace: Workspace,
    verbose: bool,
}

pub fn run(mode: Mode, config: Config, verbose: bool) -> anyhow::Result<()> {
    // Assumes that we are right in the subtree repo.
    let repo_path =
        std::env::current_dir().context("cannot determine the current repository path")?;
    let subtree_repo = tools::open_repo(repo_path, verbose)?;
    let mono_repo = tools::open_repo("path/to/monorepo", verbose)?;
    let subtree_filter = JoshFilter::<Subtree>::new(config.subtree.filter.clone(), &subtree_repo);
    let mono_filter = JoshFilter::<Mono>::new(config.mono.filter.clone(), &mono_repo);

    let workspace = Workspace::open_or_create(verbose)?;
    let exec_ctxt = ExecCtxt {
        config,
        workspace,
        verbose,
    };
    match mode {
        Mode::Pull => {
            exec_ctxt.pull()?;
        }
        Mode::Push => {
            exec_ctxt.push()?;
        }
    }
    Ok(())
}

impl ExecCtxt {
    fn pull(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn push(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Marker for things from subtree repository.
pub struct Subtree;
/// Marker for things from mono repository.
pub struct Mono;

pub enum Mode {
    Pull,
    Push,
}

struct Workspace {
    repo: Repo,
    run_id: String,
}

impl Workspace {
    // FIXME: this is unfortunate..
    pub fn open_or_create(verbose: bool) -> anyhow::Result<Self> {
        let path = workspace_dir();
        let repo = tools::open_repo(path, verbose)?;
        Ok(Self {
            repo,
            run_id: gen_runid(),
        })
    }
}

pub struct FastForward;

impl Integration for FastForward {
    fn integrate(
        &self,
        repo: &Repo,
        current: &Commit<Virtual>,
        incoming: &Commit<Virtual>,
    ) -> anyhow::Result<Option<Commit<Virtual>>> {
        if current == incoming {
            return Ok(None);
        }

        anyhow::ensure!(
            repo.is_ancestor(&current.to_string(), &incoming.to_string())?,
            "the source and destination have diverged; pull and resolve the divergence first"
        );

        Ok(Some(Commit::from(incoming.to_string())))
    }
}

pub struct MergeVirtual;

impl Integration for MergeVirtual {
    fn integrate(
        &self,
        repo: &Repo,
        current: &Commit<Virtual>,
        incoming: &Commit<Virtual>,
    ) -> anyhow::Result<Option<Commit<Virtual>>> {
        if current == incoming {
            return Ok(None);
        }

        let merged = merge_virtual_histories(repo, current, incoming)?;
        Ok(Some(merged))
    }
}

fn merge_virtual_histories(
    repo: &Repo,
    current: &Commit<Virtual>,
    incoming: &Commit<Virtual>,
) -> anyhow::Result<Commit<Virtual>> {
    todo!("")
}
