#![doc = include_str!("../README.md")]
#![no_std]

use core::{fmt::{self, Debug}, mem::replace};

#[doc = include_str!("../README.md")]
#[derive(Default)]
pub struct LazyMut<T, F = fn() -> T> {
    state: State<T, F>,
}

impl<T: Debug, F> Debug for LazyMut<T, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut t = f.debug_tuple("LazyMut");
        match self.state.try_get() {
            Some(val) => t.field(val),
            None => t.field(&format_args!("<uninit>")),
        };
        t.finish()
    }
}

impl<T, F> LazyMut<T, F> {
    /// Creates a new [`LazyMut`]
    pub const fn new(f: F) -> Self {
        Self { state: State::Uninit(f) }
    }

    /// Try get inner value reference
    ///
    /// # Examples
    ///
    /// ```
    /// # use lazymut::LazyMut;
    /// let mut lazy_mut = LazyMut::new(|| 3);
    ///
    /// assert_eq!(lazy_mut.try_get(), None);
    /// assert_eq!(lazy_mut.get(), &mut 3);
    /// assert_eq!(lazy_mut.try_get(), Some(&3));
    /// ```
    pub const fn try_get(&self) -> Option<&T> {
        self.state.try_get()
    }

    /// Try get inner value mut reference
    ///
    /// # Examples
    ///
    /// ```
    /// # use lazymut::LazyMut;
    /// let mut lazy_mut = LazyMut::new(|| 3);
    ///
    /// assert_eq!(lazy_mut.try_get_mut(), None);
    /// assert_eq!(lazy_mut.get(), &mut 3);
    /// assert_eq!(lazy_mut.try_get_mut(), Some(&mut 3));
    /// ```
    pub const fn try_get_mut(&mut self) -> Option<&mut T> {
        self.state.try_get_mut()
    }

    /// Returns value when initialized
    ///
    /// # Panics
    ///
    /// Panics if state is poisoned
    ///
    /// # Examples
    ///
    /// ```
    /// # use lazymut::LazyMut;
    /// let mut lazy_mut = LazyMut::new(|| 3);
    ///
    /// assert_eq!(lazy_mut.get(), &mut 3);
    /// assert_eq!(lazy_mut.into_inner(), Some(3));
    /// ```
    ///
    /// ```
    /// # use lazymut::LazyMut;
    /// let mut lazy_mut = LazyMut::new(|| 3i32);
    ///
    /// # assert_ne!(lazy_mut.try_get(), Some(&3));
    /// assert_eq!(lazy_mut.into_inner(), None);
    /// ```
    pub fn into_inner(self) -> Option<T> {
        match self.state {
            State::Uninit(_) => None,
            State::Poisoned => panic_poisoned(),
            State::Inited(val) => Some(val),
        }
    }
}

impl<T, F: FnOnce() -> T> LazyMut<T, F> {
    /// Get mutable value reference or initialize value
    ///
    /// # Panics
    ///
    /// Panics if state is poisoned or initializer panic
    ///
    /// # Examples
    ///
    /// ```
    /// use lazymut::LazyMut;
    ///
    /// let mut lazy_mut = LazyMut::new(|| vec![1]);
    ///
    /// assert_eq!(lazy_mut.get(), &mut vec![1]);
    /// lazy_mut.get().push(2);
    /// assert_eq!(lazy_mut.get(), &mut vec![1, 2]);
    /// ```
    pub fn get(&mut self) -> &mut T {
        self.state.get_or_init()
    }
}

#[derive(Clone)]
enum State<T, F> {
    Uninit(F),
    Inited(T),
    Poisoned,
}

impl<T: Default, F> Default for State<T, F> {
    fn default() -> Self {
        Self::Inited(Default::default())
    }
}

impl<T, F> State<T, F>
where F: FnOnce() -> T,
{
    fn get_or_init(&mut self) -> &mut T {
        match self {
            State::Poisoned => panic_poisoned(),
            State::Inited(val) => val,
            State::Uninit(_) => {
                let this = replace(self, Self::Poisoned);
                let Self::Uninit(f) = this else { unreachable!() };
                *self = State::Inited(f());
                self.try_get_mut().unwrap()
            },
        }
    }
}

impl<T, F> State<T, F> {
    const fn try_get(&self) -> Option<&T> {
        if let Self::Inited(v) = self {
            Some(v)
        } else {
            None
        }
    }

    const fn try_get_mut(&mut self) -> Option<&mut T> {
        if let Self::Inited(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[cold]
#[inline(never)]
fn panic_poisoned() -> ! {
    panic!("LazyMut instance has previously been poisoned")
}
