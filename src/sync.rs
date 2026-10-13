use crate::{Commit, josh::Virtual, tools::Repo};

pub struct SyncBikeshed<Dest> {
    // FIXME: naming bikshedding
    pub desired: Commit<Virtual>,

    // FIXME: naming bikshedding
    pub destination: Commit<Dest>,
}

pub enum SyncResBikeshed<Dest> {
    UpToDate,
    Updated(SyncBikeshed<Dest>),
}

/// The generic synchronization model.
pub fn synchronize<Source, Dest>(
    repo: &Repo,
    source_projection: &impl Projection<Source>,
    dest_projection: &impl Projection<Dest>,
    source: &Commit<Source>,
    dest_base: &Commit<Dest>,
    integ: impl Integration,
) -> anyhow::Result<SyncResBikeshed<Dest>> {
    // What the source repository currently exposes in the "virtual" space.
    // E.g., when doing pull, the source repository is mono repo, and then we filter it,
    // now we have the filtered commits in the "virtual" space.
    let incoming = source_projection.project(source);

    // Then we filter the destination repo history. We now have the counterpart in the
    // "virtual" space.
    let current = dest_projection.project(dest_base);

    // Then we let the user determine what to do with the integration, like directly
    // fast-forwarding or merging or whatever.
    let Some(desired) = integ.integrate(repo, &current, &incoming)? else {
        return Ok(SyncResBikeshed::UpToDate);
    };

    // Reconstruct it back to the destination
    let dest = dest_projection.reconstruct(dest_base, &desired);

    // Now we do roundtrip check
    let roundtripped = dest_projection.project(&dest);
    anyhow::ensure!(
        roundtripped == desired,
        "Josh created a non-roundtrip history! DO NOT merge this!!!"
    );

    Ok(SyncResBikeshed::Updated(SyncBikeshed {
        desired,
        destination: dest,
    }))
}

/// Decides what virtual history should be reconstructed at the destination.
pub trait Integration {
    fn integrate(
        &self,
        repo: &Repo,
        current: &Commit<Virtual>,
        incoming: &Commit<Virtual>,
    ) -> anyhow::Result<Option<Commit<Virtual>>>;
}

pub trait Projection<T> {
    fn project(&self, commit: &Commit<T>) -> Commit<Virtual>;
    fn reconstruct(&self, base: &Commit<T>, from: &Commit<Virtual>) -> Commit<T>;
}
