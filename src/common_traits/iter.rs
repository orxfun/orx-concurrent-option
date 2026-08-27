use crate::{ConcurrentOption, handle::Handle, states::SOME};
use core::iter::FusedIterator;

// INTO-ITER

impl<'a, T> IntoIterator for &'a ConcurrentOption<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        // hold the reservation for the lifetime of the iterator so that a concurrent mutation
        // cannot race with the borrowed value while the iterator is alive.
        match self.spin_get_handle(SOME, SOME) {
            Some(handle) => Iter {
                maybe: Some(unsafe { (*self.value.get()).assume_init_ref() }),
                _handle: Some(handle),
            },
            None => Iter {
                maybe: None,
                _handle: None,
            },
        }
    }
}

impl<'a, T> IntoIterator for &'a mut ConcurrentOption<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.exclusive_iter_mut()
    }
}

impl<T> IntoIterator for ConcurrentOption<T> {
    type Item = T;

    type IntoIter = core::option::IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.exclusive_take().into_iter()
    }
}

// ITER

/// Iterator over the `ConcurrentOption` yielding at most one element.
pub struct Iter<'a, T> {
    pub(crate) maybe: Option<&'a T>,
    /// keeps the option reserved for the lifetime of the iterator when constructed safely
    pub(crate) _handle: Option<Handle<'a>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.maybe.take()
    }
}

impl<T> FusedIterator for Iter<'_, T> {}

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        match self.maybe.is_some() {
            true => 1,
            false => 0,
        }
    }
}

impl<T> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

// ITER-MUT

/// Mutable iterator over the `ConcurrentOption` yielding at most one element.
pub struct IterMut<'a, T> {
    pub(crate) maybe: Option<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.maybe.take()
    }
}

impl<T> FusedIterator for IterMut<'_, T> {}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        match self.maybe.is_some() {
            true => 1,
            false => 0,
        }
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
