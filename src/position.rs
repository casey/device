use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Position {
  pub(crate) index: u64,
}

impl Position {
  pub(crate) fn bar(self) -> u64 {
    self.index / 16 + 1
  }

  pub(crate) fn beat(self) -> u64 {
    self.index / 4 % 4 + 1
  }

  pub(crate) fn quarter(self) -> u64 {
    self.index % 4 + 1
  }
}

impl Display for Position {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}.{}.{}", self.bar(), self.beat(), self.quarter())
  }
}
