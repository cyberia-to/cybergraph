//! Explicit legacy archive reads, including sealed sources, without a writer API.
use super::{Error, Head, Particle};
use crate::content::{Content, MAX_CONTENT_BYTES};
use bbg::storage::application::{ApplicationArchive, Error as StoreError};
pub use bbg::storage::application::{
    ArchiveSeal, ArchiveSummary, MAX_INSPECTION_BYTES, MAX_INSPECTION_ROWS,
};

pub struct Archive(ApplicationArchive);
impl Archive {
    pub fn open(path: &std::path::Path) -> Result<Self, Error> {
        Ok(Self(ApplicationArchive::open(path)?))
    }
    pub fn seal_for_transfer(
        self,
        target: Particle,
        nonce: Particle,
    ) -> Result<super::Transfer, Error> {
        super::Transfer::from_archive(self.0, target, nonce)
    }
    pub fn sources(&self) -> &[(Particle, Head)] {
        self.0.sources()
    }
    pub fn seal(&self) -> Option<&ArchiveSeal> {
        self.0.seal()
    }
    pub fn last_transaction(&self) -> Option<Particle> {
        self.0.last_transaction()
    }
    pub fn head(&self, namespace: &Particle) -> Result<Option<Head>, Error> {
        Ok(self.0.head(namespace)?)
    }
    pub fn get(&self, id: &Particle) -> Result<Option<Content>, Error> {
        self.0
            .content(id, MAX_CONTENT_BYTES + 1)?
            .map(|bytes| Content::from_stored(*id, bytes).map_err(Error::from))
            .transpose()
    }
    pub fn resolve(&self, namespace: &Particle, request: &Particle) -> Result<Option<Head>, Error> {
        Ok(self.0.resolve(namespace, request)?.map(|(_, head)| head))
    }
    pub fn history(
        &self,
        namespace: &Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error> {
        Ok(self.0.history(namespace, after, limit)?)
    }
    pub fn inspect(&self, rows: u64, bytes: u64) -> Result<ArchiveSummary, Error> {
        Ok(self.0.inspect(rows, bytes, |id, bytes, archive| {
            let content =
                Content::from_stored(id, bytes.to_vec()).map_err(|_| StoreError::Corrupt)?;
            if let Some((left, right)) = content.children() {
                for child in [left, right] {
                    let bytes = archive
                        .content(&child, MAX_CONTENT_BYTES + 1)?
                        .ok_or(StoreError::Corrupt)?;
                    Content::from_stored(child, bytes).map_err(|_| StoreError::Corrupt)?;
                }
            }
            Ok(())
        })?)
    }
}
