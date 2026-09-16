//! Verified graph content using existing stack codecs.
use crate::Particle;

pub const MAX_CONTENT_BYTES: usize = 16 * 1024 * 1024;
const FIELD_P: u64 = 0xffff_ffff_0000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Codec {
    Data,
    Blob,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentError {
    InvalidData,
    NoncanonicalParticle,
    TooLarge,
    IdentityMismatch,
}

#[derive(Debug, Clone)]
pub struct Content {
    id: Particle,
    codec: Codec,
    bytes: Vec<u8>,
}

impl Content {
    pub fn new(codec: Codec, bytes: Vec<u8>) -> Result<Self, ContentError> {
        if bytes.len() > MAX_CONTENT_BYTES {
            return Err(ContentError::TooLarge);
        }
        let id = match codec {
            Codec::Blob => *hemera::hash(&bytes).as_bytes(),
            Codec::Data => {
                if bytes.len() == 64 {
                    for limb in bytes.chunks_exact(8) {
                        let value = u64::from_le_bytes(
                            limb.try_into().map_err(|_| ContentError::InvalidData)?,
                        );
                        if value >= FIELD_P {
                            return Err(ContentError::NoncanonicalParticle);
                        }
                    }
                }
                nox::encode::particle_of(&bytes).map_err(|_| ContentError::InvalidData)?
            }
        };
        Ok(Self { id, codec, bytes })
    }
    pub fn atom(value: u64) -> Result<Self, ContentError> {
        Self::new(Codec::Data, value.to_le_bytes().to_vec())
    }
    pub fn pair(left: Particle, right: Particle) -> Result<Self, ContentError> {
        Self::new(
            Codec::Data,
            nox::encode::encode_pair(&left, &right).to_vec(),
        )
    }
    pub fn id(&self) -> Particle {
        self.id
    }
    pub fn codec(&self) -> Codec {
        self.codec
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn children(&self) -> Option<(Particle, Particle)> {
        if self.codec != Codec::Data || self.bytes.len() != 64 {
            return None;
        }
        Some((
            self.bytes[..32].try_into().ok()?,
            self.bytes[32..].try_into().ok()?,
        ))
    }
    #[cfg(feature = "local-storage")]
    pub(crate) fn stored(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.bytes.len() + 1);
        bytes.push(match self.codec {
            Codec::Data => 0,
            Codec::Blob => 1,
        });
        bytes.extend_from_slice(&self.bytes);
        bytes
    }
    #[cfg(feature = "local-storage")]
    pub(crate) fn from_stored(id: Particle, stored: Vec<u8>) -> Result<Self, ContentError> {
        let codec = match stored.first() {
            Some(0) => Codec::Data,
            Some(1) => Codec::Blob,
            _ => return Err(ContentError::InvalidData),
        };
        let content = Self::new(codec, stored[1..].to_vec())?;
        if content.id != id {
            return Err(ContentError::IdentityMismatch);
        }
        Ok(content)
    }
}
