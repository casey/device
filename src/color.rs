use super::*;

#[rustfmt::skip]
const YCGCO: Mat3f = Mat3f::new(
  0.25, 0.5, 0.25,
  0.5, 0.0, -0.5,
  -0.25, 0.5, -0.25,
);

#[rustfmt::skip]
const YCGCO_INVERSE: Mat3f = Mat3f::new(
  1.0, 1.0, -1.0,
  1.0, 0.0, 1.0,
  1.0, -1.0, -1.0,
);

#[rustfmt::skip]
const YIQ: Mat3f = Mat3f::new(
  0.299,   0.587,   0.114,
  0.5959, -0.2746, -0.3213,
  0.2115, -0.5227,  0.3112,
);

#[rustfmt::skip]
const YIQ_INVERSE: Mat3f = Mat3f::new(
  1.0, 0.956, 0.619,
  1.0, -0.272, -0.647,
  1.0, -1.106, 1.703,
);

pub(crate) fn invert() -> Mat4f {
  Mat3f::from_diagonal(&Vec3f::new(-1.0, -1.0, -1.0))
    .to_homogeneous()
    .append_translation(&Vec3f::new(1.0, 1.0, 1.0))
}

pub(crate) fn rotate_hue_blaster(r: f32) -> Mat4f {
  let mut rotation = Mat3f::identity();

  rotation
    .fixed_view_mut::<2, 2>(1, 1)
    .copy_from(Rot2f::new(r).matrix());

  (YIQ_INVERSE.transpose() * rotation * YIQ.transpose()).to_homogeneous()
}

pub(crate) fn rotate_hue_yiq(r: f32) -> Mat4f {
  let mut rotation = Mat3f::identity();

  rotation
    .fixed_view_mut::<2, 2>(1, 1)
    .copy_from(Rot2f::new(r).matrix());

  (YIQ_INVERSE * rotation * YIQ).to_homogeneous()
}

pub(crate) fn saturate(s: f32) -> Mat4f {
  (YCGCO_INVERSE * Mat3f::from_diagonal(&Vec3f::new(1.0, s, s)) * YCGCO).to_homogeneous()
}
