use super::*;

pub(crate) fn invert() -> Mat4f {
  Mat3f::from_diagonal(&vector!(-1.0, -1.0, -1.0))
    .to_homogeneous()
    .append_translation(&vector!(1.0, 1.0, 1.0))
}

pub(crate) fn rotate_hue_blaster(r: f32) -> Mat4f {
  Transformation3 {
    space: Space::Blaster,
    rotation: UnitQuaternion::from_axis_angle(&Vec3f::x_axis(), r),
    ..default()
  }
  .response(1.0)
}

pub(crate) fn rotate_hue_yiq(r: f32) -> Mat4f {
  Transformation3 {
    space: Space::Yiq,
    rotation: UnitQuaternion::from_axis_angle(&Vec3f::x_axis(), r),
    ..default()
  }
  .response(1.0)
}

pub(crate) fn saturate(s: f32) -> Mat4f {
  Transformation3 {
    space: Space::Ycgco,
    scaling: Vec3f::new(1.0, s, s),
    ..default()
  }
  .response(1.0)
}
