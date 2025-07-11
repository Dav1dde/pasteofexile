use std::iter::FusedIterator;

#[derive(Debug, thiserror::Error)]
#[error("cannot parse item {0}")]
pub struct InvalidItem(&'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Relic,
    Unique,
    Rare,
    Magic,
    Normal,
}

impl Rarity {
    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique | Self::Relic)
    }

    pub fn is_rare(&self) -> bool {
        matches!(self, Self::Rare)
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

impl Influence {
    fn parse(value: &str) -> Option<Self> {
        let influence = match value {
            "Shaper Item" => Influence::Shaper,
            "Elder Item" => Influence::Elder,

            "Crusader Item" => Influence::Crusader,
            "Hunter Item" => Influence::Hunter,
            "Redeemer Item" => Influence::Redeemer,
            "Warlord Item" => Influence::Warlord,

            "Searing Exarch Item" => Influence::SearingExarch,
            "Eater of Worlds Item" => Influence::EaterOfWorlds,

            value if value.starts_with("Synthesised") => Influence::Synthesis,

            _ => return None,
        };

        Some(influence)
    }
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
    pub alt_quality: Option<&'a str>,
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
        let mut lines = item.lines().peekable();

        let rarity = lines
            .next()
            .and_then(|s| s.strip_prefix("Rarity: "))
            .ok_or(InvalidItem("expected rarity"))?;
        let rarity = match rarity {
            "NORMAL" => Rarity::Normal,
            "MAGIC" => Rarity::Magic,
            "RARE" => Rarity::Rare,
            "UNIQUE" => Rarity::Unique,
            "RELIC" => Rarity::Relic,
            _ => return Err(InvalidItem("expected normal, magic, rare or unique rarity")),
        };

        let mut name = match rarity {
            Rarity::Rare | Rarity::Unique | Rarity::Relic => lines.next(),
            _ => None,
        };
        let mut base = lines.next().ok_or(InvalidItem("eof, expected base"))?;
        if matches!(rarity, Rarity::Normal | Rarity::Magic) {
            name = Some(base);
        }
        // Technically only necessary for Magic and Normal items.
        base = fixup_item_name(base);

        let mut item_level = 0;
        let mut level_requirement = 0;
        let mut quality = 0;
        let mut alt_quality = None;
        let mut armour = 0;
        let mut evasion = 0;
        let mut energy_shield = 0;

        let mut influence1 = None;
        let mut influence2 = None;

        let mut selected_variant = "";
        let mut implicits = "";

        loop {
            let Some(line) = lines.peek() else {
                break;
            };

            if let Some((cmd, arg)) = line.split_once(": ") {
                let _ = lines.next();

                macro_rules! p {
                    ($name:ident) => {
                        $name = arg.parse().unwrap_or($name)
                    };
                }

                match cmd {
                    "Item Level" => p!(item_level),
                    "LevelReq" => p!(level_requirement),
                    "Quality" => p!(quality),
                    "Catalyst" => alt_quality = Some(catalyst_to_alt_quality(arg)),
                    "CatalystQuality" => p!(quality),
                    "Armour" => p!(armour),
                    "Evasion" => p!(evasion),
                    "Energy Shield" => p!(energy_shield),
                    "Implicits" => {
                        let num = arg.parse().unwrap_or(0);
                        implicits = unsafe { get_n_lines(item, &mut lines, num) };
                    }
                    "Selected Variant" => selected_variant = arg,
                    _ => {
                        if let Some((a, q)) = parse_alt_quality(cmd, arg) {
                            alt_quality = Some(a);
                            quality = q;
                        }
                    }
                };
            } else if let Some(influence) = Influence::parse(line) {
                let _ = lines.next();

                if influence1.is_none() {
                    influence1 = Some(influence);
                } else if influence2.is_none() {
                    influence2 = Some(influence);
                }
            } else if line == &base {
                // Skip random base names which are not mods or commands,
                // bugged pob?
                let _ = lines.next();
            } else {
                break;
            }
        }

        let mut corrupted = false;
        let mut mirrored = false;
        let mut split = false;
        let mut rev_lines = item.lines().rev();
        let mods_end = loop {
            match rev_lines.next() {
                Some("Corrupted") => corrupted = true,
                Some("Mirrored") => mirrored = true,
                Some("Split") => split = true,
                m => break m,
            }
        };

        let is_mod = |line: &&str| {
            !line
                .split_whitespace()
                .next()
                .map(|f| f.ends_with(':'))
                .unwrap_or(false)
        };
        let first_explicit_mod = lines.find(is_mod);

        let explicits = first_explicit_mod
            .map(|start| unsafe { extract_slice_between(item, start, mods_end) })
            .unwrap_or("");

        // Postprocess magic items based on mod count ...
        if matches!(rarity, Rarity::Magic) {
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
            alt_quality,
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

    /// Attempts to make the item name PoE compatible.
    ///
    /// Strips common prefixes and suffixes from the name in the hopes
    /// of extracting an actualy valid item name.
    pub fn fixed_item_name(&self) -> Option<&'a str> {
        self.name.map(fixup_item_name)
    }

    pub fn enchants(&self) -> impl Iterator<Item = Mod<'a>> {
        ModLines::new(self.implicits)
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
            .take_while(|m| m.crafted)
    }

    pub fn implicits(&self) -> impl Iterator<Item = Mod<'a>> {
        ModLines::new(self.implicits)
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
            .skip_while(|m| m.crafted)
    }

    pub fn explicits(&self) -> impl Iterator<Item = Mod<'a>> {
        ModLines::new(self.explicits)
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
    pub tag: Option<&'a str>,

    variant: Option<&'a str>,
}

