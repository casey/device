use super::*;

const CENTERED_RGB: Mat4f = matrix!(
  2.0, 0.0, 0.0, -1.0;
  0.0, 2.0, 0.0, -1.0;
  0.0, 0.0, 2.0, -1.0;
  0.0, 0.0, 0.0,  1.0;
);

const CENTERED_RGB_INVERSE: Mat4f = matrix!(
  0.5, 0.0, 0.0, 0.5;
  0.0, 0.5, 0.0, 0.5;
  0.0, 0.0, 0.5, 0.5;
  0.0, 0.0, 0.0, 1.0;
);

const YCGCO: Mat3f = matrix!(
   0.25, 0.5,  0.25;
   0.50, 0.0, -0.50;
  -0.25, 0.5, -0.25;
);

const YCGCO_INVERSE: Mat3f = matrix!(
  1.0,  1.0, -1.0;
  1.0,  0.0,  1.0;
  1.0, -1.0, -1.0;
);

const YIQ: Mat3f = matrix!(
  0.2990,  0.5870,  0.1140;
  0.5959, -0.2746, -0.3213;
  0.2115, -0.5227,  0.3112;
);

const YIQ_INVERSE: Mat3f = matrix!(
  1.0,  0.956,  0.619;
  1.0, -0.272, -0.647;
  1.0, -1.106,  1.703;
);

#[derive(Clone, Copy, Debug, PartialEq, EnumIter)]
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
      Self::Blaster => YIQ.transpose().to_homogeneous(),
      Self::CenteredRgb => CENTERED_RGB,
      Self::Rgb => Mat4f::identity(),
      Self::Ycgco => YCGCO.to_homogeneous(),
      Self::Yiq => YIQ.to_homogeneous(),
    }
  }

  fn inverse(self) -> Mat4f {
    match self {
      Self::Blaster => YIQ_INVERSE.transpose().to_homogeneous(),
      Self::CenteredRgb => CENTERED_RGB_INVERSE,
      Self::Rgb => Mat4f::identity(),
      Self::Ycgco => YCGCO_INVERSE.to_homogeneous(),
      Self::Yiq => YIQ_INVERSE.to_homogeneous(),
    }
  }

  pub(crate) fn transform(self, transformation: Mat4f) -> Mat4f {
    self.inverse() * transformation * self.forward()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn composed_transformation_is_identity_transformation() {
    for space in Space::iter() {
      let net = space.inverse() * space.forward();
      assert!(net.is_identity(0.001));
    }
  }
}
