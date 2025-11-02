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
