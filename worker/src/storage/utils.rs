use shared::{model::PasteId, User};

use crate::Result;

pub(crate) fn to_path(id: &PasteId) -> Result<String> {
    match id {
        PasteId::Paste(id) => Ok(crate::utils::to_path(id)?),
        PasteId::UserPaste(up) => Ok(format!("user/{}/pastes/{}", up.user.normalized(), up.id)),
    }
}

pub(crate) fn to_prefix(user: &User) -> String {
    format!("user/{}/pastes/", user.normalized())
}

pub(crate) fn strip_prefix(file: &str, prefix: &str) -> Result<String> {
    file.strip_prefix(prefix).map(Into::into).ok_or_else(|| {
        crate::Error::Error(format!("expected file '{file}' to start with '{prefix}'"))
    })
}
