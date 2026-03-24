use std::io;

use async_compression::{Level, tokio::bufread::GzipEncoder};
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use tokio::io::BufReader;
use tokio_util::io::{ReaderStream, StreamReader};

const COMPRESS_BUFFER_SIZE: usize = 1024 * 4;

/// Compresses the poison stream with gzip encoding.
pub fn gzip_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>>,
) -> impl Stream<Item = Result<Bytes, io::Error>> {
    let stream = stream.map_err(|e| io::Error::new(io::ErrorKind::Other, e));
    let reader = StreamReader::new(stream);
    let buf = BufReader::with_capacity(COMPRESS_BUFFER_SIZE, reader);
    let encoder = GzipEncoder::with_quality(buf, Level::Fastest);
    ReaderStream::new(encoder)
}
