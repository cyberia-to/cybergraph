use super::{Change, Entry, Error, valid_path};
use crate::{
    Particle,
    application::Head,
    content::{Codec, Content},
};

const NODE: &[u8] = b"cybergraph/catalog/node\0";
const STATE: &[u8] = b"cybergraph/catalog/state\0";
const EVENT: &[u8] = b"cybergraph/catalog/change\0";

pub(super) enum Node {
    Leaf(String, Entry),
    Branch {
        bit: u32,
        left: Particle,
        right: Particle,
    },
}

pub(super) fn node(value: &Node) -> Result<Content, Error> {
    let mut bytes = NODE.to_vec();
    match value {
        Node::Leaf(path, entry) => {
            bytes.push(0);
            string(&mut bytes, path);
            binding(&mut bytes, *entry);
        }
        Node::Branch { bit, left, right } => {
            bytes.push(1);
            bytes.extend(bit.to_be_bytes());
            bytes.extend(left);
            bytes.extend(right);
        }
    }
    Ok(Content::new(Codec::Blob, bytes)?)
}
pub(super) fn read_node(content: &Content) -> Result<Node, Error> {
    let mut input = Input::new(content, NODE)?;
    let node = match input.byte()? {
        0 => {
            let length = u32::from_be_bytes(input.array()?) as usize;
            let path = std::str::from_utf8(input.take(length)?)
                .map_err(|_| Error::Corrupt)?
                .to_owned();
            valid_path(&path).map_err(|_| Error::Corrupt)?;
            let entry = Entry {
                binding: input.array()?,
                particle: input.array()?,
                revision: input.array()?,
            };
            Node::Leaf(path, entry)
        }
        1 => {
            let bit = u32::from_be_bytes(input.array()?);
            if bit as usize > super::MAX_PATH_BYTES * 9 {
                return Err(Error::Corrupt);
            }
            Node::Branch {
                bit,
                left: input.array()?,
                right: input.array()?,
            }
        }
        _ => return Err(Error::Corrupt),
    };
    input.end()?;
    Ok(node)
}
pub(super) fn state(
    namespace: Particle,
    index: u64,
    previous: Option<Head>,
    event: Particle,
    root: Option<Particle>,
) -> Result<Content, Error> {
    let mut bytes = STATE.to_vec();
    bytes.extend(namespace);
    bytes.extend(index.to_be_bytes());
    prior(&mut bytes, previous);
    bytes.extend(event);
    match root {
        Some(id) => {
            bytes.push(1);
            bytes.extend(id);
        }
        None => bytes.push(0),
    }
    Ok(Content::new(Codec::Blob, bytes)?)
}
pub(super) fn read_state(
    content: &Content,
    namespace: Particle,
    index: u64,
) -> Result<Option<Particle>, Error> {
    let mut input = Input::new(content, STATE)?;
    if input.array::<32>()? != namespace || u64::from_be_bytes(input.array()?) != index {
        return Err(Error::Corrupt);
    }
    match input.byte()? {
        0 if index == 0 => {}
        1 if index > 0 => {
            let previous = u64::from_be_bytes(input.array()?);
            if previous != index - 1 {
                return Err(Error::Corrupt);
            }
            input.take(32)?;
        }
        _ => return Err(Error::Corrupt),
    }
    input.take(32)?;
    let root = match input.byte()? {
        0 => None,
        1 => Some(input.array()?),
        _ => return Err(Error::Corrupt),
    };
    input.end()?;
    Ok(root)
}
pub(super) fn event(
    namespace: Particle,
    request: Particle,
    expected: Option<Head>,
    change: Change<'_>,
) -> Result<Content, Error> {
    let mut bytes = EVENT.to_vec();
    bytes.extend(namespace);
    bytes.extend(request);
    prior(&mut bytes, expected);
    match change {
        Change::Create { path, particle } => {
            valid_path(path)?;
            bytes.push(0);
            string(&mut bytes, path);
            bytes.extend(particle);
        }
        Change::Edit {
            path,
            expected,
            particle,
        } => {
            valid_path(path)?;
            bytes.push(1);
            string(&mut bytes, path);
            binding(&mut bytes, expected);
            bytes.extend(particle);
        }
        Change::Rename { from, to, expected } => {
            valid_path(from)?;
            valid_path(to)?;
            bytes.push(2);
            string(&mut bytes, from);
            string(&mut bytes, to);
            binding(&mut bytes, expected);
        }
        Change::Remove { path, expected } => {
            valid_path(path)?;
            bytes.push(3);
            string(&mut bytes, path);
            binding(&mut bytes, expected);
        }
    }
    Ok(Content::new(Codec::Blob, bytes)?)
}
fn prior(bytes: &mut Vec<u8>, head: Option<Head>) {
    match head {
        Some(head) => {
            bytes.push(1);
            bytes.extend(head.index.to_be_bytes());
            bytes.extend(head.commit);
        }
        None => bytes.push(0),
    }
}
fn string(bytes: &mut Vec<u8>, text: &str) {
    bytes.extend((text.len() as u32).to_be_bytes());
    bytes.extend(text.as_bytes());
}
fn binding(bytes: &mut Vec<u8>, entry: Entry) {
    bytes.extend(entry.binding);
    bytes.extend(entry.particle);
    bytes.extend(entry.revision);
}
struct Input<'a>(&'a [u8]);
impl<'a> Input<'a> {
    fn new(content: &'a Content, prefix: &[u8]) -> Result<Self, Error> {
        if content.codec() != Codec::Blob {
            return Err(Error::Corrupt);
        }
        Ok(Self(
            content.bytes().strip_prefix(prefix).ok_or(Error::Corrupt)?,
        ))
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let bytes = self.0.get(..n).ok_or(Error::Corrupt)?;
        self.0 = &self.0[n..];
        Ok(bytes)
    }
    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| Error::Corrupt)
    }
    fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.array::<1>()?[0])
    }
    fn end(self) -> Result<(), Error> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Error::Corrupt)
        }
    }
}
