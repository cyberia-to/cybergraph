//! Verified local text observations in the shared application graph.
mod legacy;
use crate::{
    Particle,
    application::{ApplicationGraph, Database, Error, Head, Proposal},
    content::{Codec, Content, MAX_CONTENT_BYTES},
};
pub use legacy::{ImportReport, MAX_SOURCE_BYTES};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_PROJECTION_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_OBSERVATIONS: u64 = 1_000_000;
const SCHEMA: &str = "cybergraph/text-observation/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub source: Particle,
    pub line: u64,
    pub raw: Particle,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Observation {
    schema: String,
    previous: Option<Particle>,
    request: Particle,
    text: Particle,
    created: Option<u64>,
    provenance: Option<Provenance>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextMeta {
    pub text: String,
    pub created: Option<u64>,
    pub observation: Particle,
    pub provenance: Option<Provenance>,
}

pub struct TextArchive {
    graph: ApplicationGraph,
    namespace: Particle,
}
enum PreparedObservation {
    Existing(Head),
    Append(Proposal),
}
impl TextArchive {
    pub fn from_database(database: Database, namespace: Particle) -> Self {
        Self {
            graph: ApplicationGraph::from_database(database),
            namespace,
        }
    }
    pub fn remember(
        &self,
        request: Particle,
        text: &str,
        created: Option<u64>,
    ) -> Result<Head, Error> {
        let expected = self.graph.head(&self.namespace)?;
        let prepared = self.prepare(request, text, created, None, expected)?;
        if let PreparedObservation::Append(_) = &prepared {
            let (mut sizes, mut bytes) = self.projection_sizes(expected)?;
            Self::extend_projection(
                &mut sizes,
                &mut bytes,
                *hemera::hash(text.as_bytes()).as_bytes(),
                text.len(),
            )?;
        }
        self.publish(prepared)
    }
    fn prepare_import_line(
        &self,
        source: Particle,
        line: u64,
        raw: &[u8],
        text: &str,
        created: Option<u64>,
        expected: Option<Head>,
    ) -> Result<PreparedObservation, Error> {
        if line == 0 || raw.len() > MAX_TEXT_BYTES {
            return Err(Error::InvalidProposal);
        }
        let mut hash = hemera::Hasher::new();
        hash.update(b"cybergraph/text-import-request/1\0");
        hash.update(&source);
        hash.update(&line.to_le_bytes());
        self.prepare(
            *hash.finalize().as_bytes(),
            text,
            created,
            Some((source, line, raw)),
            expected,
        )
    }
    fn prepare(
        &self,
        request: Particle,
        text: &str,
        created: Option<u64>,
        origin: Option<(Particle, u64, &[u8])>,
        expected: Option<Head>,
    ) -> Result<PreparedObservation, Error> {
        if text.len() > MAX_TEXT_BYTES {
            return Err(Error::InvalidProposal);
        }
        let content = Content::new(Codec::Blob, text.as_bytes().to_vec())?;
        let raw = origin
            .map(|(_, _, bytes)| Content::new(Codec::Blob, bytes.to_vec()))
            .transpose()?;
        let provenance = origin
            .zip(raw.as_ref())
            .map(|((source, line, _), raw)| Provenance {
                source,
                line,
                raw: raw.id(),
            });
        if let Some(head) = self.graph.resolve(&self.namespace, &request)? {
            let prior = self.read(head.commit)?;
            return if prior.request == request
                && prior.text == content.id()
                && prior.created == created
                && prior.provenance == provenance
            {
                Ok(PreparedObservation::Existing(head))
            } else {
                Err(Error::InvalidProposal)
            };
        }
        let record = Observation {
            schema: SCHEMA.into(),
            previous: expected.map(|h| h.commit),
            request,
            text: content.id(),
            created,
            provenance,
        };
        let bytes = serde_json::to_vec(&record).map_err(|e| Error::Rejected(e.to_string()))?;
        let record_content = Content::new(Codec::Blob, bytes)?;
        let head = Head {
            index: expected.map_or(Ok(0), |h| {
                h.index.checked_add(1).ok_or(Error::InvalidProposal)
            })?,
            commit: record_content.id(),
        };
        if head.index >= MAX_OBSERVATIONS {
            return Err(Error::Rejected("text observation count limit".into()));
        }
        let mut required = vec![content.id()];
        let mut contents = BTreeMap::from([
            (content.id(), content),
            (record_content.id(), record_content),
        ]);
        if let Some(raw) = raw {
            required.push(raw.id());
            contents.insert(raw.id(), raw);
        }
        if let Some(old) = expected {
            required.push(old.commit);
        }
        // The owner counts every codec byte as well as the text, raw source and
        // canonical observation. Validate this whole atomic unit during preflight.
        let mut bytes = 0usize;
        for content in contents.values() {
            bytes = bytes
                .checked_add(content.bytes().len() + 1)
                .ok_or(Error::InvalidProposal)?;
        }
        if bytes > MAX_CONTENT_BYTES {
            return Err(Error::Rejected(
                "text observation transaction byte limit".into(),
            ));
        }
        Ok(PreparedObservation::Append(Proposal {
            namespace: self.namespace,
            request,
            expected,
            head,
            content: contents.into_values().collect(),
            required,
            claims: vec![],
        }))
    }
    fn publish(&self, prepared: PreparedObservation) -> Result<Head, Error> {
        match prepared {
            PreparedObservation::Existing(head) => Ok(head),
            PreparedObservation::Append(proposal) => self.graph.commit(&proposal, |_| Ok(())),
        }
    }
    fn projection_sizes(
        &self,
        selected: Option<Head>,
    ) -> Result<(BTreeMap<Particle, usize>, usize), Error> {
        let mut bytes = 0;
        let sizes = self
            .load_at(selected)?
            .into_iter()
            .map(|(id, meta)| {
                bytes += meta.text.len();
                (id, meta.text.len())
            })
            .collect();
        Ok((sizes, bytes))
    }
    fn extend_projection(
        sizes: &mut BTreeMap<Particle, usize>,
        bytes: &mut usize,
        id: Particle,
        len: usize,
    ) -> Result<(), Error> {
        if sizes.contains_key(&id) {
            return Ok(());
        }
        let total = bytes.checked_add(len).ok_or(Error::InvalidProposal)?;
        if total > MAX_PROJECTION_BYTES {
            return Err(Error::Rejected("text projection byte limit".into()));
        }
        sizes.insert(id, len);
        *bytes = total;
        Ok(())
    }
    fn read(&self, id: Particle) -> Result<Observation, Error> {
        let content = self.graph.get(&id)?.ok_or(Error::MissingContent(id))?;
        if content.codec() != Codec::Blob || content.bytes().len() > 8192 {
            return Err(Error::InvalidProposal);
        }
        let record: Observation =
            serde_json::from_slice(content.bytes()).map_err(|e| Error::Rejected(e.to_string()))?;
        if record.schema != SCHEMA
            || serde_json::to_vec(&record).map_err(|e| Error::Rejected(e.to_string()))?
                != content.bytes()
        {
            return Err(Error::InvalidProposal);
        }
        if record.provenance.as_ref().is_some_and(|p| p.line == 0) {
            return Err(Error::InvalidProposal);
        }
        Ok(record)
    }
    /// A bounded projection at the selected immutable head. A concurrent append
    /// is observed by a later call; it cannot make this snapshot inconsistent.
    pub fn load(&self) -> Result<BTreeMap<Particle, TextMeta>, Error> {
        self.load_at(self.graph.head(&self.namespace)?)
    }
    fn load_at(&self, selected: Option<Head>) -> Result<BTreeMap<Particle, TextMeta>, Error> {
        let Some(selected) = selected else {
            return Ok(BTreeMap::new());
        };
        if selected.index >= MAX_OBSERVATIONS {
            return Err(Error::InvalidProposal);
        }
        let mut map: BTreeMap<Particle, TextMeta> = BTreeMap::new();
        let mut bytes = 0usize;
        let mut previous = None;
        let mut after = None;
        let mut index = 0u64;
        loop {
            let page = self.graph.history(&self.namespace, after, 512)?;
            if page.is_empty() {
                return Err(Error::InvalidProposal);
            }
            for head in page {
                if head.index != index {
                    return Err(Error::InvalidProposal);
                }
                let observation = self.read(head.commit)?;
                if observation.previous != previous {
                    return Err(Error::InvalidProposal);
                }
                let text = self
                    .graph
                    .get(&observation.text)?
                    .ok_or(Error::MissingContent(observation.text))?;
                if text.codec() != Codec::Blob || text.bytes().len() > MAX_TEXT_BYTES {
                    return Err(Error::InvalidProposal);
                }
                if let Some(provenance) = &observation.provenance {
                    let raw = self
                        .graph
                        .get(&provenance.raw)?
                        .ok_or(Error::MissingContent(provenance.raw))?;
                    if raw.codec() != Codec::Blob || raw.bytes().len() > MAX_TEXT_BYTES {
                        return Err(Error::InvalidProposal);
                    }
                }
                let text = String::from_utf8(text.bytes().to_vec())
                    .map_err(|e| Error::Rejected(e.to_string()))?;
                if let Some(prior) = map.get(&observation.text) {
                    bytes -= prior.text.len();
                }
                bytes = bytes
                    .checked_add(text.len())
                    .ok_or(Error::InvalidProposal)?;
                if bytes > MAX_PROJECTION_BYTES {
                    return Err(Error::InvalidProposal);
                }
                map.insert(
                    observation.text,
                    TextMeta {
                        text,
                        created: observation.created,
                        observation: head.commit,
                        provenance: observation.provenance,
                    },
                );
                if head.index == selected.index {
                    return if head.commit == selected.commit {
                        Ok(map)
                    } else {
                        Err(Error::InvalidProposal)
                    };
                }
                index += 1;
                previous = Some(head.commit);
                after = Some(head.index);
            }
        }
    }
}
