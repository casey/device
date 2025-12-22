use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Position {
  quarter: u64,
}

impl Position {
  pub(crate) fn from_beat(beat: u64) -> Self {
    Self { quarter: beat * 4 }
  }

  pub(crate) fn from_quarter(quarter: u64) -> Self {
    Self { quarter }
  }
}

impl Display for Position {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}.{}.{}",
      self.quarter / 16 + 1,
      self.quarter / 4 % 4 + 1,
      self.quarter % 4 + 1
    )
  }
}
