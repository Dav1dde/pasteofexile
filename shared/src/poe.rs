use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Class {
    Duelist,
    Marauder,
    Ranger,
    Scion,
    Shadow,
    Templar,
    Witch,
}

#[derive(Debug)]
pub struct InvalidClass;

impl std::fmt::Display for InvalidClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid class")
    }
}

impl std::error::Error for InvalidClass {}

impl FromStr for Class {
    type Err = InvalidClass;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Dex" => Self::Ranger,
            "DexInt" => Self::Shadow,
            "Int" => Self::Witch,
            "Str" => Self::Marauder,
            "StrDex" => Self::Duelist,
            "StrDexInt" => Self::Scion,
            "StrInt" => Self::Templar,

            "Duelist" => Self::Duelist,
            "Marauder" => Self::Marauder,
            "Ranger" => Self::Ranger,
            "Scion" => Self::Scion,
            "Shadow" => Self::Shadow,
            "Templar" => Self::Templar,
            "Witch" => Self::Witch,

            _ => return Err(InvalidClass),
        })
    }
}

impl std::ops::BitOr for Class {
    type Output = ClassSet;

    fn bitor(self, rhs: Self) -> Self::Output {
        ClassSet::new() | self | rhs
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClassSet(u8);

impl ClassSet {
    pub const fn new() -> Self {
        Self(0)
    }

    pub const fn from_u8(val: u8) -> Self {
        Self(val & 0b1111111)
    }

    pub fn as_u8(&self) -> u8 {
        self.0
    }

    pub fn contains(&self, other: Class) -> bool {
        (*self & other).0 > 0
    }
}

impl std::fmt::Debug for ClassSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ClassSet(")?;

        use Class::*;
        let mut first = true;
        for class in [Duelist, Marauder, Ranger, Scion, Shadow, Templar, Witch] {
            if self.contains(class) {
                if !first {
                    write!(f, " | ")?;
                }
                write!(f, "{class:?}")?;
                first = false;
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl std::ops::BitOr<Class> for ClassSet {
    type Output = ClassSet;

    fn bitor(self, rhs: Class) -> Self::Output {
        Self(self.0 | 1 << (rhs as u8))
    }
}

impl std::ops::BitAnd<Class> for ClassSet {
    type Output = ClassSet;

    fn bitand(self, rhs: Class) -> Self::Output {
        Self(self.0 & 1 << (rhs as u8))
    }
}

impl<const N: usize> From<[Class; N]> for ClassSet {
    fn from(value: [Class; N]) -> Self {
        let mut result = Self::new();
        for class in value {
            result = result | class;
        }
        result
    }
}

impl FromIterator<Class> for ClassSet {
    fn from_iter<T: IntoIterator<Item = Class>>(iter: T) -> Self {
        let mut result = Self::new();
        for class in iter {
            result = result | class;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_set() {
        assert_eq!(0b1000001, (Class::Duelist | Class::Witch).as_u8());
        assert!(ClassSet::from_u8(0b1000001).contains(Class::Duelist));
        assert!(ClassSet::from_u8(0b1000001).contains(Class::Witch));
        assert!(!ClassSet::from_u8(0b1000001).contains(Class::Ranger));
        assert_eq!(
            (Class::Duelist | Class::Witch),
            ClassSet::from([Class::Duelist, Class::Witch])
        );
        // Top most bit is unused, make sure it is discarded
        assert_eq!(ClassSet::from_u8(0b11000001), ClassSet::from_u8(0b01000001));
    }
}
