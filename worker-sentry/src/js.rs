use wasm_bindgen::JsCast;

pub fn get_random_values<const N: usize>() -> [u8; N] {
    let worker: web_sys::WorkerGlobalScope = js_sys::global().unchecked_into();

    let mut result = [0; N];
    worker
        .crypto()
        .unwrap()
        .get_random_values_with_u8_array(&mut result)
        .unwrap();

    result
}
