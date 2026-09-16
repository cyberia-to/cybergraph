//! Codec-checked import of a sealed legacy application store.
use super::{ApplicationGraph, Error, Head, Particle};
use crate::content::Content;
pub use bbg::storage::application::TransferProgress;
use bbg::storage::application::{Error as StoreError, TransferSource};
pub struct Transfer {
    source: TransferSource,
}
impl Transfer {
    pub fn open(source: &std::path::Path, target: Particle, key: Particle) -> Result<Self, Error> {
        Ok(Self {
            source: TransferSource::open(source, target, key)?,
        })
    }
    pub(super) fn from_archive(
        source: bbg::storage::application::ApplicationArchive,
        target: Particle,
        key: Particle,
    ) -> Result<Self, Error> {
        Ok(Self {
            source: TransferSource::from_archive(source, target, key)?,
        })
    }
    pub fn manifest(&self) -> Particle {
        self.source.manifest()
    }
    pub fn sources(&self) -> &[(Particle, Head)] {
        self.source.sources()
    }
    pub fn stage(
        &self,
        target: &ApplicationGraph,
        pages: usize,
    ) -> Result<TransferProgress, Error> {
        Ok(self
            .source
            .stage(&target.store, pages, |id, bytes, source| {
                let content =
                    Content::from_stored(id, bytes.to_vec()).map_err(|_| StoreError::Corrupt)?;
                if let Some((left, right)) = content.children() {
                    for child in [left, right] {
                        let stored = source
                            .content(&child, crate::content::MAX_CONTENT_BYTES + 1)?
                            .ok_or(StoreError::Corrupt)?;
                        Content::from_stored(child, stored).map_err(|_| StoreError::Corrupt)?;
                    }
                }
                Ok(())
            })?)
    }
}
