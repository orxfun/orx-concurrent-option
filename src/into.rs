use crate::{concurrent_option::ConcurrentOption, states::*};
use core::sync::atomic::Ordering;

impl<T> ConcurrentOption<T> {
    /// Returns the contained Some value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is a None with a custom panic message provided by
    /// `message`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("value");
    /// assert_eq!(x.expect("fruits are healthy"), "value");
    /// ```
    ///
    /// ```should_panic
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// x.expect("fruits are healthy"); // panics with `fruits are healthy`
    /// ```
    pub fn expect(mut self, message: &str) -> T {
        self.exclusive_take().expect(message)
    }

    /// Returns the contained Some value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the None
    /// case explicitly, or call [`ConcurrentOption::unwrap_or`], [`ConcurrentOption::unwrap_or_else`], or
    /// [`ConcurrentOption::unwrap_or_default`].
    ///
    /// # Panics
    ///
    /// Panics if the self value equals None.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("air");
    /// assert_eq!(x.unwrap(), "air");
    /// ```
    ///
    /// ```should_panic
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(x.unwrap(), "air"); // fails
    /// ```
    pub fn unwrap(self) -> T {
        self.expect("called `unwrap()` on a `None` value")
    }

    /// Returns the contained Some value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`ConcurrentOption::unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// assert_eq!(ConcurrentOption::some("car").unwrap_or("bike"), "car");
    /// assert_eq!(ConcurrentOption::none().unwrap_or("bike"), "bike");
    /// ```
    pub fn unwrap_or(mut self, default: T) -> T {
        self.exclusive_take().unwrap_or(default)
    }

    /// Returns the contained Some value or a default.
    ///
    /// Consumes the `self` argument then, if Some, returns the contained
    /// value, otherwise if None, returns the [default value] for that
    /// type.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<u32> = ConcurrentOption::none();
    /// let y: ConcurrentOption<u32> = ConcurrentOption::some(12);
    ///
    /// assert_eq!(x.unwrap_or_default(), 0);
    /// assert_eq!(y.unwrap_or_default(), 12);
    /// ```
    pub fn unwrap_or_default(mut self) -> T
    where
        T: Default,
    {
        self.exclusive_take().unwrap_or_default()
    }

    /// Returns the contained Some value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let k = 10;
    /// assert_eq!(ConcurrentOption::some(4).unwrap_or_else(|| 2 * k), 4);
    /// assert_eq!(ConcurrentOption::none().unwrap_or_else(|| 2 * k), 20);
    /// ```
    pub fn unwrap_or_else<F>(mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.exclusive_take().unwrap_or_else(f)
    }

    /// Returns the contained Some value, consuming the `self` value,
    /// without checking that the value is not None.
    ///
    /// # Safety
    ///
    /// Calling this method on None is *[undefined behavior]*.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_concurrent_option::*;
    ///
    /// let x = ConcurrentOption::some("air");
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air");
    /// ```
    ///
    /// ```no_run
    /// use orx_concurrent_option::*;
    ///
    /// let x: ConcurrentOption<&str> = ConcurrentOption::none();
    /// assert_eq!(unsafe { x.unwrap_unchecked() }, "air"); // Undefined behavior!
    /// ```
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.state.store(NONE, Ordering::Relaxed);
        let x = &mut *self.value.get();
        x.assume_init_read()
    }
}
