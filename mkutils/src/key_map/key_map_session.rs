use crate::{
    key_map::{key_map::KeyMap, key_map_state::KeyMapState},
    utils::Utils,
};
use crossterm::event::KeyEvent;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

pub struct KeyMapSession<T> {
    key_map: KeyMap<T>,
    key_map_state: KeyMapState<T>,
}

impl<T: Clone> KeyMapSession<T> {
    pub const DEFAULT_RESET_PERIOD: Duration = KeyMapState::<T>::DEFAULT_RESET_PERIOD;

    #[must_use]
    pub fn new(key_map: KeyMap<T>) -> Self {
        Self::new_with_reset_period(key_map, Self::DEFAULT_RESET_PERIOD)
    }

    #[must_use]
    pub fn new_with_reset_period(key_map: KeyMap<T>, reset_period: Duration) -> Self {
        let key_map_state = KeyMapState::new_with_reset_period(&key_map, reset_period);

        Self { key_map, key_map_state }
    }

    pub fn on_key_event(&mut self, key_event: KeyEvent) -> &[T] {
        self.key_map_state.on_key_event(&self.key_map, key_event)
    }

    pub fn on_tick(&mut self) -> &[T] {
        self.key_map_state.on_tick(&self.key_map)
    }
}

impl<T: Clone> From<KeyMap<T>> for KeyMapSession<T> {
    fn from(key_map: KeyMap<T>) -> Self {
        Self::new(key_map)
    }
}

impl<'a, T: Clone + Deserialize<'a>> Deserialize<'a> for KeyMapSession<T> {
    fn deserialize<D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        KeyMap::deserialize(deserializer)?.convert::<Self>().ok()
    }
}
