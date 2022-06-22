use crate::{model::PasteSummary, Error, Result};
use reqwasm::http::{Request, Response};
use serde::{Deserialize, Serialize};
use std::fmt;

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
}

#[allow(dead_code)] // Only used in !SSR
pub async fn create_paste(content: CreatePaste<'_>) -> Result<PasteResponse> {
    let resp = Request::post("/api/internal/paste/")
        .body(serde_json::to_string(&content)?)
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.json::<PasteResponse>().await?)
}

pub enum PasteId<'a> {
    Paste(&'a str),
    UserPaste(&'a str, &'a str),
}

impl fmt::Display for PasteId<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Paste(id) => write!(f, "{id}"),
            Self::UserPaste(user, id) => write!(f, "{user}:{id}"),
        }
    }
}

pub async fn get_paste(id: PasteId<'_>) -> Result<String> {
    let path = match id {
        PasteId::Paste(id) => format!("/{id}/raw"),
        PasteId::UserPaste(user, id) => format!("/u/{user}/{id}/raw"),
    };
    let resp = Request::get(&path).send().await?;

    if resp.status() == 404 {
        return Err(Error::NotFound("paste", id.to_string()));
    }

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(resp.text().await?)
}

pub async fn delete_paste(id: PasteId<'_>) -> Result<()> {
    let resp = Request::delete(&format!("/api/internal/paste/{id}"))
        .send()
        .await?;

    if !resp.ok() {
        return Err(handle_error_response(resp).await);
    }

    Ok(())
}

pub async fn get_user(user: &str) -> Result<Vec<PasteSummary>> {
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
