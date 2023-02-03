pub const IMG_ONERROR_EMPTY: &str =
    "this.src='data:image/gif;base64,R0lGODlhAQABAAAAACH5BAEKAAEALAAAAAABAAEAAAICTAEAOw=='";
pub const IMG_ONERROR_HIDDEN: &str = "this.style.display='none'";
pub const IMG_ONERROR_INVISIBLE: &str = "this.style.visibility='hidden'";

pub const SELECT_ONCHANGE_COLOR_FROM_OPTION: &str =
    "this.style.color = getComputedStyle(this.options[this.selectedIndex]).color";
