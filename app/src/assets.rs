pub fn ascendancy_image(ascendancy_or_class: &str) -> Option<&'static str> {
    macro_rules! assets {
        ($($name:ident  => $file:ident),+) => {
            match ascendancy_or_class {
                $(stringify!($name) => Some(concat!(
                    "https://assets.pobb.in/1/Art/2DArt/UIImages/Common/Icon", stringify!($file), ".webp"
                )),)+
                _ => None,
            }
        };
    }

    assets!(
        // Class
        Duelist => StrDex,
        Marauder => Str,
        Ranger => Dex,
        Scion => StrDexInt,
        Shadow => DexInt,
        Templar => StrInt,
        Witch => Int,
        // Ascendancy
        Ascendant => StrDexInt_Ascendant,
        Assassin => DexInt_Assassin,
        Berserker => Str_Berserker,
        Champion => StrDex_Champion,
        Chieftain => Str_Chieftain,
        Deadeye => Dex_Deadeye,
        Elementalist => Int_Elementalist,
        Gladiator => StrDex_Gladiator,
        Guardian => StrInt_Guardian,
        Hierophant => StrInt_Hierophant,
        Inquisitor => StrInt_Inquisitor,
        Juggernaut => Str_Juggernaut,
        Necromancer => Int_Necromancer,
        Occultist => Int_Occultist,
        Pathfinder => Dex_Pathfinder,
        Raider => Dex_Raider,
        Saboteur => DexInt_Saboteur,
        Slayer => StrDex_Slayer,
        Trickster => DexInt_Trickster
    )
}

pub fn logo() -> &'static str {
    "/apple-touch-icon.png"
}

pub fn item_image_url(item_image_name: &str) -> Option<String> {
    let name =
        percent_encoding::utf8_percent_encode(item_image_name, percent_encoding::NON_ALPHANUMERIC);
    Some(format!("https://assets.pobb.in/1/{name}.webp"))
}

#[cfg(test)]
mod tests {
    use crate::assets::ascendancy_image;

    #[test]
    fn test_ascendancy_images() {
        assert_eq!(
            Some("https://assets.pobb.in/1/Art/2DArt/UIImages/Common/IconStrDexInt_Ascendant.webp"),
            ascendancy_image("Ascendant")
        );
        assert_eq!(
            Some("https://assets.pobb.in/1/Art/2DArt/UIImages/Common/IconStrInt_Hierophant.webp"),
            ascendancy_image("Hierophant")
        );
        assert_eq!(None, ascendancy_image("Oops"));
    }
}
