use serde::{Deserialize, Serialize};
use sycamore::{
    context::{ContextProvider, ContextProviderProps},
    prelude::*,
};

use crate::utils::if_browser;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub name: String,
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
}

#[derive(Clone)]
pub struct SessionValue(Signal<Session>);

impl SessionValue {
    pub fn get(&self) -> std::rc::Rc<Session> {
        self.0.get()
    }

    pub fn logout(&self) {
        if_browser!(self.0.set(Session::logout()));
    }
}

impl Session {
    #[cfg(not(feature = "ssr"))]
    fn from_document() -> anyhow::Result<Self> {
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

        let (session, _) = match session.split_once('.') {
            Some((session, sig)) => (session, sig),
            None => anyhow::bail!("invalid format, missing signature"),
        };

        let session = base64::decode_config(session, base64::URL_SAFE_NO_PAD)?;
        let user = serde_json::from_slice(&session)?;

        Ok(Session::LoggedIn(user))
    }

    #[cfg(not(feature = "ssr"))]
    fn logout() -> Self {
        let _ = crate::utils::document::<web_sys::HtmlDocument>()
            .set_cookie("session=; max-age=0; path=/");
        Self::None
    }
}

#[component(SessionWrapper<G>)]
pub fn session_wrapper<F>(children: F) -> View<G>
where
    F: FnOnce() -> View<G>,
{
    let value = if_browser!(
        {
            let session = match Session::from_document() {
                Ok(session) => session,
                Err(err) => {
                    log::error!("Can not extract session: {:?}", err);
                    Session::logout()
                }
            };

            let signal = Signal::new(Session::None);

            // Ugly workaround to let hydration finish before 'logging' the user in.
            // This prevents hydration from breaking because the markup does not match the markup
            // rendered on the server side.
            crate::utils::spawn_local!(signal, {
                signal.set(session);
            });

            SessionValue(signal)
        },
        { SessionValue(Signal::new(Session::None)) }
    );

    view! { ContextProvider(ContextProviderProps{ value, children }) }
}
