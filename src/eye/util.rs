use std::ops::{Deref, DerefMut};

pub struct SendWrapper<T> {
    inner: T,
}

impl<T> SendWrapper<T> {
    pub unsafe fn new(val: T) -> Self {
        SendWrapper { inner: val }
    }
}

impl<T> Deref for SendWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for SendWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

unsafe impl<T> Send for SendWrapper<T> {}
