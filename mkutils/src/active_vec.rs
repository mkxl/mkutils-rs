/// A vector with a tracked "active" element.
///
/// `ActiveVec<T>` maintains a standard `Vec<T>` along with an index pointing to
/// the currently "active" element. This is useful for managing collections where
/// one element is selected or focused at a time.
///
/// # Examples
///
/// ```
/// use mkutils::ActiveVec;
///
/// let mut tabs = ActiveVec::new();
/// tabs.push("Home");
/// tabs.push("Settings");
/// tabs.push("About");
///
/// assert_eq!(tabs.active(), Some(&"Home")); // First element is active by default
/// ```
pub struct ActiveVec<T> {
    active_index: usize,
    vec: Vec<T>,
}

impl<T> ActiveVec<T> {
    const INITIAL_ACTIVE_INDEX: usize = 0;

    /// Creates a new empty `ActiveVec`.
    ///
    /// The active index starts at 0, but will return `None` until an element is pushed.
    #[must_use]
    pub const fn new() -> Self {
        let active_index = Self::INITIAL_ACTIVE_INDEX;
        let vec = Vec::new();

        Self { active_index, vec }
    }

    /// Returns a reference to the currently active element, if any.
    ///
    /// Returns `None` if the vector is empty or the active index is out of bounds.
    #[must_use]
    pub fn active(&self) -> Option<&T> {
        self.vec.get(self.active_index)
    }

    /// Returns a mutable reference to the currently active element, if any.
    ///
    /// Returns `None` if the vector is empty or the active index is out of bounds.
    pub fn active_mut(&mut self) -> Option<&mut T> {
        self.vec.get_mut(self.active_index)
    }

    /// Pushes an element onto the end of the vector.
    ///
    /// The active index remains unchanged, so the active element stays the same
    /// unless this is the first element being pushed.
    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }
}

impl<T> Default for ActiveVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
