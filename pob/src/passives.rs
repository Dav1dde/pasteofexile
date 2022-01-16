#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Keystone {
    ChaosInoculation,
    EldritchBattery,
    ElementalOverload,
    MindOverMatter,
}

impl Keystone {
    pub(crate) fn node(&self) -> u32 {
        match self {
            Self::ChaosInoculation => 11455,
            Self::EldritchBattery => 56075,
            Self::ElementalOverload => 22088,
            Self::MindOverMatter => 34098,
        }
    }
}
