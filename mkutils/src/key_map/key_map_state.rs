use crate::{key_map::key_map::KeyMap, utils::Utils};
use crossterm::event::KeyEvent;
use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};
use trie_rs::inc_search::{Answer, IncSearch, Position};

pub type KeyMapIncSearch<'a, T> = IncSearch<'a, KeyEvent, Vec<T>>;

pub struct KeyMapState<T> {
    reset_time: Instant,
    reset_period: Duration,
    position: Position,
    phantom: PhantomData<T>,
}

impl<T: Clone> KeyMapState<T> {
    pub const DEFAULT_RESET_PERIOD: Duration = Duration::from_millis(250);

    #[must_use]
    pub fn new(key_map: &KeyMap<T>) -> Self {
        Self::new_with_reset_period(key_map, Self::DEFAULT_RESET_PERIOD)
    }

    #[must_use]
    pub fn new_with_reset_period(key_map: &KeyMap<T>, reset_period: Duration) -> Self {
        let reset_time = Instant::now();
        let position = Self::get_key_map_trie_root_position(key_map);
        let phantom = PhantomData;
        let mut key_map_state = Self {
            reset_time,
            reset_period,
            position,
            phantom,
        };

        key_map_state.defer_reset_time();

        key_map_state
    }

    fn defer_reset_time(&mut self) {
        self.reset_time = Instant::now() + self.reset_period;
    }

    fn get_key_map_trie_root_position(key_map: &KeyMap<T>) -> Position {
        key_map.trie().inc_search().into()
    }

    fn resume_inc_search<'a>(&self, key_map: &'a KeyMap<T>) -> KeyMapIncSearch<'a, T> {
        IncSearch::resume(key_map.trie(), self.position)
    }

    fn set_position<P: Into<Position>>(&mut self, position: P) {
        self.position = position.into();
    }

    fn set_position_to_key_map_trie_root(&mut self, key_map: &KeyMap<T>) {
        let position = Self::get_key_map_trie_root_position(key_map);

        self.set_position(position);
    }

    fn reset(&mut self, key_map: &KeyMap<T>) {
        self.defer_reset_time();
        self.set_position_to_key_map_trie_root(key_map);
    }

    fn get_commands_from_inc_search_and_reset<'a>(
        &mut self,
        inc_search: &KeyMapIncSearch<'a, T>,
        key_map: &'a KeyMap<T>,
    ) -> &'a [T] {
        let commands = if let Some(commands) = inc_search.value() {
            commands.as_slice()
        } else {
            &[]
        };

        self.reset(key_map);

        commands
    }

    fn get_commands_from_key_map_and_reset<'a>(&mut self, key_map: &'a KeyMap<T>) -> &'a [T] {
        let inc_search = self.resume_inc_search(key_map);

        self.get_commands_from_inc_search_and_reset(&inc_search, key_map)
    }

    pub fn on_key_event<'a>(&mut self, key_map: &'a KeyMap<T>, key_event: KeyEvent) -> &'a [T] {
        self.defer_reset_time();

        let mut inc_search = self.resume_inc_search(key_map);

        match inc_search.query(&key_event) {
            Some(Answer::Match) => return self.get_commands_from_inc_search_and_reset(&inc_search, key_map),
            Some(Answer::PrefixAndMatch | Answer::Prefix) => self.set_position(inc_search),
            None => self.set_position_to_key_map_trie_root(key_map),
        }

        &[]
    }

    pub fn on_tick<'a>(&mut self, key_map: &'a KeyMap<T>) -> &'a [T] {
        if self.reset_time.has_happened() {
            self.get_commands_from_key_map_and_reset(key_map)
        } else {
            &[]
        }
    }

    pub fn on_key_map_update(&mut self, key_map: &KeyMap<T>) {
        self.reset(key_map);
    }
}
