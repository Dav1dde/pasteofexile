use shared::{model::PasteId, User};
use worker::{Method, Request};

use crate::assets;

#[derive(Debug, Clone)]
pub enum Route {
    App(app::Route),
    Api(Api),
    Asset,
    NotFound,
}

impl Route {
    pub fn new(req: &Request) -> Self {
        use sycamore_router::Route;

        let path = req.path();

        // API needs to match first (oembed.json might be an asset). API also has
        // the most specific routes (no `/<paste>` route)
        match req.method() {
            Method::Get => {
                let route = GetEndpoints::match_path(&path);
                if !matches!(route, GetEndpoints::NotFound) {
                    return Self::Api(Api::Get(route));
                }
            }
            Method::Post => {
                let route = PostEndpoints::match_path(&path);
                if !matches!(route, PostEndpoints::NotFound) {
                    return Self::Api(Api::Post(route));
                }
            }
            Method::Delete => {
                let route = DeleteEndpoints::match_path(&path);
                if !matches!(route, DeleteEndpoints::NotFound) {
                    return Self::Api(Api::Delete(route));
                }
            }
            _ => (),
        }

        if req.method() == Method::Get {
            // Assets need to match next, because the app routes contain routes which
            // would match on assets (e.g. app contains `/<paste>`)
            if assets::is_asset_path(&path) {
                return Self::Asset;
            }

            // App is a catch all
            let app = app::Route::match_path(&path);
            if !matches!(app, app::Route::NotFound) {
                return Self::App(app);
            }
        }

        Self::NotFound
    }
}

#[derive(Debug, Clone)]
pub enum Api {
    Get(GetEndpoints),
    Post(PostEndpoints),
    Delete(DeleteEndpoints),
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum GetEndpoints {
    #[to("/oembed.json")]
    Oembed,
    #[to("/api/internal/user/<user>")]
    User(User),
    #[to("/<id>/raw")]
    Paste(String),
    #[to("/u/<name>/<id>/raw")]
    UserPaste(User, String),
    #[to("/<id>/json")]
    PasteJson(String),
    #[to("/u/<name>/<id>/json")]
    UserPasteJson(User, String),
    /// Path of Building endpoint for importing builds.
    /// This supports the anonymous and user scoped paste IDs.
    /// User scoped paste IDs are used in `pob://` protocol links.
    /// Anonymous paste IDs are coming from importing an anonymous build URL in PoB.
    #[to("/pob/<id>")]
    PobPaste(PasteId),
    /// Path of Building endpoint for importing user paste URLs.
    #[to("/pob/u/<name>/<id>")]
    PobUserPaste(User, String),
    #[to("/login")]
    Login,
    #[to("/oauth2/authorization/poe")]
    Oauht2Poe,
    #[not_found]
    NotFound,
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum PostEndpoints {
    #[to("/api/internal/paste/")]
    Upload(),
    #[to("/pob/")]
    PobUpload(),
    #[not_found]
    NotFound,
}

#[derive(sycamore_router::Route, strum::IntoStaticStr, Debug, Clone)]
pub enum DeleteEndpoints {
    #[to("/api/internal/paste/<id>")]
    DeletePaste(PasteId),
    #[not_found]
    NotFound,
}
