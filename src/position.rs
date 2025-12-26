use super::*;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Position {
  quarter: u64,
}

impl Position {
  pub(crate) const fn bar(self) -> u64 {
    self.quarter / 16
  }

  pub(crate) const fn beat(self) -> u64 {
    self.quarter / 4
  }

  pub(crate) const fn from_bar(bar: u64) -> Self {
    Self { quarter: bar * 16 }
  }

  pub(crate) const fn from_bar_beat_quarter(bar: u64, beat: u64, quarter: u64) -> Self {
    Self {
      quarter: bar * 16 + beat * 4 + quarter,
    }
  }

  pub(crate) const fn from_beat(beat: u64) -> Self {
    Self { quarter: beat * 4 }
  }

  pub(crate) const fn from_quarter(quarter: u64) -> Self {
    Self { quarter }
  }

  pub(crate) const fn quarter(self) -> u64 {
    self.quarter
  }
}

impl Add for Position {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      quarter: self.quarter.checked_add(other.quarter).unwrap(),
    }
  }
}

impl Sub for Position {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      quarter: self.quarter.checked_sub(other.quarter).unwrap(),
    }
  }
}

impl Display for Position {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}.{}.{}",
      self.bar() + 1,
      self.beat() % 4 + 1,
      self.quarter() % 4 + 1
    )
  }
}

pub(crate) const fn bar(bar: u64) -> Position {
  Position::from_bar(bar.checked_sub(1).unwrap())
}

pub(crate) fn bars(range: Range<u64>) -> impl Iterator<Item = Position> {
  range.map(|bar| Position::from_bar(bar.checked_sub(1).unwrap()))
}

pub(crate) const fn bbq(bar: u64, beat: u64, quarter: u64) -> Position {
  Position::from_bar_beat_quarter(
    bar.checked_sub(1).unwrap(),
    beat.checked_sub(1).unwrap(),
    quarter.checked_sub(1).unwrap(),
  )
}

pub(crate) const fn beat(beat: u64) -> Position {
  Position::from_beat(beat.checked_sub(1).unwrap())
}

pub(crate) const fn quarter(quarter: u64) -> Position {
  Position::from_quarter(quarter.checked_sub(1).unwrap())
}
