use super::*;

pub(crate) enum Axis {
  #[cfg(test)]
  Red,
  Green,
  Blue,
}

impl Axis {
  pub(crate) fn axis(self) -> Unit<Vec3f> {
    match self {
      #[cfg(test)]
      Self::Red => Vec3f::x_axis(),
      Self::Green => Vec3f::y_axis(),
      Self::Blue => Vec3f::z_axis(),
    }
  }
}
