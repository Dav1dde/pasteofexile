use crate::{consts, Result};
use worker::{Env, Response};

pub async fn get(env: &Env, path: &str) -> Result<Option<Response>> {
    let kv = env.kv(consts::KV_PASTE_STORAGE)?;

    let data = kv.get(path).text().await?;
    Ok(data.map(|data| Response::ok(data).unwrap()))
}

pub async fn put(env: &Env, filename: &str, _sha1: &[u8], data: &mut [u8]) -> Result<()> {
    let kv = env.kv(consts::KV_PASTE_STORAGE)?;

    kv.put_bytes(filename, data)?.execute().await?;

    Ok(())
}
