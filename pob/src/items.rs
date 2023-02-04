#[derive(Debug, thiserror::Error)]
#[error("cannot parse item {0}")]
pub struct InvalidItem(&'static str);

#[derive(Clone, Copy, Debug)]
pub enum Rarity {
    Noraml,
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
}

impl<'a> Item<'a> {
    pub fn parse(item: &'a str) -> Result<Self, InvalidItem> {
        let mut lines = item.lines();

        let rarity = lines
            .next()
            .and_then(|s| s.strip_prefix("Rarity: "))
            .ok_or(InvalidItem("expected rarity"))?;
        let rarity = match rarity {
            "NORMAL" => Rarity::Noraml,
            "MAGIC" => Rarity::Magic,
            "RARE" => Rarity::Rare,
            "UNIQUE" => Rarity::Unique,
            _ => return Err(InvalidItem("expected normal, magic, rare or unique rarity")),
        };

        let name = match rarity {
            Rarity::Rare | Rarity::Unique => lines.next(),
            _ => None,
        };

        let mut base = lines.next().ok_or(InvalidItem("eof, expected base"))?;
        if !matches!(rarity, Rarity::Noraml | Rarity::Unique) {
            if let Some(flask) = extract_flask(base) {
                base = flask;
            }
        }

        Ok(Item { rarity, name, base })
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
Removes Curses on use
        "#,
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
20% reduced Mana Cost of Skills during Effect
        "#,
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
51% increased Evasion Rating during Flask effect
        "#,
        )
        .unwrap();

        assert_eq!(item.base, "Silver Flask");
    }
}
