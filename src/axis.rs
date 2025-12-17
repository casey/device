use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Axis {
  Red,
  Green,
  Blue,
}

impl Axis {
  pub(crate) fn axis(self) -> Unit<Vec3f> {
    match self {
      Self::Red => Vec3f::x_axis(),
      Self::Green => Vec3f::y_axis(),
      Self::Blue => Vec3f::z_axis(),
    }
  }

  pub(crate) fn invert(self) -> Mat4f {
    match self {
      Self::Red => Mat3f::from_diagonal(&Vec3f::new(-1.0, 1.0, 1.0))
        .to_homogeneous()
        .append_translation(&Vec3f::new(1.0, 0.0, 0.0)),
      Self::Green => Mat3f::from_diagonal(&Vec3f::new(1.0, -1.0, 1.0))
        .to_homogeneous()
        .append_translation(&Vec3f::new(0.0, 1.0, 0.0)),
      Self::Blue => Mat3f::from_diagonal(&Vec3f::new(1.0, 1.0, -1.0))
        .to_homogeneous()
        .append_translation(&Vec3f::new(0.0, 0.0, 1.0)),
    }
  }

  pub(crate) fn rotate(self, angle: f32) -> Mat4f {
    Space::CenteredRgb.transform(Mat4f::from_axis_angle(&self.axis(), angle))
  }
}
