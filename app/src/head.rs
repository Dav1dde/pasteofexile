use crate::{Context, Route};
use pob::PathOfBuilding;
use sycamore::prelude::*;

static TITLE: &str = "Paste of Exile";
static DESCRIPTION: &str =
    "Paste of Exile is a website to share your Path of Building builds online";
static DEFAULT_COLOR: &str = "#0ea5e9";

struct Meta {
    title: String,
    description: String,
    image: String,
    color: &'static str,
}

#[component(Head<G>)]
pub fn head(ctx: Context) -> View<G> {
    let meta = get_meta(&ctx);

    let title = meta.title.clone();
    let oembed = format!("https://{}/oembed.json", ctx.host());
    view! {
        title { (title) }
        meta(property="og:title", content=meta.title)
        meta(property="og:description", content=meta.description)
        meta(property="og:image", content=meta.image)
        meta(name="theme-color", content=meta.color)
        link(type="application/json+oembed", href=oembed)
    }
}

fn get_meta(ctx: &Context) -> Meta {
    match ctx.route().unwrap() {
        Route::NotFound => Meta {
            title: format!("{} - Not Found", TITLE),
            description: DESCRIPTION.to_owned(),
            image: "".to_owned(),
            color: DEFAULT_COLOR,
        },
        Route::Index => Meta {
            title: format!("{} - Share your Path of Exile build", TITLE),
            description: DESCRIPTION.to_owned(),
            image: "".to_owned(),
            color: DEFAULT_COLOR,
        },
        Route::Paste(_) => {
            let pob = ctx.get_paste().unwrap().path_of_building().unwrap();
            let title = crate::pob::title(&*pob);
            let description = "3000 Life, 500 ES, 900 Mana\n1003 DPS\nConfig: Sirus".to_owned();
            let image = format!("/assets/asc/{}.png", pob.ascendancy_name());
            let color = get_color(pob.ascendancy_name());
            Meta {
                title,
                description,
                image,
                color,
            }
        }
    }
}

fn get_color(ascendancy_name: &str) -> &'static str {
    match ascendancy_name {
        "Slayer" => "#96afc8",
        "Gladiator" => "#96afc8",
        "Champion" => "#96afc8",
        "Juggernaut" => "#af5a32",
        "Berserker" => "#af5a32",
        "Chieftain" => "#af5a32",
        "Raider" => "#7cb376",
        "Deadeye" => "#7cb376",
        "Pathfinder" => "#7cb376",
        "Assassin" => "#72818d",
        "Trickster" => "#72818d",
        "Saboteur" => "#72818d",
        "Inquisitor" => "#cfbd8a",
        "Hierophant" => "#cfbd8a",
        "Guardian" => "#cfbd8a",
        "Occultist" => "#9ac3c9",
        "Elementalist" => "#9ac3c9",
        "Necromancer" => "#9ac3c9",
        "Ascendant" => "#cccccc",
        _ => DEFAULT_COLOR,
    }
}
