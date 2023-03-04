#[derive(Debug, thiserror::Error)]
#[error("cannot parse item {0}")]
pub struct InvalidItem(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Unique,
    Rare,
    Magic,
    Normal,
}

impl Rarity {
    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Influence {
    Shaper,
    Elder,

    Crusader,
    Hunter,
    Redeemer,
    Warlord,

    SearingExarch,
    EaterOfWorlds,

    Synthesis,
    Fracture,
}

#[derive(Clone, Copy, Debug)]
pub struct Item<'a> {
    pub rarity: Rarity,
    pub name: Option<&'a str>,
    pub base: &'a str,
    // TODO: mabybe 0 should be options
    pub item_level: u8,
    pub level_requirement: u8,
    pub quality: u8,
    pub armour: u16,
    pub evasion: u16,
    pub energy_shield: u16,

    pub influence1: Option<Influence>,
    pub influence2: Option<Influence>,

    pub mirrored: bool,
    pub split: bool,
    pub corrupted: bool,

    selected_variant: &'a str,
    implicits: &'a str,
    explicits: &'a str,
}

impl<'a> Item<'a> {
    pub fn parse(item: &'a str) -> Result<Self, InvalidItem> {
        let mut lines = item.lines();

        let rarity = lines
            .next()
            .and_then(|s| s.strip_prefix("Rarity: "))
            .ok_or(InvalidItem("expected rarity"))?;
        let rarity = match rarity {
            "NORMAL" => Rarity::Normal,
            "MAGIC" => Rarity::Magic,
            "RARE" => Rarity::Rare,
            "UNIQUE" => Rarity::Unique,
            _ => return Err(InvalidItem("expected normal, magic, rare or unique rarity")),
        };

        let mut name = match rarity {
            Rarity::Rare | Rarity::Unique => lines.next(),
            _ => None,
        };
        let mut base = lines.next().ok_or(InvalidItem("eof, expected base"))?;
        if matches!(rarity, Rarity::Normal) {
            name = Some(base);
        }

        let mut item_level = 0;
        let mut level_requirement = 0;
        let mut quality = 0;
        let mut armour = 0;
        let mut evasion = 0;
        let mut energy_shield = 0;

        let mut influence1 = None;
        let mut influence2 = None;

        let mut selected_variant = "";
        let mut num_implicits = 0;
        for line in lines.by_ref() {
            if let Some((cmd, arg)) = line.split_once(": ") {
                macro_rules! parse {
                    ($(($pat:expr, $name:ident)),*) => {
                        match cmd {
                            $($pat => $name = arg.parse().unwrap_or($name)),*,
                            "Selected Variant" => selected_variant = arg,
                            _ => (),
                        }
                    };
                }

                parse! {
                    ("Item Level", item_level),
                    ("LevelReq", level_requirement),
                    ("Quality", quality),
                    ("Armour", armour),
                    ("Evasion", evasion),
                    ("Energy Shield", energy_shield),

                    ("Implicits", num_implicits)
                };

                // Section with mods starts
                if cmd == "Implicits" {
                    break;
                }
            } else {
                let influence = match line {
                    "Shaper Item" => Some(Influence::Shaper),
                    "Elder Item" => Some(Influence::Elder),

                    "Crusader Item" => Some(Influence::Crusader),
                    "Hunter Item" => Some(Influence::Hunter),
                    "Redeemer Item" => Some(Influence::Redeemer),
                    "Warlord Item" => Some(Influence::Warlord),

                    "Searing Exarch Item" => Some(Influence::SearingExarch),
                    "Eater of Worlds Item" => Some(Influence::EaterOfWorlds),

                    line if line.starts_with("Synthesised") => Some(Influence::Synthesis),

                    _ => None,
                };

                if let Some(influence) = influence {
                    if influence1.is_none() {
                        influence1 = Some(influence);
                    } else if influence2.is_none() {
                        influence2 = Some(influence);
                    }
                }
            }
        }

        let mut lines = lines.peekable();

        let mut implicits = &item[0..0];
        if num_implicits > 0 {
            if let Some(first_mod) = lines.peek() {
                // in case we have 0 implicits
                let first_mod_idx =
                    unsafe { first_mod.as_ptr().offset_from(item.as_ptr()) } as usize;
                for _ in 0..num_implicits - 1 {
                    lines.next();
                }
                let last_mod_idx = lines
                    .next()
                    .map(|m| unsafe { m.as_ptr().offset_from(item.as_ptr()) as usize + m.len() })
                    .unwrap_or(item.len());
                implicits = &item[first_mod_idx..last_mod_idx];
            }
        }

        let mut corrupted = false;
        let mut mirrored = false;
        let mut split = false;
        let mut rev_lines = item.lines().rev().peekable();
        let mut mods_end = None;
        loop {
            match rev_lines.peek() {
                Some(&"Corrupted") => corrupted = true,
                Some(&"Mirrored") => mirrored = true,
                Some(&"Split") => split = true,
                _ => break,
            }
            mods_end = rev_lines.next();
        }

        let is_mod = |line: &&str| {
            !line
                .split_whitespace()
                .next()
                .map(|f| f.ends_with(':'))
                .unwrap_or(false)
        };
        let first_explicit_mod = lines.find(is_mod);

        let mut explicits = &item[0..0];
        if let Some(first_mod) = first_explicit_mod {
            // in case we have 0 implicits
            let first_mod_idx = unsafe { first_mod.as_ptr().offset_from(item.as_ptr()) } as usize;
            let mods_end = mods_end
                .map(|m| unsafe { m.as_ptr().offset_from(item.as_ptr()) } as usize)
                .unwrap_or(item.len());
            explicits = &item[first_mod_idx..mods_end];
        }

        // Postprocess magic items based on mod count ...
        if matches!(rarity, Rarity::Magic) {
            name = Some(base);
            base = extract_magic_base(base, explicits.lines().count());
        }

        if influence1.is_none() && explicits.lines().any(|m| m.starts_with("{fractured}")) {
            influence1 = Some(Influence::Fracture);
        }
        // Copy the first influence to the second if there is not already an influence
        if influence2.is_none() {
            influence2 = influence1;
        }

        Ok(Item {
            rarity,
            name,
            base,
            item_level,
            level_requirement,
            quality,
            armour,
            evasion,
            energy_shield,
            influence1,
            influence2,
            corrupted,
            mirrored,
            split,
            selected_variant,
            implicits,
            explicits,
        })
    }

