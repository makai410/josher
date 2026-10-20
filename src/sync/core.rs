use std::marker::PhantomData;

use crate::sync::{Integration, SyncDirection, SyncResult};

impl<Src, Dest> SyncDirection<Src, Dest> {
    /// The generic synchronization model.
    pub(super) fn sync<'repo>(
        src_input: SyncInput<'repo, Src, impl Projection<'repo, Src>>,
        dest_input: SyncInput<'repo, Dest, impl Projection<'repo, Dest>>,
        integ: impl Integration,
    ) -> anyhow::Result<SyncResult<'repo>> {
        // What the source repository currently exposes in the "virtual" space.
        // E.g., when doing pull, the source repository is mono repo, and then we filter it,
        // now we have the filtered commits in the "virtual" space.
        let incoming = src_input.projection.project(&src_input.reference);

        // Then we filter the destination repo history. We now have the counterpart in the
        // "virtual" space.
        let current = dest_input.projection.project(&dest_input.reference);

        // Then we let the user determine what to do with the integration, like directly
        // fast-forwarding or merging or whatever.
        let Some(desired) = integ.integrate(&current, &incoming)? else {
            return Ok(SyncResult::UpToDate);
        };

        // Reconstruct it back to the destination
        let dest = dest_input
            .projection
            .reconstruct(&dest_input.reference, &desired);

        // Now we do roundtrip check
        let roundtripped = dest_input.projection.project(&dest);
        anyhow::ensure!(
            roundtripped == desired,
            "Josh created a non-roundtrip history! DO NOT merge this!!!"
        );

        Ok(SyncResult::Updated(dest))
    }
}

pub(super) trait Projection<'repo, T> {
    fn project(&self, ref_: &git2::Reference<'repo>) -> git2::Reference<'repo>;
    fn reconstruct(
        &self,
        base: &git2::Reference<'repo>,
        from: &git2::Reference<'repo>,
    ) -> git2::Reference<'repo>;
}

pub(super) struct SyncInput<'repo, T, P>
where
    P: Projection<'repo, T>,
{
    pub projection: P,
    // FIXME: we also need to make sure this is typed...
    pub reference: git2::Reference<'repo>,
    pub _universe: PhantomData<fn() -> T>,
}
