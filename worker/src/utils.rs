use cfg_if::cfg_if;
use worker::wasm_bindgen::JsCast;
use worker::worker_sys::WorkerGlobalScope;
use worker::{js_sys, Result};

cfg_if! {
    // https://github.com/rustwasm/console_error_panic_hook#readme
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

pub fn btoa(s: &str) -> Result<String> {
    let worker: WorkerGlobalScope = js_sys::global().unchecked_into();
    Ok(worker.btoa(s)?)
}

pub fn basic_auth(username: &str, password: &str) -> Result<String> {
    let mut s = username.to_owned();
    s.push(':');
    s.push_str(password);

    let mut result = "Basic ".to_owned();
    result.push_str(&btoa(&s)?);
    Ok(result)
}
