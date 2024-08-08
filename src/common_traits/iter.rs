use crate::ConcurrentOption;
use std::{iter::FusedIterator, sync::atomic::Ordering};

// INTO-ITER

impl<'a, T> IntoIterator for &'a ConcurrentOption<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter(Ordering::Relaxed)
    }
}

impl<'a, T> IntoIterator for &'a mut ConcurrentOption<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> IntoIterator for ConcurrentOption<T> {
    type Item = T;

    type IntoIter = std::option::IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        self.take().into_iter()
    }
}

// ITER

/// Iterator over the `ConcurrentOption` yielding at most one element.
pub struct Iter<'a, T> {
    pub(crate) maybe: Option<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.maybe.take()
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        match self.maybe.is_some() {
            true => 1,
            false => 0,
        }
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
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

impl<'a, T> FusedIterator for IterMut<'a, T> {}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        match self.maybe.is_some() {
            true => 1,
            false => 0,
        }
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
