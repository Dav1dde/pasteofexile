#[derive(Debug, thiserror::Error)]
#[error("cannot parse item {0}")]
pub struct InvalidItem(&'static str);

#[derive(Clone, Copy, Debug)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
    Unique,
}

impl Rarity {
    pub fn is_unique(&self) -> bool {
        matches!(self, Self::Unique)
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
    pub armour: u16,
    pub evasion: u16,
    pub energy_shield: u16,

    selected_variant: &'a str,
    implicits: u8,
    mods: &'a str,
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
        if !matches!(rarity, Rarity::Normal | Rarity::Unique) {
            if let Some(flask) = extract_flask(base) {
                name = Some(base);
                base = flask;
            }
        }

        if matches!(rarity, Rarity::Normal) {
            name = Some(base);
        }

        let mut item_level = 0;
        let mut level_requirement = 0;
        let mut quality = 0;
        let mut armour = 0;
        let mut evasion = 0;
        let mut energy_shield = 0;

        let mut selected_variant = "";
        let mut implicits = 0;
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

                    ("Implicits", implicits)
                };

                // Section with mods starts
                if cmd == "Implicits" {
                    break;
                }
            }
        }

        let mut mods = &item[0..0];
        if let Some(first_mod) = lines.next() {
            let first_mod_idx = unsafe { first_mod.as_ptr().offset_from(item.as_ptr()) } as usize;
            mods = &item[first_mod_idx..];
        }

        // TODO: corrupted, split, mirrored and influenced items

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
            selected_variant,
            implicits,
            mods,
        })
    }

    pub fn implicits(&self) -> impl Iterator<Item = Mod<'a>> {
        self.mods
            .lines()
            .take(self.implicits as usize)
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
    }

    pub fn explicits(&self) -> impl Iterator<Item = Mod<'a>> {
        self.mods
            .lines()
            .skip(self.implicits as usize)
            .map(Mod::parse)
            .filter(|m| m.has_variant(self.selected_variant))
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

fn extract_flask(base: &str) -> Option<&str> {
    let mut iter = base
        .split_whitespace()
        .rev()
        .skip_while(|item| item != &"Flask");
    let flask = iter.next()?;
    let start = match iter.next() {
        Some("Life") | Some("Mana") => iter.next(),
        f => f,
    }?;

    let start_idx = unsafe { start.as_ptr().offset_from(base.as_ptr()) } as usize;
    let end_idx = unsafe { flask.as_ptr().offset_from(base.as_ptr()) + 5 } as usize;
    Some(&base[start_idx..end_idx])
}

#[cfg(test)]
mod tests {
    use crate::Item;

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
    }
}
