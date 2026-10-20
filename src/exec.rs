use anyhow::Context;

use crate::{
    config::Config,
    git::{self, Repo},
    josh::{JoshFilter, get_josh_filter_bin},
    utils::{mono_id, workspace_dir},
};

/// Marker for things from subtree repository.
pub struct Subtree;
/// Marker for things from mono repository.
pub struct Mono;

pub enum Mode {
    Pull,
    Push,
}

pub fn run(mode: Mode, config: Config, verbose: bool) -> anyhow::Result<()> {
    let workspace = Workspace::open_or_create(mono_id(&config))?;

    let exec_ctxt = ExecCtxt {
        config,
        workspace: &workspace,
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

struct ExecCtxt<'a> {
    config: Config,
    workspace: &'a Workspace,
    verbose: bool,
}

impl<'a> ExecCtxt<'a> {
    fn pull(&self) -> anyhow::Result<()> {
        // Assumes that we are right in the subtree repo.
        let repo_path =
            std::env::current_dir().context("cannot determine the current repository path")?;
        let subtree_repo = git::open_repo(repo_path)?;
        let repo = &self.workspace.repo;
        let bin = get_josh_filter_bin(repo, self.verbose)?;
        let subtree_filter = JoshFilter::<Subtree>::new(self.config.subtree.filter.clone(), &bin);
        let mono_filter = JoshFilter::<Mono>::new(self.config.mono.filter.clone(), &bin);
        // Create a new branch in subtree repo
        // Prepare commit in subtree repo...

        // fetch subtree repo HEAD into workspace
        let subtree_repo_path = subtree_repo
            .path()
            .to_str()
            .expect("the path is not valid unicode");
        let dest_ref = repo.fetch("subtree", subtree_repo_path, "HEAD", "todo")?;

        // fetch mono repo HEAD into workspace
        let src_ref = repo.fetch("mono", &self.config.mono.url, "refs/heads/main", "todo")?;

        // sync::synchronize(
        //     &mono_filter,
        //     &subtree_filter,
        //     &src_ref,
        //     &dest_ref,
        //     MergeVirtual,
        // );

        Ok(())
    }

    fn push(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

struct Workspace {
    repo: Repo,
}

impl Workspace {
    pub fn open_or_create(id: String) -> anyhow::Result<Self> {
        let path = workspace_dir().join(&id);
        let repo = git::open_or_create_bare_repo(path)?;
        Ok(Self { repo })
    }
}

// pub struct FastForward;

// impl Integration for FastForward {
//     fn integrate(
//         &self,
//         current: &Ref<Virtual>,
//         incoming: &Ref<Virtual>,
//     ) -> anyhow::Result<Option<Ref<Virtual>>> {
//         if current == incoming {
//             return Ok(None);
//         }

//         anyhow::ensure!(
//             repo.is_ancestor(&current, &incoming)?,
//             "the source and destination have diverged; pull and resolve the divergence first"
//         );

//         Ok(Some(Commit::from(incoming.to_string())))
//     }
// }

// pub struct MergeVirtual;

// impl Integration for MergeVirtual {
//     fn integrate(
//         &self,
//         current: &Commit<Virtual>,
//         incoming: &Commit<Virtual>,
//     ) -> anyhow::Result<Option<Commit<Virtual>>> {
//         if current == incoming {
//             return Ok(None);
//         }

//         let merged = merge_virtual_histories(current, incoming)?;
//         Ok(Some(merged))
//     }
// }

// fn merge_virtual_histories(
//     current: &Commit<Virtual>,
//     incoming: &Commit<Virtual>,
// ) -> anyhow::Result<Commit<Virtual>> {
//     todo!("")
// }
