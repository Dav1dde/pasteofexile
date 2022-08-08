use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsValue;
use web_sys::{CryptoKey, HmacImportParams, WorkerGlobalScope};
use worker::wasm_bindgen::JsCast;
use worker::wasm_bindgen_futures::JsFuture;
use worker::{js_sys, Result};

pub async fn sha1(data: &[u8]) -> Result<Vec<u8>> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let data = unsafe { Uint8Array::view(data) };
    let digest = JsFuture::from(
        worker
            .crypto()?
            .subtle()
            .digest_with_str_and_buffer_source("SHA-1", &data)?,
    )
    .await?;
    assert!(digest.is_instance_of::<js_sys::ArrayBuffer>());
    Ok(Uint8Array::new(&digest).to_vec())
}

pub async fn sign_hmac_256(secret: &[u8], payload: &mut [u8]) -> Result<Vec<u8>> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let subtle = worker.crypto()?.subtle();

    let secret = Uint8Array::from(secret);
    let algorithm = HmacImportParams::new("HMAC", &JsValue::from_str("SHA-256"));

    let usage = Array::of1(&JsValue::from_str("sign"));

    let key = subtle.import_key_with_object("raw", &secret.buffer(), &algorithm, false, &usage)?;
    let key = JsFuture::from(key).await?.unchecked_into::<CryptoKey>();

    let signed = subtle.sign_with_str_and_u8_array("HMAC", &key, payload)?;
    let signed = JsFuture::from(signed).await?;

    Ok(Uint8Array::new(&signed).to_vec())
}

pub async fn verify_hmac_256(
    secret: &[u8],
    signature: &mut [u8],
    payload: &mut [u8],
) -> Result<bool> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let subtle = worker.crypto()?.subtle();

    let secret = Uint8Array::from(secret);
    let algorithm = HmacImportParams::new("HMAC", &JsValue::from_str("SHA-256"));
    let usage = Array::of1(&JsValue::from_str("verify"));
    let key = subtle.import_key_with_object("raw", &secret.buffer(), &algorithm, false, &usage)?;
    let key = JsFuture::from(key).await?.unchecked_into::<CryptoKey>();

    let signed =
        subtle.verify_with_str_and_u8_array_and_u8_array("HMAC", &key, signature, payload)?;

    Ok(JsFuture::from(signed).await?.as_bool().unwrap_or(false))
}

pub fn get_random_values<const N: usize>() -> Result<[u8; N]> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();

    let mut result = [0; N];
    worker
        .crypto()?
        .get_random_values_with_u8_array(&mut result)?;
    Ok(result)
}
