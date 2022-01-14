use web_sys::WorkerGlobalScope;
use worker::wasm_bindgen::JsCast;
use worker::wasm_bindgen_futures::JsFuture;
use worker::{js_sys, Result};

pub async fn sha1(data: &mut [u8]) -> Result<Vec<u8>> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    let digest = JsFuture::from(
        worker
            .crypto()?
            .subtle()
            .digest_with_str_and_u8_array("SHA-1", data)?,
    )
    .await?;
    assert!(digest.is_instance_of::<js_sys::ArrayBuffer>());
    Ok(js_sys::Uint8Array::new(&digest).to_vec())
}

pub fn get_random_values<const N: usize>() -> Result<[u8; N]> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();

    let mut result = [0; N];
    worker
        .crypto()?
        .get_random_values_with_u8_array(&mut result)?;
    Ok(result)
}
