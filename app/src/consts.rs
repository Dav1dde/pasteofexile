pub const IMG_ONERROR_EMPTY: &str =
    "this.src='data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw=='";
pub const IMG_ONERROR_HIDDEN: &str = "this.style.display='none'";
pub const IMG_ONERROR_INVISIBLE: &str = "this.style.visibility='hidden'";

pub const SELECT_ONCHANGE_COLOR_FROM_OPTION: &str =
    "this.style.color = getComputedStyle(this.options[this.selectedIndex]).color";

pub const LINK_WHITELIST: [&str; 13] = [
    "old.reddit.com",
    "pastebin.com",
    "pathofexile.com",
    "pobb.in",
    "poe.ninja",
    "poe.re",
    "poedb.tw",
    "poegems.quest",
    "poewiki.net",
    "reddit.com",
    "twitch.tv",
    "youtu.be",
    "youtube.com",
];

pub const POE_WIKI: &str = "https://www.poewiki.net/wiki/";

pub const SELF_URL: &str = "https://pobb.in";
