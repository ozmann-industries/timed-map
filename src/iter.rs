use super::*;

#[allow(clippy::enum_variant_names)]
pub(crate) enum GenericMapIter<'a, K, V> {
    BTreeMap(btree_map::Iter<'a, K, V>),
    #[cfg(feature = "std")]
    HashMap(hash_map::Iter<'a, K, V>),
    #[cfg(all(feature = "std", feature = "rustc-hash"))]
    FxHashMap(hash_map::Iter<'a, K, V>),
}

#[allow(clippy::enum_variant_names)]
pub(crate) enum GenericMapIterMut<'a, K, V> {
    BTreeMap(btree_map::IterMut<'a, K, V>),
    #[cfg(feature = "std")]
    HashMap(hash_map::IterMut<'a, K, V>),
    #[cfg(all(feature = "std", feature = "rustc-hash"))]
    FxHashMap(hash_map::IterMut<'a, K, V>),
}

#[allow(clippy::enum_variant_names)]
pub(crate) enum GenericMapIntoIter<K, V> {
    BTreeMap(btree_map::IntoIter<K, V>),
    #[cfg(feature = "std")]
    HashMap(hash_map::IntoIter<K, V>),
    #[cfg(all(feature = "std", feature = "rustc-hash"))]
    FxHashMap(hash_map::IntoIter<K, V>),
}

impl<'a, K, V> Iterator for GenericMapIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::BTreeMap(iter) => iter.next(),
            #[cfg(feature = "std")]
            Self::HashMap(iter) => iter.next(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(iter) => iter.next(),
        }
    }
}

impl<'a, K, V> Iterator for GenericMapIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::BTreeMap(iter) => iter.next(),
            #[cfg(feature = "std")]
            Self::HashMap(iter) => iter.next(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(iter) => iter.next(),
        }
    }
}

impl<K, V> Iterator for GenericMapIntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::BTreeMap(iter) => iter.next(),
            #[cfg(feature = "std")]
            Self::HashMap(iter) => iter.next(),
            #[cfg(all(feature = "std", feature = "rustc-hash"))]
            Self::FxHashMap(iter) => iter.next(),
        }
    }
}
