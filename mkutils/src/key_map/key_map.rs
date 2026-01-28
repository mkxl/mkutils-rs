use crate::{key_map::key_binding::KeyBinding, utils::Utils};
use crossterm::event::KeyEvent;
use serde::{Deserialize, Deserializer};
use trie_rs::map::Trie;

pub type KeyBindingTrie<T> = Trie<KeyEvent, T>;

pub struct KeyMap<T> {
    bindings: Vec<KeyBinding<T>>,
    trie: KeyBindingTrie<T>,
}

impl<T: Clone> KeyMap<T> {
    #[must_use]
    pub fn new(bindings: Vec<KeyBinding<T>>) -> Self {
        let trie = Self::new_trie(bindings.clone());

        Self { bindings, trie }
    }

    fn new_trie(key_bindings: Vec<KeyBinding<T>>) -> KeyBindingTrie<T> {
        key_bindings.into_iter().map(KeyBinding::into).collect()
    }

    #[must_use]
    pub fn bindings(&self) -> &[KeyBinding<T>] {
        &self.bindings
    }

    pub fn set_bindings(&mut self, key_bindings: Vec<KeyBinding<T>>) {
        self.bindings.clone_from(&key_bindings);

        self.trie = Self::new_trie(key_bindings);
    }

    #[must_use]
    pub const fn trie(&self) -> &KeyBindingTrie<T> {
        &self.trie
    }
}

impl<T: Clone> From<Vec<KeyBinding<T>>> for KeyMap<T> {
    fn from(key_bindings: Vec<KeyBinding<T>>) -> Self {
        Self::new(key_bindings)
    }
}

impl<'a, T: Clone + Deserialize<'a>> Deserialize<'a> for KeyMap<T> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::deserialize(deserializer)?.convert::<Self>().ok()
    }
}
