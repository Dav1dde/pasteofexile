pub fn ascendancy_image(ascendancy_or_class: &str) -> Option<&'static str> {
    macro_rules! assets {
        ($($name:ident),+) => {
            match ascendancy_or_class {
                $(stringify!($name) => Some(concat!("/assets/asc/", stringify!($name), ".png")),)+
                _ => None,
            }
        };
    }

    assets!(
        Ascendant,
        Assassin,
        Berserker,
        Champion,
        Chieftain,
        Deadeye,
        Elementalist,
        Gladiator,
        Guardian,
        Hierophant,
        Inquisitor,
        Juggernaut,
        Necromancer,
        Occultist,
        Pathfinder,
        Raider,
        Saboteur,
        Slayer,
        Trickster
    )
}

#[cfg(test)]
mod tests {
    use crate::assets::ascendancy_image;

    #[test]
    fn test_ascendancy_images() {
        assert_eq!(
            Some("/assets/asc/Ascendant.png"),
            ascendancy_image("Ascendant")
        );
        assert_eq!(
            Some("/assets/asc/Hierophant.png"),
            ascendancy_image("Hierophant")
        );
        assert_eq!(
            Some("/assets/asc/Trickster.png"),
            ascendancy_image("Trickster")
        );
        assert_eq!(None, ascendancy_image("Oops"));
    }
}
