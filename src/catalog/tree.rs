use super::{
    ContentMap, Entry, Error, Page,
    codec::{self, Node},
};
use crate::{Particle, application::ApplicationGraph};
use std::collections::BTreeMap;

pub(super) struct Tree<'a> {
    graph: &'a ApplicationGraph,
    pub content: ContentMap,
    pub required: BTreeMap<Particle, ()>,
}
struct Frame {
    bit: u32,
    sibling: Particle,
    right: bool,
}
impl<'a> Tree<'a> {
    pub fn new(graph: &'a ApplicationGraph) -> Self {
        Self {
            graph,
            content: BTreeMap::new(),
            required: BTreeMap::new(),
        }
    }
    fn load(&self, id: Particle) -> Result<Node, Error> {
        match self.content.get(&id) {
            Some(content) => codec::read_node(content),
            None => codec::read_node(&self.graph.get(&id)?.ok_or(Error::Corrupt)?),
        }
    }
    fn store(&mut self, node: Node) -> Result<Particle, Error> {
        if let Node::Branch { left, right, .. } = &node {
            self.required.insert(*left, ());
            self.required.insert(*right, ());
        }
        let content = codec::node(&node)?;
        let id = content.id();
        self.content.insert(id, content);
        Ok(id)
    }
    pub fn get(&self, root: Option<Particle>, path: &str) -> Result<Option<Entry>, Error> {
        let Some(mut id) = root else {
            return Ok(None);
        };
        let mut previous = None;
        loop {
            match self.load(id)? {
                Node::Leaf(key, entry) => return Ok((key == path).then_some(entry)),
                Node::Branch { bit, left, right } => {
                    increasing(previous, bit)?;
                    previous = Some(bit);
                    id = if path_bit(path, bit) { right } else { left };
                }
            }
        }
    }
    pub fn set(
        &mut self,
        root: Option<Particle>,
        path: &str,
        value: Option<Entry>,
    ) -> Result<Option<Particle>, Error> {
        let Some(root) = root else {
            return value
                .map(|entry| self.store(Node::Leaf(path.to_owned(), entry)))
                .transpose();
        };
        let mut id = root;
        let mut frames = Vec::new();
        let existing = loop {
            match self.load(id)? {
                Node::Leaf(key, _) => break key,
                Node::Branch { bit, left, right } => {
                    increasing(frames.last().map(|f: &Frame| f.bit), bit)?;
                    let goes_right = path_bit(path, bit);
                    frames.push(Frame {
                        bit,
                        sibling: if goes_right { left } else { right },
                        right: goes_right,
                    });
                    id = if goes_right { right } else { left };
                }
            }
        };
        let next = if existing == path {
            match value {
                Some(entry) => self.store(Node::Leaf(path.to_owned(), entry))?,
                None => match frames.pop() {
                    Some(frame) => frame.sibling,
                    None => return Ok(None),
                },
            }
        } else {
            let Some(entry) = value else {
                return Ok(Some(root));
            };
            let split = difference(&existing, path)?;
            // Reuse the existing subtree where the first differing path bit falls.
            while frames.last().is_some_and(|frame| frame.bit >= split) {
                frames.pop();
            }
            let mut subtree = root;
            for frame in &frames {
                match self.load(subtree)? {
                    Node::Branch { left, right, .. } => {
                        subtree = if frame.right { right } else { left }
                    }
                    Node::Leaf(..) => return Err(Error::Corrupt),
                }
            }
            let leaf = self.store(Node::Leaf(path.to_owned(), entry))?;
            let (left, right) = if path_bit(path, split) {
                (subtree, leaf)
            } else {
                (leaf, subtree)
            };
            self.store(Node::Branch {
                bit: split,
                left,
                right,
            })?
        };
        let mut next = next;
        while let Some(frame) = frames.pop() {
            let (left, right) = if frame.right {
                (frame.sibling, next)
            } else {
                (next, frame.sibling)
            };
            next = self.store(Node::Branch {
                bit: frame.bit,
                left,
                right,
            })?;
        }
        Ok(Some(next))
    }
    pub fn list(
        &self,
        root: Option<Particle>,
        after: Option<&str>,
        limit: usize,
    ) -> Result<Page, Error> {
        let Some(root) = root else {
            return if after.is_some() {
                Err(Error::InvalidCursor)
            } else {
                Ok(Page {
                    entries: Vec::new(),
                    next: None,
                })
            };
        };
        let mut pending = Vec::new();
        if let Some(path) = after {
            let mut id = root;
            let mut previous = None;
            loop {
                match self.load(id)? {
                    Node::Leaf(key, _) => {
                        if key != path {
                            return Err(Error::InvalidCursor);
                        }
                        break;
                    }
                    Node::Branch { bit, left, right } => {
                        increasing(previous, bit)?;
                        previous = Some(bit);
                        if path_bit(path, bit) {
                            id = right;
                        } else {
                            pending.push((right, Some(bit)));
                            id = left;
                        }
                    }
                }
            }
        } else {
            pending.push((root, None));
        }
        let mut entries = Vec::new();
        while let Some((mut id, mut previous)) = pending.pop() {
            loop {
                match self.load(id)? {
                    Node::Leaf(path, entry) => {
                        if entries.len() == limit {
                            return Ok(Page {
                                next: entries
                                    .last()
                                    .map(|(path, _): &(String, Entry)| path.clone()),
                                entries,
                            });
                        }
                        entries.push((path, entry));
                        break;
                    }
                    Node::Branch { bit, left, right } => {
                        increasing(previous, bit)?;
                        previous = Some(bit);
                        pending.push((right, previous));
                        id = left;
                    }
                }
            }
        }
        Ok(Page {
            entries,
            next: None,
        })
    }
}
fn increasing(previous: Option<u32>, bit: u32) -> Result<(), Error> {
    if previous.is_some_and(|previous| previous >= bit) {
        Err(Error::Corrupt)
    } else {
        Ok(())
    }
}
fn path_bit(path: &str, bit: u32) -> bool {
    let bit = bit as usize;
    match path.as_bytes().get(bit / 9) {
        None => false,
        Some(_) if bit.is_multiple_of(9) => true,
        Some(byte) => byte & (1 << (8 - bit % 9)) != 0,
    }
}
fn difference(left: &str, right: &str) -> Result<u32, Error> {
    let same = left
        .bytes()
        .zip(right.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    let first = same * 9;
    for bit in first..first + 9 {
        if path_bit(left, bit as u32) != path_bit(right, bit as u32) {
            return Ok(bit as u32);
        }
    }
    Err(Error::Corrupt)
}