impl<'a> Mod<'a> {
    fn parse(mut mod_line: &'a str) -> Self {
        let mut fractured = false;
        let mut crafted = false;
        let mut variant = None;
        let mut tag = None;

        while let Some((attr, other)) = mod_line.trim_start_matches('{').split_once('}') {
            mod_line = other;

            let (attr, value) = attr.split_once(':').unwrap_or((attr, ""));
            match attr {
                "variant" => variant = Some(value),
                "fractured" => fractured = true,
                "crafted" => crafted = true,
                "tags" | "custom" | "range" => (),
                t => tag = Some(t),
            }
        }

        Mod {
            fractured,
            crafted,
            line: mod_line,
            tag,
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

/// Iterator which supports mods split over multiple lines.
struct ModLines<'a> {
    lines: &'a str,
}

impl<'a> ModLines<'a> {
    fn new(lines: &'a str) -> Self {
        Self { lines }
    }
}

impl<'a> Iterator for ModLines<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lines.is_empty() {
            return None;
        }

        let mut offset = 0;
        loop {
            let eom = match self.lines[offset..].find('\n') {
                Some(eom) => eom + offset,
                None => {
                    let next = self.lines;
                    self.lines = "";
                    break Some(next);
                }
            };
            let next = &self.lines[..eom];

            let is_multiline = next.ends_with("you've");

            if is_multiline {
                offset = eom + 1;
            } else {
                self.lines = &self.lines[eom + 1..];
                break Some(next);
            }
        }
    }
}

impl FusedIterator for ModLines<'_> {}

/// Reads the next `num` lines from the `lines` iterator and returns
/// a continuous str slice of the iterated range.
///
/// ## Safety:
///
/// Lines yielded from the `lines` iterator must be contained within `item`.
unsafe fn get_n_lines<'a>(
    item: &'a str,
    lines: &mut std::iter::Peekable<std::str::Lines<'a>>,
    num: usize,
) -> &'a str {
    if num == 0 {
        return "";
    }

    let Some(start) = lines.peek() else {
        return "";
    };

    unsafe { extract_slice_between(item, start, lines.nth(num - 1)) }
}

/// Extracts the string slice between `start` and `end`.
///
/// If `end` is `None` it returns the slice from `start` to the end of `s`.
///
///
/// ## Safety:
///
/// `start` and `end` must be slices taken from `s`.
unsafe fn extract_slice_between<'a>(s: &'a str, start: &'a str, end: Option<&'a str>) -> &'a str {
    let start = unsafe { start.as_ptr().offset_from(s.as_ptr()) } as usize;
    let end = end
        .map(|end| unsafe { end.as_ptr().offset_from(s.as_ptr()) as usize + end.len() })
        .unwrap_or(s.len());

    if start > end {
        ""
    } else {
        &s[start..end]
    }
}

/// Parses an alt quality string into the type of quality and it's value.
fn parse_alt_quality<'a>(cmd: &'a str, arg: &'a str) -> Option<(&'a str, u8)> {
    let s = cmd.strip_prefix("Quality (")?.strip_suffix(')')?;
    let value = arg.strip_prefix('+')?.strip_suffix('%')?.parse().ok()?;

    Some((s, value))
}

