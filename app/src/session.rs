use serde::{Deserialize, Serialize};
use sycamore::{prelude::*, reactive::provide_context};

use crate::consts::MAX_SESSION_DURATION;

// TODO move this into shared
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub name: shared::User,
}

pub enum Session {
    None,
    #[allow(dead_code)] // TODO: I don't like this
    LoggedIn(User),
}

impl Session {
    pub fn is_logged_in(&self) -> bool {
        matches!(self, Self::LoggedIn(_))
    }

    pub fn user(&self) -> Option<&User> {
        match self {
            Self::LoggedIn(user) => Some(user),
            _ => None,
        }
    }

    pub fn logout() -> Self {
        let _ = crate::utils::document::<web_sys::HtmlDocument>()
            .set_cookie("session=; max-age=0; path=/");
        Self::None
    }

    fn from_document() -> Result<Self, Box<dyn std::error::Error>> {
        let session = crate::utils::document::<web_sys::HtmlDocument>()
            .cookie()
            .unwrap()
            .split(';')
            .filter_map(|part| part.split_once('='))
            .find(|(k, _)| k.trim() == "session")
            .map(|(_, v)| v.trim().to_owned());

        let session = match session {
            Some(session) => session,
            None => return Ok(Session::None),
        };

        let (session, _) = match session.rsplit_once('.') {
            Some((session, sig)) => (session, sig),
            None => return Err("invalid format, missing signature".into()),
        };

        let (session, ts) = match session.rsplit_once('.') {
            Some((session, ts)) => (session, ts),
            None => return Err("invalid format, missing timestamp".into()),
        };

        let ts = u64::from_le_bytes(
            base64::decode_config(ts, base64::URL_SAFE_NO_PAD)?
                .try_into()
                .map_err(|_| "Invalid timestamp")?,
        );
        let now = js_sys::Date::new_0().get_time() as u64;
        if ts > now || (now - ts) as u128 > MAX_SESSION_DURATION.as_millis() {
            return Err("Token Expired".into());
        }

        let user = serde_json::from_str(session)?;

        Ok(Session::LoggedIn(user))
    }
}

pub type SessionValue = RcSignal<Session>;

pub fn provide_session<G: Html>(cx: Scope) {
    let signal = create_rc_signal(Session::None);

    if G::IS_BROWSER {
        let session = match Session::from_document() {
            Ok(session) => session,
            Err(err) => {
                tracing::error!("Can not extract session: {:?}", err);
                Session::logout()
            }
        };

        // Ugly workaround to let hydration finish before 'logging' the user in.
        // This prevents hydration from breaking because the markup does not match the markup
        // rendered on the server side.
        let s = signal.clone();
        sycamore::futures::spawn_local(async move {
            s.set(session);
        });
    }

    provide_context(cx, signal);
}
