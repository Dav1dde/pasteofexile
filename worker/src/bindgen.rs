use wasm_bindgen::{prelude::*, JsCast};
use worker::worker_sys;

#[wasm_bindgen]
extern "C" {
    pub type Response;

    #[wasm_bindgen(catch, constructor, js_class=Response)]
    pub fn new_with_opt_stream_and_init(
        body: Option<web_sys::ReadableStream>,
        init: &web_sys::ResponseInit,
    ) -> std::result::Result<Response, JsValue>;
}

impl Response {
    pub fn dup(
        response: worker::Response,
        headers: &worker::Headers,
    ) -> crate::Result<worker::Response> {
        let mut response_init = web_sys::ResponseInit::new();
        response_init.headers(&headers.0);

        let response: worker_sys::Response = response.into();
        let response = Response::new_with_opt_stream_and_init(response.body(), &response_init)?;
        let response = response.unchecked_into::<worker_sys::Response>();
        let body = worker::ResponseBody::Stream(response);

        Ok(worker::Response::from_body(body)?)
    }
}
