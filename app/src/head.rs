use crate::{pob, Context, Route};
use ::pob::{PathOfBuildingExt, SerdePathOfBuilding};
use sycamore::prelude::*;

const TITLE: &str = "POB B.in";
const DESCRIPTION: &str = "POB B.in is a website to share your Path of Building builds online";
const DEFAULT_COLOR: &str = "#0ea5e9";

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

            let config = pob::TitleConfig { no_title: true };
            let title = pob::title_with_config(&*pob, &config);

            let description = get_paste_summary(&pob).join("\n");

            let image = format!("/assets/asc/{}.png", pob.ascendancy_or_class_name());
            let color = get_color(pob.ascendancy_or_class_name());

            Meta {
                title,
                description,
                image,
                color,
            }
        }
    }
}

fn get_paste_summary(pob: &SerdePathOfBuilding) -> Vec<String> {
    let core_stats = pob::summary::core_stats(pob);
    let defense = pob::summary::defense(pob);
    let offense = pob::summary::offense(pob);
    let config = pob::summary::config(pob);

    vec![core_stats, defense, offense, config]
        .into_iter()
        .map(|line| {
            line.into_iter()
                .filter_map(|stat| stat.render_to_string())
                .collect::<Vec<_>>()
        })
        .map(|line| line.join("\u{318d}"))
        .map(|line| format!("\u{27A4} {}", line))
        .collect()
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