    pub fn enchants(&self) -> impl Iterator<Item = Mod<'a>> {
        self.implicits
            .lines()
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
            .take_while(|m| m.crafted)
    }

    pub fn implicits(&self) -> impl Iterator<Item = Mod<'a>> {
        self.implicits
            .lines()
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
            .skip_while(|m| m.crafted)
    }

    pub fn explicits(&self) -> impl Iterator<Item = Mod<'a>> {
        self.explicits
            .lines()
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
    }

    pub fn is_cluster_jewel(&self) -> bool {
        self.base.contains("Cluster Jewel")
    }
}

#[derive(Debug)]
pub struct Mod<'a> {
    pub fractured: bool,
    pub crafted: bool,
    pub line: &'a str,

    variant: Option<&'a str>,
}

impl<'a> Mod<'a> {
    fn parse(mut mod_line: &'a str) -> Self {
        let mut fractured = false;
        let mut crafted = false;
        let mut variant = None;

        while let Some((attr, other)) = mod_line.trim_start_matches('{').split_once('}') {
            mod_line = other;

            let (attr, value) = attr.split_once(':').unwrap_or((attr, ""));
            match attr {
                "variant" => variant = Some(value),
                "fractured" => fractured = true,
                "crafted" => crafted = true,
                _ => (),
            }
        }

        Mod {
            fractured,
            crafted,
            line: mod_line,
            variant,
        }
    }

    fn has_variant(&self, target: &str) -> bool {
        if target.is_empty() {
            return true;
        }
        let Some(variant) = self.variant else {
            return true;
        };
        variant.split(',').any(|variant| variant == target)
    }
}

