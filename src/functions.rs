use super::*;

pub(crate) fn default<T: Default>() -> T {
  T::default()
}

pub(crate) fn display<'a, T: Display + 'a>(t: T) -> Box<dyn Display + 'a> {
  Box::new(t)
}

pub(crate) fn pad(i: usize, alignment: usize) -> usize {
  assert!(alignment.is_power_of_two());
  (i + alignment - 1) & !(alignment - 1)
}
