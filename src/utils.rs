use std::collections::BTreeMap;

pub(crate) fn btree_maps_are_equal<K: Ord + Eq, V: Eq>(
    map1: &BTreeMap<K, V>,
    map2: &BTreeMap<K, V>,
) -> bool {
    if map1.len() != map2.len() {
        return false;
    }

    for (key, value) in map1 {
        if map2.get(key) != Some(value) {
            return false;
        }
    }

    true
}
