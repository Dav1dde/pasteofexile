pub mod gems {
    use shared::{Class, ClassSet, Color};

    pub struct Gem {
        pub color: Color,
        pub level: u8,
        pub vendors: &'static [Vendor],
    }

    impl Gem {
        pub fn vendors(&self, class: Class) -> impl Iterator<Item = &'static Vendor> + '_ {
            self.vendors
                .iter()
                .filter(move |vendor| vendor.classes.contains(class))
        }
    }

    pub struct Vendor {
        pub act: u8,
        pub npc: &'static str,
        pub quest: &'static str,
        pub classes: ClassSet,
    }

    pub fn by_id(id: &str) -> Option<&'static Gem> {
        data::GEMS.get(id)
    }

    mod data {
        include!(concat!(env!("OUT_DIR"), "/gems.rs"));
    }
}
