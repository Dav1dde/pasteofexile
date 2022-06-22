pub fn could_be_pastebin_id(id: &str) -> bool {
    id.len() == 8
}

pub async fn fetch_raw(id: &str) -> crate::Result<worker::Response> {
    let request = worker::Request::new(
        &format!("https://pastebin.com/raw/{id}"),
        worker::Method::Get,
    )?;
    Ok(worker::Fetch::Request(request).send().await?)
}
