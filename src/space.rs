use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Space {
  Blaster,
  CenteredRgb,
  Rgb,
  Ycgco,
  Yiq,
}

impl Space {
  fn forward(self) -> Mat4f {
    match self {
      Self::Blaster => color::YIQ.transpose().to_homogeneous(),
      Self::CenteredRgb => color::CENTERED_RGB,
      Self::Rgb => Mat4f::identity(),
      Self::Ycgco => color::YCGCO.to_homogeneous(),
      Self::Yiq => color::YIQ.to_homogeneous(),
    }
  }

  fn inverse(self) -> Mat4f {
    match self {
      Self::Blaster => color::YIQ_INVERSE.transpose().to_homogeneous(),
      Self::CenteredRgb => color::CENTERED_RGB_INVERSE,
      Self::Rgb => Mat4f::identity(),
      Self::Ycgco => color::YCGCO_INVERSE.to_homogeneous(),
      Self::Yiq => color::YIQ_INVERSE.to_homogeneous(),
    }
  }

  pub(crate) fn transform(self, transformation: Mat4f) -> Mat4f {
    self.inverse() * transformation * self.forward()
  }
}
