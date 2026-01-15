pub struct ActiveVec<T> {
    active_index: usize,
    vec: Vec<T>,
}

impl<T> ActiveVec<T> {
    const INITIAL_ACTIVE_INDEX: usize = 0;

    #[must_use]
    pub const fn new() -> Self {
        let active_index = Self::INITIAL_ACTIVE_INDEX;
        let vec = Vec::new();

        Self { active_index, vec }
    }

    pub fn as_slice_mut(&mut self) -> &mut [T] {
        &mut self.vec
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.vec.get_mut(index)
    }

    #[must_use]
    pub fn active(&self) -> Option<&T> {
        self.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut T> {
        self.get_mut(self.active_index)
    }

    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }
}

impl<T> Default for ActiveVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
