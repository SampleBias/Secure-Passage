//! Chunked encrypted file transfer over MixnetStream.

use crate::crypto::SessionKey;
use crate::protocol::{read_frame, write_frame};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const FILE_MAGIC: &str = "SPFILE01";
pub const CHUNK_SIZE: usize = 256 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMeta {
    pub magic: String,
    pub file_name: String,
    pub size: u64,
    pub sha256: String,
    pub password_hash: Option<String>,
}

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn file_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub async fn send_file_meta<W: AsyncWrite + Unpin>(
    writer: &mut W,
    key: &SessionKey,
    meta: &FileMeta,
) -> Result<()> {
    let plain = serde_json::to_vec(meta)?;
    let enc = key.encrypt(&plain).context("encrypt file meta")?;
    write_frame(writer, &enc).await?;
    Ok(())
}

pub async fn recv_file_meta<R: AsyncRead + Unpin>(
    reader: &mut R,
    key: &SessionKey,
) -> Result<FileMeta> {
    let enc = read_frame(reader).await?;
    let plain = key.decrypt(&enc).context("decrypt file meta")?;
    let meta: FileMeta = serde_json::from_slice(&plain)?;
    if meta.magic != FILE_MAGIC {
        bail!("invalid file transfer magic");
    }
    Ok(meta)
}

/// Send encrypted chunks; invokes `on_progress(bytes_done, total)`.
pub async fn send_file_data<W: AsyncWrite + Unpin, F>(
    writer: &mut W,
    key: &SessionKey,
    data: &[u8],
    mut on_progress: F,
) -> Result<()>
where
    F: FnMut(u64, u64),
{
    let total = data.len() as u64;
    let mut done = 0u64;
    for chunk in data.chunks(CHUNK_SIZE) {
        let enc = key.encrypt(chunk).context("encrypt chunk")?;
        write_frame(writer, &enc).await?;
        done += chunk.len() as u64;
        on_progress(done, total);
    }
    // Empty frame signals end
    write_frame(writer, &[]).await?;
    Ok(())
}

pub async fn recv_file_data<R: AsyncRead + Unpin, F>(
    reader: &mut R,
    key: &SessionKey,
    expected_size: u64,
    mut on_progress: F,
) -> Result<Vec<u8>>
where
    F: FnMut(u64, u64),
{
    let mut out = Vec::with_capacity(expected_size as usize);
    loop {
        let enc = read_frame(reader).await?;
        if enc.is_empty() {
            break;
        }
        let plain = key.decrypt(&enc).context("decrypt chunk")?;
        out.extend_from_slice(&plain);
        on_progress(out.len() as u64, expected_size);
    }
    if out.len() as u64 != expected_size {
        bail!(
            "size mismatch: got {} expected {}",
            out.len(),
            expected_size
        );
    }
    Ok(out)
}

pub async fn write_bytes_to_path(path: &std::path::Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await.ok();
    }
    let mut f = tokio::fs::File::create(path).await?;
    f.write_all(data).await?;
    f.flush().await?;
    Ok(())
}

pub async fn read_bytes_from_path(path: &std::path::Path) -> Result<Vec<u8>> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await?;
    Ok(buf)
}
