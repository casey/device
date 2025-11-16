use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct Parameter(i8);

impl From<i8> for Parameter {
  fn from(value: i8) -> Self {
    Self(value.clamp(Self::MIN, Self::MAX))
  }
}

impl Add<i8> for Parameter {
  type Output = Self;

  fn add(self, rhs: i8) -> Self {
    self.0.saturating_add(rhs).into()
  }
}

impl AddAssign<i8> for Parameter {
  fn add_assign(&mut self, rhs: i8) {
    *self = self.0.saturating_add(rhs).into();
  }
}

impl SubAssign<i8> for Parameter {
  fn sub_assign(&mut self, rhs: i8) {
    *self = self.0.saturating_sub(rhs).into();
  }
}

impl Parameter {
  const MAX: i8 = 63;
  const MIN: i8 = -64;

  pub fn bipolar(self) -> f32 {
    if self.0 < 0 {
      f32::from(self.0) / 64.0
    } else {
      f32::from(self.0) / 63.0
    }
  }

  pub fn unipolar(self) -> f32 {
    f32::from(self.0 + 64) / 127.0
  }

  pub fn value(self) -> i8 {
    self.0
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn unipolar() {
    assert_eq!(Parameter::from(Parameter::MIN).unipolar(), 0.0);
    assert_eq!(Parameter::from(Parameter::MAX).unipolar(), 1.0);
  }
}
