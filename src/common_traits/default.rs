use crate::ConcurrentOption;

impl<T> Default for ConcurrentOption<T> {
    fn default() -> Self {
        Self::none()
    }
}
