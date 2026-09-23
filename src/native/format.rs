//! Compatibility is established by full replay, never by changing stored roots.
use super::*;
use bbg::{
    storage::database::Transaction,
    transition::{MetadataError, NATIVE_RECORD_VERSION, native_metadata_version},
};

pub(super) const MAX_METADATA: usize = 256;

pub(super) fn read(db: &Database) -> Result<Vec<u8>, Error> {
    let bytes = db
        .read_record(D::NativeState, &[0], MAX_METADATA)?
        .ok_or_else(|| corrupt("missing native state metadata"))?;
    validate(&bytes)?;
    Ok(bytes)
}

fn validate(bytes: &[u8]) -> Result<(), Error> {
    native_metadata_version(bytes).map(|_| ()).map_err(|error| match error {
        MetadataError::Malformed => corrupt("malformed native state metadata"),
        MetadataError::Unsupported(version) => Error::Unsupported(format!(
            "native state format {version}; this binary reads legacy 1 and {NATIVE_RECORD_VERSION}; preserve the store and use a compatible binary"
        )),
    })
}

pub(super) fn legacy(bytes: &[u8]) -> bool {
    bytes.starts_with(&1u32.to_le_bytes())
}

pub(super) fn equivalent(stored: &[u8], current: &[u8]) -> bool {
    stored == current
        || (legacy(stored)
            && current.starts_with(&NATIVE_RECORD_VERSION.to_le_bytes())
            && stored.get(4..) == current.get(4..))
}

pub(super) fn recovery_error(metadata: &[u8], error: Error) -> Error {
    if legacy(metadata)
        && let Error::Corrupt(detail) = error
    {
        return Error::Unsupported(format!(
            "legacy native state format 1 does not replay under current commitment semantics ({detail}); preserve the store and use its original writer for diagnosis or explicit export"
        ));
    }
    error
}

pub(super) fn require_unchanged(tx: &mut Transaction<'_>, expected: &[u8]) -> Result<(), Error> {
    if tx
        .read_record(D::NativeState, &[0], MAX_METADATA)?
        .as_deref()
        != Some(expected)
    {
        return Err(corrupt(
            "native state metadata changed outside its coordinator",
        ));
    }
    Ok(())
}

pub(super) fn promote(tx: &mut Transaction<'_>) -> Result<Vec<u8>, Error> {
    let mut bytes = tx
        .read_record(D::NativeState, &[0], MAX_METADATA)?
        .ok_or_else(|| corrupt("missing native state metadata"))?;
    validate(&bytes)?;
    if legacy(&bytes) {
        bytes[..4].copy_from_slice(&NATIVE_RECORD_VERSION.to_le_bytes());
        tx.put_record(D::NativeState, &[0], &bytes)?;
    }
    Ok(bytes)
}
