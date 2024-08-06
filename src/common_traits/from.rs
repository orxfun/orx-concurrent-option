use crate::ConcurrentOption;

// FROM

impl<T> From<T> for ConcurrentOption<T> {
    fn from(value: T) -> Self {
        ConcurrentOption::some(value)
    }
}

impl<T> From<Option<T>> for ConcurrentOption<T> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(value) => ConcurrentOption::some(value),
            None => ConcurrentOption::none(),
        }
    }
}

// INTO

impl<T> From<ConcurrentOption<T>> for Option<T> {
    fn from(mut value: ConcurrentOption<T>) -> Self {
        value.take()
    }
}
