use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
pub(crate) enum Axis {
  Red,
  Green,
  Blue,
}

impl Axis {
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
    const FROM_RGB: Mat4f = Mat4f::new(
      2.0, 0.0, 0.0, -1.0, 0.0, 2.0, 0.0, -1.0, 0.0, 0.0, 2.0, -1.0, 0.0, 0.0, 0.0, 1.0,
    );

    const TO_RGB: Mat4f = Mat4f::new(
      0.5, 0.0, 0.0, 0.5, 0.0, 0.5, 0.0, 0.5, 0.0, 0.0, 0.5, 0.5, 0.0, 0.0, 0.0, 1.0,
    );

    let axis = match self {
      Self::Red => Vec3f::x_axis(),
      Self::Green => Vec3f::y_axis(),
      Self::Blue => Vec3f::z_axis(),
    };

    let transformation = Mat4f::from_axis_angle(&axis, angle);

    TO_RGB * transformation * FROM_RGB
  }
}