/// Maps a catalyst name to the alt quality name in game.
fn catalyst_to_alt_quality(s: &str) -> &str {
    match s {
        "Abrasive" => "Attack Modifiers",
        "Accelerating" => "Speed Modifiers",
        "Fertile" => "Life and Mana Modifiers",
        "Imbued" => "Caster Modifiers",
        "Intrinsic" => "Attribute Modifiers",
        "Noxious" => "Physical and Chaos Damage Modifiers",
        "Prismatic" => "Resistance Modifiers",
        "Tempering" => "Defense Modifiers",
        "Turbulent" => "Elemental Modifiers",
        "Unstable" => "Critical Modifiers",
        s => s,
    }
}

/// 'Fixes' a PoE item name, this can be an actual name (Unique)
/// or a base item.
///
/// Strips prefixes used by creators (`Endgame - Mageblood`)
/// and suffixes added by PoB.
fn fixup_item_name(mut name: &str) -> &str {
    // Some creators prefix their items with `{Prefix} - `,
    // strip the prefix. Support `Foo - Bar - ` prefixes.
    // Items: like `Pig-Faced Bascinet` exist.
    name = name.rsplit_once("- ").map(|(_, name)| name).unwrap_or(name);
    // PoB generates Legion Jewels with `[Seed]` at the end:
    // Brutal Restraint [...] -> Brutal Restraint
    //
    // Some content creators use `(` now to add comments/annotations to the item.
    let end = name.find(['[', '(']).unwrap_or(name.len());
    name[..end].trim()
}

fn extract_magic_base(base: &str, num_mods: usize) -> &str {
    if num_mods == 0 {
        return base;
    }

    // Strip common prefixes.
    let base = base.trim_start_matches("Synthesised ");

    let end = base.find(" of").unwrap_or(base.len());
    let has_suffix_in_name = end != base.len();
    // strip off the suffix if there is one
    let base = &base[..end];

    #[allow(clippy::if_same_then_else)]
    if has_suffix_in_name && num_mods == 1 {
        // Only suffix
        base
    } else if may_be_full_base(base) {
        // Technically incorrect item names,
        // "Jade Flask" but has a mod.
        // Happens if you add a crafted mod to a flask on pob.
        // But also includes items that have 2 mod lines but
        // it's just a multiline suffix.
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
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 2);
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
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 2);
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
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 2);
    }

    #[test]
    fn magic_utility_flask_no_mods_enchant() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Jade Flask
Crafted: true
Prefix: None
Suffix: None
CatalystQuality: 20
Quality: 20
LevelReq: 27
Implicits: 0
{tags:flask,resource,unveiled_mod,mana}{crafted}{range:1}(20-25)% reduced Mana Cost of Skills during Effect"#,
        )
        .unwrap();

        assert_eq!(item.name, Some("Jade Flask"));
        assert_eq!(item.base, "Jade Flask");
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 1);
    }

    #[test]
    fn magic_synthesised_jewel() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Synthesised Flaring Ghastly Eye Jewel of Shelter
