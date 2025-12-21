use super::*;

#[derive(Clone, Copy, Debug, EnumIter, IntoStaticStr)]
#[repr(u32)]
pub(crate) enum Field {
  All,
  Bottom,
  Circle { radius: f32 },
  Cross,
  Frequencies,
  Left,
  None,
  Right,
  Samples,
  Square,
  Texture(&'static str),
  Top,
  Triangle,
  X,
}

#[allow(clippy::derivable_impls)]
impl Default for Field {
  fn default() -> Self {
    Field::All
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
      Self::Circle { .. } => '●',
      Self::Cross => '✚',
      Self::Frequencies => 'F',
      Self::Left => 'L',
      Self::None => '□',
      Self::Right => 'R',
      Self::Samples => 'S',
      Self::Square => '■',
      Self::Texture(_) => '▧',
      Self::Top => 'T',
      Self::Triangle => '▲',
      Self::X => 'X',
    }
  }

  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn number(self) -> u32 {
    unsafe { *(&raw const self).cast() }
  }

  pub(crate) fn parameter(self) -> f32 {
    match self {
      Self::Circle { radius } => radius,
      _ => 0.0,
    }
  }
}
