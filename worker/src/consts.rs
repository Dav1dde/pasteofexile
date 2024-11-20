#![allow(dead_code)]

use std::time::Duration;

const ONE_KB: usize = 1024;

pub const MAX_UPLOAD_SIZE: usize = 75 * ONE_KB;
pub const MAX_UPLOAD_SIZE_LOGGED_IN: usize = 150 * ONE_KB;

pub const KV_STATIC_CONTENT: &str = "__STATIC_CONTENT";

pub const R2_STORAGE_BUCKET: &str = "STORAGE_BUCKET";

pub const ENV_SENTRY_PROJECT: &str = "SENTRY_PROJECT";
pub const ENV_SENTRY_TOKEN: &str = "SENTRY_TOKEN";
pub const ENV_SECRET_KEY: &str = "SECRET_KEY";

pub const ENV_OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
pub const ENV_OAUTH_CLIENT_SECRET: &str = "OAUTH_CLIENT_SECRET";

pub const ENV_STATS_URL: &str = "STATS_URL";
pub const ENV_STATS_TOKEN: &str = "STATS_TOKEN";

pub const OAUTH_SCOPE: &str = "account:profile";

pub const CACHE_A_BIT: Duration = Duration::from_secs(21600); // 6 Hours
pub const CACHE_FOREVER: Duration = Duration::from_secs(31536000);

pub const MAX_SESSION_DURATION: Duration = Duration::from_secs(1814400); // 3 Weeks

pub const CORS_POB_API: &[&str] = &["https://pob.cool"];
pub const CORS_READ_API: &[&str] = &["https://pobarchives.com"];
