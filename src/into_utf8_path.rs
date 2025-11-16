use super::*;

pub(crate) trait IntoUtf8Path<T>: Sized {
  fn into_utf8_path(self) -> Result<T>;
}

impl<'a> IntoUtf8Path<&'a Utf8Path> for &'a std::path::Path {
  fn into_utf8_path(self) -> Result<&'a Utf8Path> {
    self.try_into().context(error::PathUnicode { path: self })
  }
}
