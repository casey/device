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
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mat3f() {
    assert_eq!(
      matrix!(
        0.0, 1.0, 2.0;
        3.0, 4.0, 5.0;
        6.0, 7.0, 8.0;
      )
      .to_affine(),
      matrix!(
        0.0, 1.0, 2.0;
        3.0, 4.0, 5.0;
      ),
    );
  }

  #[test]
  fn mat4f() {
    assert_eq!(
      matrix!(
         0.0,  1.0,  2.0,  3.0;
         4.0,  5.0,  6.0,  7.0;
         8.0,  9.0, 10.0, 11.0;
        12.0, 13.0, 14.0, 15.0;
      )
      .to_affine(),
      matrix!(
         0.0,  1.0,  2.0,  3.0;
         4.0,  5.0,  6.0,  7.0;
         8.0,  9.0, 10.0, 11.0;
      )
    );
  }
}
