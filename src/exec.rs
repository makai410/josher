use anyhow::Context;

use crate::{
    Commit,
    config::Config,
    josh::{JoshFilter, Virtual, get_josh_filter_bin},
    refs::{RefTag, Refspec},
    sync::Integration,
    tools::{self, Repo},
    utils::{gen_runid, mono_id, workspace_dir},
};

struct ExecCtxt<'a> {
    config: Config,
    workspace: &'a Workspace,
    run_id: String,
    verbose: bool,
}

pub fn run(mode: Mode, config: Config, verbose: bool) -> anyhow::Result<()> {
    // Assumes that we are right in the subtree repo.
    let repo_path =
        std::env::current_dir().context("cannot determine the current repository path")?;
    let subtree_repo = tools::open_repo::<Subtree>(repo_path, verbose)?;
    let workspace = Workspace::open_or_create(verbose, mono_id(&config))?;
    let bin = get_josh_filter_bin(&workspace.repo, verbose)?;
    let subtree_filter = JoshFilter::<Subtree>::new(config.subtree.filter.clone(), &bin);
    let mono_filter = JoshFilter::<Mono>::new(config.mono.filter.clone(), &bin);

    let exec_ctxt = ExecCtxt {
        config,
        workspace: &workspace,
        run_id: gen_runid(),
        verbose,
    };

    match mode {
        Mode::Pull => {
            exec_ctxt.pull(subtree_repo, subtree_filter, mono_filter)?;
        }
        Mode::Push => {
            exec_ctxt.push()?;
        }
    }
    Ok(())
}

impl<'a> ExecCtxt<'a> {
    fn pull(
        &self,
        subtree_repo: Repo<Subtree>,
        subtree_filter: JoshFilter<Subtree>,
        mono_filter: JoshFilter<Mono>,
    ) -> anyhow::Result<()> {
        // Create a new branch in subtree repo
        // Prepare commit in subtree repo...

        // fetch subtree repo HEAD into workspace
        self.workspace.repo.fetch(
            &subtree_repo.path().to_str().expect(&format!(
                "{} is invalid unicode",
                &subtree_repo.path().display()
            )),
            &[self.refspec(RefTag::DestinationBase).as_ref()],
        )?;

        Ok(())
    }

    fn push(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn refspec(&self, tag: RefTag) -> Refspec {
        Refspec::new(&self.run_id, tag.as_str())
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
    repo: Repo<Virtual>,
    id: String,
}

impl Workspace {
    pub fn open_or_create(verbose: bool, id: String) -> anyhow::Result<Self> {
        let path = workspace_dir().join(&id);
        let repo = tools::open_or_create_bare_repo(path, verbose)?;
        Ok(Self { repo, id })
    }
}

pub struct FastForward;

impl Integration for FastForward {
    fn integrate(
        &self,
        repo: &Repo<Virtual>,
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
        repo: &Repo<Virtual>,
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
    repo: &Repo<Virtual>,
    current: &Commit<Virtual>,
    incoming: &Commit<Virtual>,
) -> anyhow::Result<Commit<Virtual>> {
    todo!("")
}