fn extract_magic_base(base: &str, num_mods: usize) -> &str {
    if num_mods == 0 {
        return base;
    }

    let end = base.find(" of").unwrap_or(base.len());
    let has_suffix_in_name = end != base.len();
    // strip off the suffix if there is one
    let base = &base[..end];

    #[allow(clippy::if_same_then_else)]
    if has_suffix_in_name && num_mods == 1 {
        // Only suffix
        base
    } else if has_suffix_in_name && num_mods == 2 && may_be_full_base(base) {
        // 2 mods, but they both belong to the suffix and `base` is already
        // the full base name
        base
    } else {
        // Prefix with or without suffix
        base.split_once(' ').map_or(base, |s| s.1)
    }
}

/// Whether the name may be a full base without prefix/Suffix for magic items.
///
/// This is an inaccurate way of determining whether a name is a base or not.
///
/// Most bases are 2 words with the exception of shields, jewels and talismans.
/// Shields always are 3 words, abyss and cluster jewels are always 3 words,
/// talismans are mixed.
///
/// This ignores talismans and just assumes talismans are 2 words.
///
/// There are also some bases that are only one (singular) word, like `Ring`,
/// these bases can only obtained by 'bricking' a unique and then using a
/// vendor recipe to obtain a craftable base, these cases are also ignored.
///
/// Why this guessing? Accurately determining what the base of a magic item
/// is from a pob export would require a mod  or base db, which is quite costly
/// in terms of storage and code size. This is hopefully good enough.
fn may_be_full_base(name: &str) -> bool {
    let words = if name.ends_with("Shield")
        || name.ends_with("Cluster Jewel")
        || name.ends_with("Abyss Jewel")
    {
        3
    } else {
        2
    };

    // Check whether if the name has exactly `words` words
    name.split_whitespace().take(words + 1).count() == words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_life_flask() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Endless Grand Life Flask of Warding
Crafted: true
Prefix: {range:0.5}FlaskChargesAddedIncreasePercent3_
Suffix: {range:0.5}FlaskCurseImmunity1
Quality: 20
LevelReq: 34
Implicits: 0
28% increased Charge Recovery
Removes Curses on use"#,
        )
        .unwrap();

        assert_eq!(item.base, "Grand Life Flask");
    }

    #[test]
    fn magic_mana_flask() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Bountiful Eternal Mana Flask of the Mage
Crafted: true
Prefix: {range:0.5}FlaskExtraCharges3_
Suffix: {range:0.5}FlaskBuffReducedManaCostWhileHealing3
Quality: 20
LevelReq: 65
Implicits: 0
+26 to Maximum Charges
20% reduced Mana Cost of Skills during Effect"#,
        )
        .unwrap();

        assert_eq!(item.base, "Eternal Mana Flask");
    }

    #[test]
    fn magic_utility_flask() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Bountiful Silver Flask of the Ibex
