Similar to [`core::cell::LazyCell`], but use `&mut self` to get or initialization

Suitable for being wrapped in a `Mutex`

# Examples

```rust
use lazymut::LazyMut;
use std::sync::Mutex;

static FOO: Mutex<LazyMut<Vec<i32>>> = Mutex::new(LazyMut::new(|| {
    vec![1]
}));

let mut lock = FOO.lock().unwrap();
assert_eq!(lock.get(), &mut vec![1]);
lock.get().push(2);
assert_eq!(lock.get(), &mut vec![1, 2]);
```
