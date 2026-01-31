use crate::utils::Utils;

pub struct ActiveVec<T> {
    active_index: usize,
    vec: Vec<T>,
}

impl<T> ActiveVec<T> {
    const INITIAL_ACTIVE_INDEX: usize = 0;

    pub fn new(item: T) -> Self {
        let active_index = Self::INITIAL_ACTIVE_INDEX;
        let vec = std::vec![item];

        Self { active_index, vec }
    }

    #[must_use]
    pub const fn active_index(&self) -> usize {
        self.active_index
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.vec
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }

    #[must_use]
    pub fn active(&self) -> &T {
        &self.vec[self.active_index]
    }

    pub fn active_mut(&mut self) -> &mut T {
        &mut self.vec[self.active_index]
    }

    pub fn remove_active(&mut self) -> Option<T> {
        if self.vec.len() < 2 {
            return None;
        }

        let element = self.vec.remove(self.active_index);

        self.active_index = self.active_index.saturating_sub(1);

        element.some()
    }

    pub fn push(&mut self, item: T) -> (usize, &mut T) {
        let index = self.vec.len();
        let item = self.vec.mut_push(item);

        (index, item)
    }

    pub const fn set_active_index(&mut self, active_index: usize) {
        if active_index < self.vec.len() {
            self.active_index = active_index;
        }
    }

    pub fn cycle(&mut self, amount: isize) -> &mut Self {
        self.active_index.cycle_in_place(amount, self.vec.len());

        self
    }

    pub fn cycle_next(&mut self) -> &mut Self {
        self.cycle(1)
    }

    pub fn cycle_prev(&mut self) -> &mut Self {
        self.cycle(-1)
    }
}
