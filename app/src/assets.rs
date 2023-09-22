use shared::{Ascendancy, AscendancyOrClass, Class};

pub fn ascendancy_image(ascendancy_or_class: AscendancyOrClass) -> &'static str {
    macro_rules! assets {
        ($($name:pat => $file:ident),+) => {
            match ascendancy_or_class {
                $($name => concat!(
                    "https://assets.pobb.in/1/Art/2DArt/UIImages/Common/Icon", stringify!($file), ".webp"
                ),)+
            }
        };
    }

    assets!(
        // Class
        AscendancyOrClass::Class(Class::Duelist) => StrDex,
        AscendancyOrClass::Class(Class::Marauder) => Str,
        AscendancyOrClass::Class(Class::Ranger) => Dex,
        AscendancyOrClass::Class(Class::Scion) => StrDexInt,
        AscendancyOrClass::Class(Class::Shadow) => DexInt,
        AscendancyOrClass::Class(Class::Templar) => StrInt,
        AscendancyOrClass::Class(Class::Witch) => Int,
        // Ascendancy
        AscendancyOrClass::Ascendancy(Ascendancy::Ascendant) => StrDexInt_Ascendant,
        AscendancyOrClass::Ascendancy(Ascendancy::Assassin) => DexInt_Assassin,
        AscendancyOrClass::Ascendancy(Ascendancy::Berserker) => Str_Berserker,
        AscendancyOrClass::Ascendancy(Ascendancy::Champion) => StrDex_Champion,
        AscendancyOrClass::Ascendancy(Ascendancy::Chieftain) => Str_Chieftain,
        AscendancyOrClass::Ascendancy(Ascendancy::Deadeye) => Dex_Deadeye,
        AscendancyOrClass::Ascendancy(Ascendancy::Elementalist) => Int_Elementalist,
        AscendancyOrClass::Ascendancy(Ascendancy::Gladiator) => StrDex_Gladiator,
        AscendancyOrClass::Ascendancy(Ascendancy::Guardian) => StrInt_Guardian,
        AscendancyOrClass::Ascendancy(Ascendancy::Hierophant) => StrInt_Hierophant,
        AscendancyOrClass::Ascendancy(Ascendancy::Inquisitor) => StrInt_Inquisitor,
        AscendancyOrClass::Ascendancy(Ascendancy::Juggernaut) => Str_Juggernaut,
        AscendancyOrClass::Ascendancy(Ascendancy::Necromancer) => Int_Necromancer,
        AscendancyOrClass::Ascendancy(Ascendancy::Occultist) => Int_Occultist,
        AscendancyOrClass::Ascendancy(Ascendancy::Pathfinder) => Dex_Pathfinder,
        AscendancyOrClass::Ascendancy(Ascendancy::Raider) => Dex_Raider,
        AscendancyOrClass::Ascendancy(Ascendancy::Saboteur) => DexInt_Saboteur,
        AscendancyOrClass::Ascendancy(Ascendancy::Slayer) => StrDex_Slayer,
        AscendancyOrClass::Ascendancy(Ascendancy::Trickster) => DexInt_Trickster
    )
}

pub fn logo() -> &'static str {
    "/apple-touch-icon.png"
}

pub fn item_image_url(item_image_name: &str) -> String {
    let name =
        percent_encoding::utf8_percent_encode(item_image_name, percent_encoding::NON_ALPHANUMERIC);
    format!("https://assets.pobb.in/1/{name}.webp")
}

#[cfg(test)]
mod tests {
    use shared::{Ascendancy, Class};

    use crate::assets::ascendancy_image;

    #[test]
    fn test_ascendancy_images() {
        assert_eq!(
            "https://assets.pobb.in/1/Art/2DArt/UIImages/Common/IconStrDexInt_Ascendant.webp",
            ascendancy_image(Ascendancy::Ascendant.into())
        );
        assert_eq!(
            "https://assets.pobb.in/1/Art/2DArt/UIImages/Common/IconStrInt_Hierophant.webp",
            ascendancy_image(Ascendancy::Hierophant.into())
        );
        assert_eq!(
            "https://assets.pobb.in/1/Art/2DArt/UIImages/Common/IconDexInt.webp",
            ascendancy_image(Class::Shadow.into())
        );
    }
}
