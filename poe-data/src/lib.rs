pub mod gems {
    use shared::ClassSet;

    #[derive(Debug)]
    pub enum Color {
        Red,
        Green,
        Blue,
        White,
    }

    pub struct Gem {
        pub color: Color,
        pub rewards: &'static [Reward],
    }

    pub struct Reward {
        pub act: u8,
        pub npc: &'static str,
        pub quest: &'static str,
        pub classes: ClassSet,
    }

    mod data {
        include!(concat!(env!("OUT_DIR"), "/gems.rs"));
    }
}
