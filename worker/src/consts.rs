#![allow(dead_code)]

const ONE_KB: usize = 1024;

pub const MAX_UPLOAD_SIZE: usize = 50 * ONE_KB;

pub const KV_STATIC_CONTENT: &str = "__STATIC_CONTENT";
pub const KV_B2_CREDENTIALS: &str = "B2_CREDENTIALS";
pub const KV_PASTE_STORAGE: &str = "PASTE_STORAGE";

pub const ENV_B2_KEY_ID: &str = "B2_KEY_ID";
pub const ENV_B2_APPLICATION_KEY: &str = "B2_APPLICATION_KEY";
pub const ENV_B2_PUBLIC_FILE_URL: &str = "B2_PUBLIC_FILE_URL";
pub const ENV_SENTRY_PROJECT: &str = "SENTRY_PROJECT";
pub const ENV_SENTRY_TOKEN: &str = "SENTRY_TOKEN";

const HOUR: u32 = 3_600;
const DAY: u32 = 24 * HOUR;

pub const CACHE_ASSETS: u32 = 2 * DAY;
