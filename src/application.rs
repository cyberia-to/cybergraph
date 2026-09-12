//! Local graph application transactions, backed by BBG storage.
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use crate::{
    Particle,
    content::{Content, ContentError, MAX_CONTENT_BYTES},
};
use bbg::storage::application::{ApplicationStore, Write};
pub use bbg::storage::application::{Error as StorageError, Head};
pub use bbg::storage::database::{Backend, Database};

#[derive(Debug)]
pub enum Error {
    Storage(bbg::storage::application::Error),
    Content(ContentError),
    MissingContent(Particle),
    InvalidProposal,
    Rejected(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "application graph: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<bbg::storage::application::Error> for Error {
    fn from(e: bbg::storage::application::Error) -> Self {
        Self::Storage(e)
    }
}
impl From<ContentError> for Error {
    fn from(e: ContentError) -> Self {
        Self::Content(e)
    }
}

#[derive(Debug)]
pub struct Proposal {
    pub namespace: Particle,
    pub request: Particle,
    pub expected: Option<Head>,
    pub head: Head,
    pub content: Vec<Content>,
    /// Application references encoded as data, rather than structural children.
    pub required: Vec<Particle>,
    pub claims: Vec<(Particle, Particle)>,
}

pub struct ApplicationGraph {
    store: ApplicationStore,
}
impl ApplicationGraph {
    /// Open the default Fjall database directory.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        Ok(Self {
            store: ApplicationStore::open(path)?,
        })
    }
    /// Share the database's writer lock, backend and failure state with other views.
    pub fn from_database(database: Database) -> Self {
        Self {
            store: ApplicationStore::from_database(database),
        }
    }
    /// Import a legacy application redb file into a fresh Fjall directory.
    #[cfg(feature = "legacy-redb-migration")]
    pub fn migrate_redb(
        source: impl AsRef<Path>,
        destination: impl AsRef<Path>,
    ) -> Result<(), Error> {
        Ok(ApplicationStore::migrate_redb(source, destination)?)
    }
    pub fn head(&self, namespace: &Particle) -> Result<Option<Head>, Error> {
        Ok(self.store.head(namespace)?)
    }
    pub fn get(&self, id: &Particle) -> Result<Option<Content>, Error> {
        self.store
            .content(id, MAX_CONTENT_BYTES + 1)?
            .map(|bytes| Content::from_stored(*id, bytes).map_err(Error::from))
            .transpose()
    }
    pub fn resolve(&self, namespace: &Particle, request: &Particle) -> Result<Option<Head>, Error> {
        Ok(self
            .store
            .resolve(namespace, request)?
            .map(|(_, head)| head))
    }
    pub fn history(
        &self,
        namespace: &Particle,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<Head>, Error> {
        Ok(self.store.history(namespace, after, limit)?)
    }
    pub fn commit(
        &self,
        proposal: &Proposal,
        validate: impl FnOnce(&Self) -> Result<(), Error>,
    ) -> Result<Head, Error> {
        if proposal.content.len() > 131_072
            || proposal.required.len() > 131_072
            || proposal.claims.len() > 4096
        {
            return Err(Error::InvalidProposal);
        }
        let mut content = BTreeMap::new();
        let mut bytes = 0usize;
        for value in &proposal.content {
            bytes = bytes
                .checked_add(value.bytes().len() + 1)
                .ok_or(Error::InvalidProposal)?;
            if bytes > MAX_CONTENT_BYTES || content.insert(value.id(), value).is_some() {
                return Err(Error::InvalidProposal);
            }
        }
        let fingerprint = fingerprint(proposal, &content);
        if let Some((prior, head)) = self.store.resolve(&proposal.namespace, &proposal.request)? {
            return if prior == fingerprint {
                Ok(head)
            } else {
                Err(Error::Storage(bbg::storage::application::Error::Conflict))
            };
        }
        let require = |id: &Particle| -> Result<(), Error> {
            if !content.contains_key(id) && self.get(id)?.is_none() {
                return Err(Error::MissingContent(*id));
            }
            Ok(())
        };
        require(&proposal.head.commit)?;
        for id in &proposal.required {
            require(id)?;
        }
        for value in content.values() {
            if let Some((left, right)) = value.children() {
                require(&left)?;
                require(&right)?;
            }
        }
        validate(self)?;
        let stored: Vec<_> = content
            .into_iter()
            .map(|(id, value)| (id, value.stored()))
            .collect();
        Ok(self.store.apply(&Write {
            namespace: proposal.namespace,
            request: proposal.request,
            fingerprint,
            expected: proposal.expected,
            head: proposal.head,
            content: &stored,
            claims: &proposal.claims,
        })?)
    }
}

fn fingerprint(proposal: &Proposal, content: &BTreeMap<Particle, &Content>) -> Particle {
    // Transaction lookup identity is internal framing; application content uses
    // its declared codec independently. Counts bind the variable collections.
    let mut hash = hemera::Hasher::new();
    hash.update(b"cybergraph/application-transaction/1\0");
    hash.update(&proposal.namespace);
    hash.update(&proposal.request);
    hash.update(&[u8::from(proposal.expected.is_some())]);
    if let Some(head) = proposal.expected {
        hash.update(&head.index.to_le_bytes());
        hash.update(&head.commit);
    }
    hash.update(&proposal.head.index.to_le_bytes());
    hash.update(&proposal.head.commit);
    hash.update(&(content.len() as u64).to_le_bytes());
    for id in content.keys() {
        hash.update(id);
    }
    hash.update(&(proposal.required.len() as u64).to_le_bytes());
    for id in &proposal.required {
        hash.update(id);
    }
    hash.update(&(proposal.claims.len() as u64).to_le_bytes());
    for (key, value) in &proposal.claims {
        hash.update(key);
        hash.update(value);
    }
    *hash.finalize().as_bytes()
}
