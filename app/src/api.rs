use reqwasm::http::{Request, Response};
use serde::Deserialize;
use std::rc::Rc;

#[derive(Debug, Deserialize)]
pub struct PasteResponse {
    pub id: String,
}

#[allow(dead_code)] // Only used in !SSR
pub async fn create_paste(content: Rc<String>) -> anyhow::Result<PasteResponse> {
    let resp = Request::post("/api/v1/paste/")
        .body(&*content)
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json::<PasteResponse>().await?)
}

pub async fn get_paste(id: String) -> anyhow::Result<String> {
    let path = format!("/{}/raw", id);
    let resp = Request::get(&path).send().await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.text().await?)
}

async fn handle_error_response(resp: Response) -> anyhow::Error {
    anyhow::anyhow!(resp.status_text())
}