Item Level: 64
Quality: 20
LevelReq: 46
Implicits: 0
+27 to Maximum Charges
51% increased Evasion Rating during Flask effect"#,
        )
        .unwrap();

        assert_eq!(item.base, "Silver Flask");
    }

    #[test]
    fn unique_carcas_jack() {
        let item = Item::parse(
            r#"Rarity: UNIQUE
Carcass Jack
Varnished Coat
Evasion: 1020
EvasionBasePercentile: 0.2766
Energy Shield: 209
EnergyShieldBasePercentile: 0.2961
Variant: Pre 3.0.0
Variant: Pre 3.5.0
Variant: Current
Selected Variant: 3
Varnished Coat
Quality: 20
Sockets: G-G-G-G-G-G
LevelReq: 62
Implicits: 0
{range:0.5}(120-150)% increased Evasion and Energy Shield
{range:0.5}+(50-70) to maximum Life
{range:0.5}+(9-12)% to all Elemental Resistances
{variant:1,2}20% increased Area of Effect
{variant:3}{range:0.5}(40-50)% increased Area of Effect
{variant:1}12% increased Area Damage
{variant:2,3}{range:0.5}(40-50)% increased Area Damage
Extra gore"#,
        )
        .unwrap();

        assert_eq!(item.item_level, 0);
    }

    #[test]
    fn rare_implicits_fractured_crafted() {
        let item = Item::parse(
            r#"Rarity: RARE
Gloom Vise
Dragonscale Gauntlets
Armour: 209
ArmourBasePercentile: 0.4451
Evasion: 208
EvasionBasePercentile: 0.4451
Unique ID: 77b08dd4ab3fff0da12b6b8d40d44a801ee5682717445454497de03514208f13
Item Level: 85
Quality: 20
Sockets: B-B-R-G
LevelReq: 67
Implicits: 2
Inflict Fire Exposure on Hit, applying -13% to Fire Resistance
Ignites you inflict spread to other Enemies within a Radius of 14
{fractured}+13% chance to Suppress Spell Damage
Adds 1 to 23 Lightning Damage to Attacks
42% increased Armour and Evasion
+85 to maximum Life
+35% to Chaos Resistance
16% increased Stun and Block Recovery
{crafted}40% reduced Effect of Chill and Shock on you"#,
        )
        .unwrap();

        assert_eq!(item.item_level, 85);
        assert_eq!(item.level_requirement, 67);
        assert_eq!(item.quality, 20);
        assert_eq!(item.armour, 209);
        assert_eq!(item.evasion, 208);
        assert_eq!(item.energy_shield, 0);
        assert_eq!(item.influence1, Some(Influence::Fracture));
        assert_eq!(item.influence2, Some(Influence::Fracture));
    }

    #[test]
    fn double_mod_only_suffix() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Sapphire Flask of the Lizard
Unique ID: 5ef536d249d5a5dfd3905be71ad31ee10b2bf04d4fd84c1c7917a306c80b3ec3
Item Level: 20
Quality: 20
LevelReq: 18
Implicits: 1
{crafted}Used when an adjacent Flask is used
46% less Duration
Immunity to Bleeding and Corrupted Blood during Effect"#,
        )
        .unwrap();

        assert_eq!("Sapphire Flask", item.base);
    }

    #[test]
    fn influences() {
        let item = Item::parse(
            r#"Rarity: RARE
Damnation Salvation
Devout Chainmail
Warlord Item
Crusader Item
Implicits: 0"#,
        )
        .unwrap();
        assert_eq!(item.influence1, Some(Influence::Warlord));
        assert_eq!(item.influence2, Some(Influence::Crusader));

        let item = Item::parse(
            r#"Rarity: UNIQUE
Valyrium
Moonstone Ring
Elder Item
Redeemer Item
Implicits: 0"#,
        )
        .unwrap();
        assert_eq!(item.influence1, Some(Influence::Elder));
        assert_eq!(item.influence2, Some(Influence::Redeemer));

        let item = Item::parse(
            r#"Rarity: RARE
Storm Rock
Mosaic Kite Shield
Shaper Item
Hunter Item
Implicits: 0"#,
        )
        .unwrap();
        assert_eq!(item.influence1, Some(Influence::Shaper));
        assert_eq!(item.influence2, Some(Influence::Hunter));

        let item = Item::parse(
            r#"Rarity: RARE
Armageddon Halo
Steel Circlet
Searing Exarch Item
Eater of Worlds Item
Implicits: 0"#,
        )
        .unwrap();
        assert_eq!(item.influence1, Some(Influence::SearingExarch));
        assert_eq!(item.influence2, Some(Influence::EaterOfWorlds));

        let item = Item::parse(
            r#"Rarity: RARE
Brimstone Torc
Blue Pearl Amulet
Synthesised Blue Pearl Amulet
Implicits: 0"#,
        )
        .unwrap();
        assert_eq!(item.influence1, Some(Influence::Synthesis));
        assert_eq!(item.influence2, Some(Influence::Synthesis));
    }
}
