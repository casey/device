#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) enum Mirror {
  Inverse,
  #[default]
  Off,
  Triangle,
}

impl Mirror {
  pub(crate) fn is_on(self) -> bool {
    matches!(self, Self::Inverse | Self::Triangle)
  }

  pub(crate) fn select(self) -> f32 {
    match self {
      Self::Inverse => 1.0,
      Self::Off | Self::Triangle => 0.0,
    }
  }
}
