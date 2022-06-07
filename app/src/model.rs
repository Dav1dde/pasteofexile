#[derive(Debug, Clone)]
pub struct PasteSummary {
    pub id: String,
    pub user: Option<String>,
    pub title: String,
    pub ascendancy: String,
    pub last_modified: u64,
}

impl PasteSummary {
    pub(crate) fn to_url(&self) -> String {
        if let Some(ref user) = self.user {
            format!("/u/{user}/{}", self.id)
        } else {
            format!("/{}", self.id)
        }
    }
}
