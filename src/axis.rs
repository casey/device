use super::*;

pub(crate) enum Axis {
  Blue,
  Green,
  #[cfg(test)]
  Red,
}

impl Axis {
  pub(crate) fn axis(self) -> Unit<Vec3f> {
    match self {
      Self::Blue => Vec3f::z_axis(),
      Self::Green => Vec3f::y_axis(),
      #[cfg(test)]
      Self::Red => Vec3f::x_axis(),
    }
  }
}
