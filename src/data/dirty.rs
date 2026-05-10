use std::ops::{Deref, DerefMut};

pub struct DirtyMarker<T> {
    pub inner: T,
    pub dirty: bool,
}

impl<T> From<T> for DirtyMarker<T> {
    fn from(inner: T) -> Self {
        Self {
            inner,
            dirty: false,
        }
    }
}

impl<T> Deref for DirtyMarker<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for DirtyMarker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty = true;
        &mut self.inner
    }
}

impl<T> DirtyMarker<T> {
    pub fn clean(inner: T) -> Self {
        Self {
            inner,
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn dirty(inner: T) -> Self {
        Self { inner, dirty: true }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_from_is_clean() {
        let d: DirtyMarker<i32> = DirtyMarker::from(42);
        assert!(!d.is_dirty());
    }

    #[test]
    fn clean_constructor_is_clean() {
        let d = DirtyMarker::clean(99u8);
        assert!(!d.is_dirty());
    }

    #[test]
    fn dirty_constructor_is_dirty() {
        let d = DirtyMarker::dirty("hello");
        assert!(d.is_dirty());
    }

    #[test]
    fn deref_read_does_not_mark_dirty() {
        let d = DirtyMarker::clean(5i32);
        let _ = *d; // immutable deref
        assert!(!d.is_dirty());
    }

    #[test]
    fn deref_mut_marks_dirty() {
        let mut d = DirtyMarker::clean(5i32);
        *d = 10;
        assert!(d.is_dirty());
        assert_eq!(*d, 10);
    }

    #[test]
    fn mark_dirty_sets_flag() {
        let mut d = DirtyMarker::clean(1u32);
        d.mark_dirty();
        assert!(d.is_dirty());
    }

    #[test]
    fn mark_clean_clears_flag() {
        let mut d = DirtyMarker::dirty(1u32);
        d.mark_clean();
        assert!(!d.is_dirty());
    }
}
