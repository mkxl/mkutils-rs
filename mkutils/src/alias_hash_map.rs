use crate::utils::Utils;
use derive_more::From;
use either::Either;
use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Default, From)]
pub struct AliasHashMap<K, V> {
    hash_map: HashMap<K, Either<K, V>>,
}

impl<K: Eq + Hash, V> AliasHashMap<K, V> {
    #[must_use]
    pub fn new() -> Self {
        HashMap::new().into()
    }

    pub fn get<Q: Eq + Hash + ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        let mut seen_keys = HashSet::new();
        let (mut key, mut either) = self.hash_map.get_key_value(key)?;

        loop {
            // NOTE: if key was not inserted, then it's been seen before, and we've hit a cycle
            if !seen_keys.insert(key) {
                return None;
            }

            match either {
                Either::Left(new_key) => {
                    (key, either) = self.hash_map.get_key_value::<K>(new_key)?;
                }
                Either::Right(value) => return value.some(),
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.hash_map.insert(key, value.into_right());
    }

    pub fn insert_alias(&mut self, from_key: K, to_key: K) {
        self.hash_map.insert(from_key, to_key.into_left());
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.hash_map
            .iter()
            .map(Utils::into_second)
            .filter_map(Either::get_right)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.hash_map
            .iter_mut()
            .map(Utils::into_second)
            .filter_map(Either::get_right_mut)
    }
}
