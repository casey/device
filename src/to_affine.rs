use super::*;

pub(crate) trait ToAffine {
  type R;

  fn to_affine(self) -> Self::R;
}

impl ToAffine for Mat3f {
  type R = Mat2x3f;

  fn to_affine(self) -> Self::R {
    self.fixed_view::<2, 3>(0, 0).into()
  }
}

impl ToAffine for Mat4f {
  type R = Mat3x4f;

  fn to_affine(self) -> Self::R {
    self.fixed_view::<3, 4>(0, 0).into()
  }
}
