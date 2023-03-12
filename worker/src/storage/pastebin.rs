use shared::PasteId;

use super::StoredPaste;
use crate::{net, Error, Result};

pub(crate) fn could_be_pastebin_id(paste: &PasteId) -> bool {
    paste.user().is_none() && paste.id().len() == 8
}

#[tracing::instrument]
pub async fn get(id: &PasteId) -> Result<Option<StoredPaste>> {
    let mut response = net::Request::get(format!("https://pastebin.com/raw/{}", id.id()))
        .send()
        .await?;

    let content = match response.status_code() {
        200 => response.text().await?,
        404 => return Ok(None),
        code => return Err(Error::RemoteFailed(code, "pastebin.com get failed".into())),
    };

    Ok(Some(StoredPaste {
        content,
        entity_id: format!("pastebin-{id}"),
        last_modified: 0,
        metadata: None,
    }))
}
