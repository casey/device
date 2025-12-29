use super::*;

#[derive(Clone, Copy)]
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
    match self {
      Self::Fill { .. } => "Fill",
      Self::Fit => "Fit",
    }
  }

  pub(crate) fn toggle(&mut self) {
    *self = match *self {
      Self::Fit => Self::Fill {
        position: Vec2f::zeros(),
      },
      Self::Fill { .. } => Self::Fit,
    };
  }
}

impl Display for Viewport {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
