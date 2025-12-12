#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Mirror {
  #[default]
  Off,
  Triangle,
  Inverse,
}

impl Mirror {
  pub(crate) fn is_on(self) -> bool {
    !matches!(self, Self::Off)
  }

  pub(crate) fn select(self) -> u8 {
    match self {
      Self::Off | Self::Triangle => 0,
      Self::Inverse => 1,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_on() {
    assert!(!Mirror::Off.is_on());
    assert!(Mirror::Triangle.is_on());
    assert!(Mirror::Inverse.is_on());
  }

  #[test]
  fn select() {
    assert_eq!(Mirror::Off.select(), 0);
    assert_eq!(Mirror::Triangle.select(), 0);
    assert_eq!(Mirror::Inverse.select(), 1);
  }
}
