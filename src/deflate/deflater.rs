use libdeflater::*;

use crate::{PngError, PngResult};

/// Compresses `data` as zlib. With the `parallel` feature, large data is compressed in chunks on
/// several threads (see `libdeflater`'s `zlib_compress_parallel`), which costs a little
/// compression; data smaller than two chunks is compressed the same either way.
pub fn deflate(data: &[u8], level: u8, max_size: Option<usize>) -> PngResult<Vec<u8>> {
    let mut compressor = Compressor::new(CompressionLvl::new(level.into()).unwrap());
    #[cfg(feature = "parallel")]
    let bound = compressor.zlib_compress_bound_parallel(data.len());
    #[cfg(not(feature = "parallel"))]
    let bound = compressor.zlib_compress_bound(data.len());
    let capacity = max_size.unwrap_or(bound);
    let mut dest = vec![0; capacity];
    #[cfg(feature = "parallel")]
    let result = compressor.zlib_compress_parallel(data, &mut dest);
    #[cfg(not(feature = "parallel"))]
    let result = compressor.zlib_compress(data, &mut dest);
    let len = result
        .map_err(|err| match err {
            CompressionError::InsufficientSpace => PngError::DeflatedDataTooLong(capacity),
        })?;
    dest.truncate(len);
    Ok(dest)
}

pub fn inflate(data: &[u8], out_size: usize) -> PngResult<Vec<u8>> {
    let mut decompressor = Decompressor::new();
    let mut dest = vec![0; out_size];
    let len = decompressor
        .zlib_decompress(data, &mut dest)
        .map_err(|err| match err {
            DecompressionError::BadData => PngError::InvalidData,
            DecompressionError::InsufficientSpace => PngError::InflatedDataTooLong(out_size),
        })?;
    dest.truncate(len);
    Ok(dest)
}

#[must_use]
pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = Crc::new();
    crc.update(data);
    crc.sum()
}
