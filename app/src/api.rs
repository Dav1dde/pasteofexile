use crate::{Error, Result};
use reqwasm::http::{Request, Response};
use serde::Deserialize;
use std::rc::Rc;

#[derive(Debug, Deserialize)]
pub struct PasteResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[allow(dead_code)] // Only used in !SSR
pub async fn create_paste(content: Rc<String>) -> Result<PasteResponse> {
    let resp = Request::post("/api/v1/paste/")
        .body(&*content)
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json::<PasteResponse>().await?)
}

pub async fn get_paste(id: String) -> Result<String> {
    let path = format!("/{}/raw", id);
    let resp = Request::get(&path).send().await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("paste", id));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.text().await?)
}

async fn handle_error_response(resp: Response) -> Error {
    if let Ok(err) = resp.json::<ErrorResponse>().await {
        Error::ApiError(err.code, err.message)
    } else {
        Error::UnhandledStatus(resp.status(), resp.status_text())
    }
}
