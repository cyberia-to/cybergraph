//! Versioned local file names over one shared application database.
mod codec;
mod tree;

use crate::{
    Particle,
    application::{ApplicationGraph, Head, Proposal},
    content::Content,
};
use std::{collections::BTreeMap, fmt};

/// Maximum encoded bytes in one path, independent of collection size.
pub const MAX_PATH_BYTES: usize = 4096;
/// Maximum entries returned by one listing or history request.
pub const MAX_PAGE_ENTRIES: usize = 4096;

/// A stable name binding and its current immutable payload revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Entry {
    pub binding: Particle,
    pub particle: Particle,
    pub revision: Particle,
}

/// One conditional namespace operation.
#[derive(Debug, Clone, Copy)]
pub enum Change<'a> {
    Create {
        path: &'a str,
        particle: Particle,
    },
    Edit {
        path: &'a str,
        expected: Entry,
        particle: Particle,
    },
    Rename {
        from: &'a str,
        to: &'a str,
        expected: Entry,
    },
    Remove {
        path: &'a str,
        expected: Entry,
    },
}

/// A bounded listing in bytewise path order at the selected head.
#[derive(Debug, PartialEq, Eq)]
pub struct Page {
    pub entries: Vec<(String, Entry)>,
    /// Last returned path when further entries exist in this same state.
    pub next: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    Application(crate::application::Error),
    InvalidPath,
    InvalidCursor,
    Conflict,
    Corrupt,
    Limit,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file catalog: {self:?}")
    }
}
impl std::error::Error for Error {}
impl From<crate::application::Error> for Error {
    fn from(error: crate::application::Error) -> Self {
        Self::Application(error)
    }
}
impl From<crate::content::ContentError> for Error {
    fn from(error: crate::content::ContentError) -> Self {
        Self::Application(crate::application::Error::Content(error))
    }
}

/// Namespace-scoped names borrowing the shared database owner.
pub struct Catalog<'a> {
    graph: &'a ApplicationGraph,
    namespace: Particle,
}
impl fmt::Debug for Catalog<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Catalog").finish_non_exhaustive()
    }
}
impl<'a> Catalog<'a> {
    pub fn new(graph: &'a ApplicationGraph, namespace: Particle) -> Self {
        Self { graph, namespace }
    }
    pub fn head(&self) -> Result<Option<Head>, Error> {
        Ok(self.graph.head(&self.namespace)?)
    }
    pub fn history(&self, after: Option<u64>, limit: usize) -> Result<Vec<Head>, Error> {
        Ok(self.graph.history(&self.namespace, after, limit)?)
    }
    pub fn resolve(&self, at: Option<Head>, path: &str) -> Result<Option<Entry>, Error> {
        valid_path(path)?;
        tree::Tree::new(self.graph).get(self.root(at)?, path)
    }
    pub fn list(&self, at: Option<Head>, after: Option<&str>, limit: usize) -> Result<Page, Error> {
        if limit == 0 || limit > MAX_PAGE_ENTRIES {
            return Err(Error::Limit);
        }
        if let Some(path) = after {
            valid_path(path)?;
        }
        tree::Tree::new(self.graph).list(self.root(at)?, after, limit)
    }
    /// Atomically publish one operation with history, payload retention and retry receipt.
    pub fn apply(
        &self,
        request: Particle,
        expected: Option<Head>,
        change: Change<'_>,
    ) -> Result<Head, Error> {
        let event = codec::event(self.namespace, request, expected, change)?;
        let mut tree = tree::Tree::new(self.graph);
        let mut root = self.root(expected)?;
        let mut blobs = Vec::new();
        match change {
            Change::Create { path, particle } => {
                if tree.get(root, path)?.is_some() {
                    return Err(Error::Conflict);
                }
                let entry = Entry {
                    binding: event.id(),
                    particle,
                    revision: event.id(),
                };
                root = tree.set(root, path, Some(entry))?;
                blobs.push(particle);
            }
            Change::Edit {
                path,
                expected,
                particle,
            } => {
                require_entry(tree.get(root, path)?, expected)?;
                let entry = Entry {
                    particle,
                    revision: event.id(),
                    ..expected
                };
                root = tree.set(root, path, Some(entry))?;
                blobs.push(particle);
            }
            Change::Rename { from, to, expected } => {
                require_entry(tree.get(root, from)?, expected)?;
                if tree.get(root, to)?.is_some() {
                    return Err(Error::Conflict);
                }
                root = tree.set(root, from, None)?;
                root = tree.set(root, to, Some(expected))?;
            }
            Change::Remove { path, expected } => {
                require_entry(tree.get(root, path)?, expected)?;
                root = tree.set(root, path, None)?;
            }
        }
        let index = match expected {
            Some(head) => head.index.checked_add(1).ok_or(Error::Limit)?,
            None => 0,
        };
        let state = codec::state(self.namespace, index, expected, event.id(), root)?;
        let head = Head {
            index,
            commit: state.id(),
        };
        let mut content = tree.content;
        content.insert(event.id(), event);
        content.insert(state.id(), state);
        let mut required = tree.required;
        if let Some(id) = root {
            required.insert(id, ());
        }
        if let Some(head) = expected {
            required.insert(head.commit, ());
        }
        let proposal = Proposal {
            namespace: self.namespace,
            request,
            expected,
            head,
            content: content.into_values().collect(),
            required: required.into_keys().collect(),
            claims: Vec::new(),
        };
        Ok(self
            .graph
            .commit_with_blobs(&proposal, &blobs, |_| Ok(()))?)
    }
    fn root(&self, at: Option<Head>) -> Result<Option<Particle>, Error> {
        match at {
            None => Ok(None),
            Some(head) => {
                let content = self.graph.get(&head.commit)?.ok_or(Error::Corrupt)?;
                codec::read_state(&content, self.namespace, head.index)
            }
        }
    }
}

fn require_entry(actual: Option<Entry>, expected: Entry) -> Result<(), Error> {
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(Error::Conflict)
    }
}
fn valid_path(path: &str) -> Result<(), Error> {
    if path.len() > MAX_PATH_BYTES {
        return Err(Error::Limit);
    }
    if path.is_empty()
        || path.contains('\0')
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(Error::InvalidPath);
    }
    Ok(())
}

type ContentMap = BTreeMap<Particle, Content>;
