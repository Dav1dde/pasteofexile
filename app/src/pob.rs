use pob::PathOfBuilding;

#[inline]
pub fn title<T: PathOfBuilding>(pob: &T) -> String {
    format!("Level {} {}", pob.level(), pob.ascendancy_name())
}
