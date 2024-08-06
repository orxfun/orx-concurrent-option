use crate::ConcurrentOption;

pub trait IntoOption<T> {
    fn into_option(self) -> Option<T>;
}

impl<T> IntoOption<T> for Option<T> {
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T> IntoOption<T> for ConcurrentOption<T> {
    fn into_option(mut self) -> Option<T> {
        self.take()
    }
}
