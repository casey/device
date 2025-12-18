#[derive(Clone, Copy)]
pub(crate) struct Alignment(usize);

impl Alignment {
  pub(crate) fn max(self, other: Self) -> Self {
    Self(self.0.max(other.0))
  }

  pub(crate) const fn new(alignment: usize) -> Self {
    assert!(alignment.is_power_of_two());
    Self(alignment)
  }

  pub(crate) fn pad(self, i: usize) -> usize {
    (i + self.0 - 1) & !(self.0 - 1)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn pad() {
    assert_eq!(Alignment::new(1).pad(0), 0);
    assert_eq!(Alignment::new(1).pad(1), 1);
    assert_eq!(Alignment::new(2).pad(1), 2);
    assert_eq!(Alignment::new(4).pad(1), 4);
    assert_eq!(Alignment::new(4).pad(4), 4);
  }
}
