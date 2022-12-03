#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum Kind {
    Keystone,
    Node,
    Notable,
    Mastery,
}

impl Kind {
    pub fn is_keystone(&self) -> bool {
        matches!(self, Self::Keystone)
    }

    pub fn is_notable(&self) -> bool {
        matches!(self, Self::Notable)
    }

    pub fn is_mastery(&self) -> bool {
        matches!(self, Self::Mastery)
    }
}

#[derive(Debug)]
pub struct Node {
    pub kind: Kind,
    pub name: &'static str,
    pub stats: &'static [&'static str],
    pub mastery_effects: &'static [MasteryEffect],
}

#[derive(Debug)]
pub struct MasteryEffect {
    pub effect: u32,
    pub stats: &'static [&'static str],
}

#[derive(Debug)]
pub struct ParseVersionError;

impl std::fmt::Display for ParseVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse version")
    }
}

impl std::error::Error for ParseVersionError {}

macro_rules! gen {
    ($(($version:ident, $file:expr, $module:ident, $feature:expr, $m:pat)),+) => {
        #[derive(Copy, Clone)]
        pub enum Version {
            $(
                #[cfg(feature = $feature)]
                $version,
            )*
        }

        impl Version {
            fn get_node(&self, _id: u32) -> Option<&'static Node> {
                match self {
                    $(
                        #[cfg(feature = $feature)]
                        Self::$version => self::$module::TREE.get(&_id),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => None,
                }
            }
        }

        impl std::str::FromStr for Version {
            type Err = ParseVersionError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let r = match s {
                    $(
                        #[cfg(feature = $feature)]
                        $m => Self::$version,
                    )*
                    _ => return Err(ParseVersionError),
                };

                Ok(r)
            }
        }

        $(
            #[cfg(feature = $feature)]
            pub(crate) mod $module {
                include!(concat!(env!("OUT_DIR"), $file));
            }
        )*
    };
}

pub fn get_node_opt(version: &str, id: u32) -> Option<&'static Node> {
    version.parse::<Version>().ok().and_then(|v| v.get_node(id))
}

pub fn get_node(version: Version, id: u32) -> Option<&'static Node> {
    version.get_node(id)
}

gen! {
    (V3_15, "/tree3_15.rs", tree3_15, "tree-3_15", "3_15" | "3.15"),
    (V3_16, "/tree3_16.rs", tree3_16, "tree-3_16", "3_16" | "3.16"),
    (V3_17, "/tree3_17.rs", tree3_17, "tree-3_17", "3_17" | "3.17"),
    (V3_18, "/tree3_18.rs", tree3_18, "tree-3_18", "3_18" | "3.18"),
    (V3_19, "/tree3_19.rs", tree3_19, "tree-3_19", "3_19" | "3.19"),
    (V3_20, "/tree3_20.rs", tree3_20, "tree-3_20", "3_20" | "3.20")
}
