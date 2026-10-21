//! Authorized Radio file capabilities over the existing Cybergraph/BBG owner.
mod sink;
mod source;

pub use radio::files::{ALPN, Client, Descriptor, FileId, FileProtocol};
pub use sink::{FileSink, Page, receive_page};
pub use source::FileSource;

use std::io;

async fn blocking<T: Send + 'static>(
    job: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    tokio::task::spawn_blocking(job)
        .await
        .map_err(io::Error::other)?
}

fn storage(error: cybergraph::files::Error) -> io::Error {
    io::Error::other(error)
}
