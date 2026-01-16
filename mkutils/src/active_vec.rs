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
    pub fn active(&self) -> Option<&T> {
        self.vec.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut T> {
        self.vec.get_mut(self.active_index)
    }

    pub fn remove_active(&mut self) -> Option<T> {
        if self.vec.len() < 2 {
            return None;
        }

        let element = self.vec.remove(self.active_index);

        self.active_index = self.active_index.saturating_sub(1);

        element.some()
    }

    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }

    pub fn cycle(&mut self, amount: isize) -> &mut Self {
        self.active_index.cycle_in_place(amount, self.vec.len());

        self
    }
}
