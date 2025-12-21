use super::*;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) struct ArrayString<const CAPACITY: usize> {
  array: [u8; CAPACITY],
  size: u8,
}

impl<const CAPACITY: usize> ArrayString<CAPACITY> {
  fn as_str(&self) -> &str {
    str::from_utf8(&self.array[..self.size.into_usize()]).unwrap()
  }
}

impl<const CAPACITY: usize> From<&str> for ArrayString<CAPACITY> {
  fn from(s: &str) -> Self {
    let mut array = [0; CAPACITY];

    let size = s.floor_char_boundary(CAPACITY);

    array[..size].copy_from_slice(&s.as_bytes()[..size]);

    Self {
      array,
      size: size.try_into().unwrap(),
    }
  }
}

impl<const CAPACITY: usize> From<String> for ArrayString<CAPACITY> {
  fn from(s: String) -> Self {
    s.as_str().into()
  }
}

impl<const CAPACITY: usize> PartialEq<str> for ArrayString<CAPACITY> {
  fn eq(&self, other: &str) -> bool {
    self.as_str().eq(other)
  }
}

impl<const CAPACITY: usize> Debug for ArrayString<CAPACITY> {
  #[inline]
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_struct("ArrayString")
      .field("array", &self.as_str())
      .field("size", &self.size)
      .finish()
  }
}

impl<const CAPACITY: usize> Deref for ArrayString<CAPACITY> {
  type Target = str;

  fn deref(&self) -> &Self::Target {
    self.as_str()
  }
}

impl<const CAPACITY: usize> AsRef<str> for ArrayString<CAPACITY> {
  fn as_ref(&self) -> &str {
    self.as_str()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn array_string() {
    assert_eq!(&SmallString::from(""), "");
    assert_eq!(&SmallString::from("hello"), "hello");
    assert_eq!(
      &SmallString::from("this string is too long"),
      "this string is ",
    );
  }

  #[test]
  fn size() {
    assert_eq!(size_of::<SmallString>(), size_of::<&str>());
  }
}
