use crate::{git::Ref, josh::Virtual};

pub struct SyncBikeshed<'repo, Dest> {
    // FIXME: naming bikshedding
    pub desired: Ref<'repo, Virtual>,

    // FIXME: naming bikshedding
    pub destination: Ref<'repo, Dest>,
}

pub enum SyncResBikeshed<'repo, Dest> {
    UpToDate,
    Updated(SyncBikeshed<'repo, Dest>),
}

/// The generic synchronization model.
pub fn synchronize<'repo, Source, Dest>(
    source_projection: &impl Projection<'repo, Source>,
    dest_projection: &impl Projection<'repo, Dest>,
    source_ref: &Ref<'repo, Source>,
    dest_ref: &Ref<'repo, Dest>,
    integ: impl Integration,
) -> anyhow::Result<SyncResBikeshed<'repo, Dest>> {
    // What the source repository currently exposes in the "virtual" space.
    // E.g., when doing pull, the source repository is mono repo, and then we filter it,
    // now we have the filtered commits in the "virtual" space.
    let incoming = source_projection.project(source_ref);

    // Then we filter the destination repo history. We now have the counterpart in the
    // "virtual" space.
    let current = dest_projection.project(dest_ref);

    // Then we let the user determine what to do with the integration, like directly
    // fast-forwarding or merging or whatever.
    let Some(desired) = integ.integrate(&current, &incoming)? else {
        return Ok(SyncResBikeshed::UpToDate);
    };

    // Reconstruct it back to the destination
    let dest = dest_projection.reconstruct(dest_ref, &desired);

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
    fn integrate<'repo>(
        &self,
        current: &Ref<'repo, Virtual>,
        incoming: &Ref<'repo, Virtual>,
    ) -> anyhow::Result<Option<Ref<'repo, Virtual>>>;
}

pub trait Projection<'repo, T> {
    fn project(&self, ref_: &Ref<'repo, T>) -> Ref<'repo, Virtual>;
    fn reconstruct(&self, base: &Ref<'repo, T>, from: &Ref<'repo, Virtual>) -> Ref<'repo, T>;
}
