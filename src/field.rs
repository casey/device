use super::*;

#[derive(Clone, Copy, EnumIter, IntoStaticStr)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Bottom,
  Circle,
  Cross,
  Frequencies,
  Left,
  None,
  Right,
  Samples,
  Square,
  Top,
  Triangle,
  X,
}

#[allow(clippy::derivable_impls)]
impl Default for Field {
  fn default() -> Self {
    Field::None
  }
}

impl Field {
  pub(crate) fn constant(self) -> String {
    format!("FIELD_{}", self.name().to_uppercase())
  }

  pub(crate) fn function(self) -> String {
    format!("field_{}", self.name().to_lowercase())
  }

  pub(crate) fn icon(self) -> char {
    match self {
      Self::All => 'A',
      Self::Bottom => 'B',
      Self::Circle => '●',
      Self::Cross => '✚',
      Self::Frequencies => 'F',
      Self::Left => 'L',
      Self::None => '□',
      Self::Right => 'R',
      Self::Samples => 'S',
      Self::Square => '■',
      Self::Top => 'T',
      Self::Triangle => '▲',
      Self::X => 'X',
    }
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }
}
