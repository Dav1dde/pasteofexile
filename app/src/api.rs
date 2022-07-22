use crate::{Error, Result};
use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize};
use shared::model::{Paste, PasteId, PasteSummary};

#[derive(Debug, Deserialize)]
pub struct PasteResponse {
    pub id: String,
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[derive(Serialize)]
pub struct CreatePaste<'a> {
    pub as_user: bool,
    pub content: &'a str,
    pub title: &'a str,
    pub id: Option<&'a PasteId>,
}

#[allow(dead_code)] // Only used in !SSR
pub async fn create_paste(content: CreatePaste<'_>) -> Result<PasteResponse> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::post("/api/internal/paste/")
        .body(serde_json::to_string(&content)?)
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json::<PasteResponse>().await?)
}

pub async fn get_paste(id: &PasteId) -> Result<Paste> {
    let _in_flight = crate::progress::start_request();
    let path = id.to_json_url();

    let resp = Request::get(&path).send().await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("paste", id.to_string()));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json().await?)
}

#[cfg(not(feature = "ssr"))]
pub async fn delete_paste(id: &PasteId) -> Result<()> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::delete(&format!("/api/internal/paste/{id}"))
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(())
}

pub async fn get_user(user: &str) -> Result<Vec<PasteSummary>> {
    let _in_flight = crate::progress::start_request();
    let resp = Request::get(&format!("/api/internal/user/{user}"))
        .send()
        .await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("user", user.to_string()));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json().await?)
}

async fn handle_error_response(resp: Response) -> Error {
    if let Ok(err) = resp.json::<ErrorResponse>().await {
        Error::ApiError(err.code, err.message)
    } else {
        Error::UnhandledStatus(resp.status(), resp.status_text())
    }
}