Item Level: 85
LevelReq: 66
Implicits: 1
You cannot be Hindered
+11% to Cold and Lightning Resistances
Minions deal 26 to 33 additional Physical Damage
Corrupted
"#,
        )
        .unwrap();

        assert_eq!(
            item.name,
            Some("Synthesised Flaring Ghastly Eye Jewel of Shelter")
        );
        assert_eq!(item.base, "Ghastly Eye Jewel");
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 1);
        assert_eq!(item.explicits().count(), 2);
    }

    #[test]
    fn unique_carcass_jack() {
        let item = Item::parse(
            r#"Rarity: UNIQUE
Endgame - Carcass-Jack [123]
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

        dbg!(&item);

        assert_eq!(item.item_level, 0);
        assert_eq!(item.name, Some("Endgame - Carcass-Jack [123]"));
        assert_eq!(item.fixed_item_name(), Some("Carcass-Jack"));
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 6);
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
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 2);
        assert_eq!(item.explicits().count(), 7);
    }

    #[test]
    fn rare_alt_quality_ring() {
        let item = Item::parse(
            r#"Rarity: RARE
Sorrow Hold
Amethyst Ring
Item Level: 85
LevelReq: 65
Implicits: 1
+27% to Chaos Resistance
Quality (Resistance Modifiers): +17%
{fractured}+18% to all Elemental Resistances
+45 to Intelligence
Adds 1 to 2 Physical Damage to Attacks
+58 to maximum Life
Regenerate 5.3 Mana per second
+32% to Chaos Resistance
{crafted}Non-Channelling Skills have -7 to Total Mana Cost"#,
        )
        .unwrap();

        assert_eq!(item.item_level, 85);
        assert_eq!(item.level_requirement, 65);
        assert_eq!(item.quality, 17);
        assert_eq!(item.alt_quality, Some("Resistance Modifiers"));
        assert_eq!(item.influence1, Some(Influence::Fracture));
        assert_eq!(item.influence2, Some(Influence::Fracture));
    }

    #[test]
    fn rare_alt_quality_amulet() {
        let item = Item::parse(
            r#"Rarity: RARE
Amulet
Agate Amulet
Crafted: true
Catalyst: Accelerating
CatalystQuality: 13
LevelReq: 35
Implicits: 1
{tags:attribute}{range:0.5}+(16-24) to Strength and Intelligence
+30 to Strength
+30 to Intelligence"#,
        )
        .unwrap();

        assert_eq!(item.quality, 13);
        assert_eq!(item.alt_quality, Some("Speed Modifiers"));
        assert_eq!(item.influence1, None);
        assert_eq!(item.influence2, None);
    }

    #[test]
    fn unique_tabula_no_explicits() {
        let item = Item::parse(
            r#"Rarity: UNIQUE
Tabula Rasa
Simple Robe
Quality: 20
Sockets: W-W-W-W-W-W
Implicits: 1
{tags:gem}+1 to Level of Socketed Gems
Corrupted"#,
        )
        .unwrap();

        assert_eq!(item.name, Some("Tabula Rasa"));
        assert_eq!(item.base, "Simple Robe");
        assert_eq!(item.enchants().count(), 0);
        assert_eq!(item.implicits().count(), 1);
        assert_eq!(item.explicits().count(), 0);
        assert!(item.corrupted);
    }

    #[test]
    fn multiline_enchant() {
        let item = Item::parse(
            r#"Rarity: UNIQUE
March of the Legion
Legion Boots
Armour: 496
ArmourBasePercentile: 0.9341
Energy Shield: 104
EnergyShieldBasePercentile: 0.985
Unique ID: a497050cd8fd2f5ba43b9ab0cc9d721335a642ae35f77cd62e809b9eb912b8d4
Item Level: 82
Quality: 20
Sockets: R-B-B-R
LevelReq: 58
Implicits: 2
{crafted}+8% chance to Suppress Spell Damage if you've
taken Spell Damage Recently
+3 to Level of Socketed Aura Gems
Socketed Gems are Supported by Level 25 Divine Blessing
297% increased Armour and Energy Shield
+17% to all Elemental Resistances
24% increased Movement Speed"#,
        )
        .unwrap();

        assert_eq!(item.enchants().count(), 1);
        assert_eq!(item.implicits().count(), 0);
        assert_eq!(item.explicits().count(), 5);
    }

    #[test]
    fn double_mod_only_suffix() {
        let item = Item::parse(
            r#"Rarity: MAGIC
Endgame Flask - Sapphire Flask of the Lizard
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

        assert_eq!(
            Some("Endgame Flask - Sapphire Flask of the Lizard"),
            item.name
        );
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

    #[test]
    pub fn relic_rarity() {
        let item = Item::parse(
            r#"Rarity: RELIC
Farrul's Fur
Triumphant Lamellar
Implicits: 0
+93 to maximum Life"#,
        )
        .unwrap();

        assert_eq!(item.rarity, Rarity::Relic);
        assert_eq!(item.name, Some("Farrul's Fur"));
        assert_eq!(item.base, "Triumphant Lamellar");
    }

    #[test]
    fn mod_tag() {
        let item = Item::parse(
            r#"Rarity: RELIC
Farrul's Fur
Triumphant Lamellar
Implicits: 0
+93 to maximum Life
{crucible}+35% to Chaos Resistance"#,
        )
        .unwrap();

        let mut explicits = item.explicits();
        let life = explicits.next().unwrap();
        let chaos_res = explicits.next().unwrap();
        assert_eq!(life.tag, None);
        assert_eq!(chaos_res.tag, Some("crucible"));
    }

    #[test]
    fn mod_lines() {
        let lines = ModLines::new("foo\nbar\nfirst you've\nsecond\nbaz").collect::<Vec<_>>();
        assert_eq!(lines, vec!["foo", "bar", "first you've\nsecond", "baz"]);

        let lines = ModLines::new("first you've\nsecond").collect::<Vec<_>>();
        assert_eq!(lines, vec!["first you've\nsecond"]);

        let lines = ModLines::new("first you've").collect::<Vec<_>>();
        assert_eq!(lines, vec!["first you've"]);
    }
}
