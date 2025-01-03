use shared::{Ascendancy, AscendancyOrClass, Class, GameVersion};

pub fn ascendancy_image(ascendancy_or_class: AscendancyOrClass) -> &'static str {
    macro_rules! assets {
        ($($v:expr, $name:pat => $file:ident),+) => {
            match ascendancy_or_class {
                $($name => concat!(
                    "https://assets.pobb.in/", $v, "/Art/2DArt/UIImages/Common/Icon", stringify!($file), ".webp"
                ),)+
            }
        };
    }

    // TODO: this needs to be version dependent, because of the overlap in classes/ascendancies
    assets!(
        // Class
        1, AscendancyOrClass::Class(Class::Duelist) => StrDex,
        1, AscendancyOrClass::Class(Class::Marauder) => Str,
        1, AscendancyOrClass::Class(Class::Ranger) => Dex,
        1, AscendancyOrClass::Class(Class::Scion) => StrDexInt,
        1, AscendancyOrClass::Class(Class::Shadow) => DexInt,
        1, AscendancyOrClass::Class(Class::Templar) => StrInt,
        1, AscendancyOrClass::Class(Class::Witch) => Int,
        2, AscendancyOrClass::Class(Class::Warrior) => StrFourb,
        2, AscendancyOrClass::Class(Class::Mercenary) => StrDexFourb,
        2, AscendancyOrClass::Class(Class::Huntress) => DexFourb,
        2, AscendancyOrClass::Class(Class::Monk) => DexIntFourb,
        2, AscendancyOrClass::Class(Class::Sorceress) => IntFourb,
        2, AscendancyOrClass::Class(Class::Druid) => StrIntFourb,
        // Ascendancy
        1, AscendancyOrClass::Ascendancy(Ascendancy::Ascendant) => StrDexInt_Ascendant,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Assassin) => DexInt_Assassin,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Berserker) => Str_Berserker,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Champion) => StrDex_Champion,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Chieftain) => Str_Chieftain,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Deadeye) => Dex_Deadeye,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Elementalist) => Int_Elementalist,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Gladiator) => StrDex_Gladiator,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Guardian) => StrInt_Guardian,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Hierophant) => StrInt_Hierophant,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Inquisitor) => StrInt_Inquisitor,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Juggernaut) => Str_Juggernaut,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Necromancer) => Int_Necromancer,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Occultist) => Int_Occultist,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Pathfinder) => Dex_Pathfinder,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Raider) => Dex_Raider,
        2, AscendancyOrClass::Ascendancy(Ascendancy::BloodMage) => IntFour_Witch2,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Infernalist) => IntFour_Witch1,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Titan) => StrFourb_Warrior1,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Warbringer) => StrFourb_Warrior2,
        2, AscendancyOrClass::Ascendancy(Ascendancy::WitchHunter) => StrDexFourb_Mercenary2,
        2, AscendancyOrClass::Ascendancy(Ascendancy::GemlingLegionnaire) => StrDexFourb_Mercenary3,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Invoker) => DexIntFourb_Monk2,
        2, AscendancyOrClass::Ascendancy(Ascendancy::AcolyteOfChayula) => DexIntFourb_Monk3,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Stormweaver) => IntFourb_Sorceress1,
        2, AscendancyOrClass::Ascendancy(Ascendancy::Chronomancer) => IntFourb_Sorceress2,

        // No new asset for Warden yet.
        1, AscendancyOrClass::Ascendancy(Ascendancy::Warden) => Dex_Raider,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Saboteur) => DexInt_Saboteur,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Slayer) => StrDex_Slayer,
        1, AscendancyOrClass::Ascendancy(Ascendancy::Trickster) => DexInt_Trickster
    )
}

pub fn logo() -> &'static str {
    "/apple-touch-icon.png"
}

pub fn item_image_url(v: GameVersion, item_image_name: &str) -> String {
    let name =
        percent_encoding::utf8_percent_encode(item_image_name, percent_encoding::NON_ALPHANUMERIC);
    match v {
        GameVersion::One => format!("https://assets.pobb.in/1/{name}.webp"),
        GameVersion::Two => format!("https://assets.pobb.in/2/{name}.webp"),
    }
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
