use super::*;

#[derive(Clone, Copy, Default, IntoStaticStr)]
pub(crate) enum Viewport {
  #[default]
  Fill,
  Fit,
}

impl Viewport {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn toggle(&mut self) {
    *self = match self {
      Self::Fit => Self::Fill,
      Self::Fill => Self::Fit,
    };
  }
}

impl Display for Viewport {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
