use shared::{PasteId, User};

use crate::Result;

pub(crate) fn to_path_r2(id: &PasteId) -> Result<String> {
    match id {
        PasteId::Paste(id) => Ok(format!("pastes/{}", crate::utils::to_path(id))),
        PasteId::UserPaste(up) => Ok(format!("users/{}/pastes/{}", up.user.normalized(), up.id)),
    }
}

pub(crate) fn to_prefix_r2(user: &User) -> String {
    format!("users/{}/pastes/", user.normalized())
}

pub(crate) fn strip_prefix(file: &str, prefix: &str) -> Result<String> {
    file.strip_prefix(prefix).map(Into::into).ok_or_else(|| {
        crate::Error::Error(format!("expected file '{file}' to start with '{prefix}'"))
    })
}
