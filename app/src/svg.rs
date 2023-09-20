// Icons by Font-Awesome -> https://fontawesome.com/

// Width is hardcoded to 16px because everything else I tried did break the flexbox
// and we can't set the width on the svg element any other way at the moment.
pub static LOGOUT: &str = include_str!("svg/logout.svg");
pub static PEN: &str = include_str!("svg/pen.svg");
pub static TRASH: &str = include_str!("svg/trash.svg");
// Ugly hack with the concat!() but, good enough for now
pub static SPINNER: &str = concat!(include_str!("svg/spinner.svg"), "Creating&nbsp;...");
pub static BACK: &str = include_str!("svg/back.svg");
pub static HISTORY: &str = include_str!("svg/history.svg");
pub static CHEVRON_RIGHT: &str = include_str!("svg/chevron_right.svg");
pub static GITHUB: &str = include_str!("svg/github.svg");

pub static ICON_AMULET: &str = include_str!("svg/icon_amulet.svg");
pub static ICON_BELT: &str = include_str!("svg/icon_belt.svg");
pub static ICON_BODY_ARMOUR: &str = include_str!("svg/icon_body_armour.svg");
pub static ICON_BOOTS: &str = include_str!("svg/icon_boots.svg");
pub static ICON_BOW: &str = include_str!("svg/icon_bow.svg");
pub static ICON_GLOVES: &str = include_str!("svg/icon_gloves.svg");
pub static ICON_HELMET: &str = include_str!("svg/icon_helmet.svg");
pub static ICON_QUIVER: &str = include_str!("svg/icon_quiver.svg");
pub static ICON_RING: &str = include_str!("svg/icon_ring.svg");
pub static ICON_SHIELD: &str = include_str!("svg/icon_shield.svg");
pub static ICON_WEAPON: &str = include_str!("svg/icon_weapon.svg");
