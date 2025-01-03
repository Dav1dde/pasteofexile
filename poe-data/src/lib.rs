pub mod gems {
    use shared::{Class, ClassSet, Color};

    pub struct Gem {
        pub name: &'static str,
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

    pub fn by_id_poe1(id: &str) -> Option<&'static Gem> {
        data_poe1::GEMS.get(id)
    }
    mod data_poe1 {
        include!(concat!(env!("OUT_DIR"), "/gems.rs"));
    }

    pub fn by_id_poe2(id: &str) -> Option<&'static Gem> {
        data_poe2::GEMS.get(id)
    }
    mod data_poe2 {
        include!(concat!(env!("OUT_DIR"), "/gems2.rs"));
    }
}
