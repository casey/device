use super::*;

#[derive(Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Viewport {
  Fill { position: Vec2f },
  Fit,
}

impl Default for Viewport {
  fn default() -> Self {
    Self::Fill {
      position: Vec2f::zeros(),
    }
  }
}

impl Viewport {
  pub(crate) fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn toggle(&mut self) {
    *self = match *self {
      Self::Fill { .. } => Self::Fit,
      Self::Fit => Self::Fill {
        position: Vec2f::zeros(),
      },
    };
  }
}

impl Display for Viewport {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
