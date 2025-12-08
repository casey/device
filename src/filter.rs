use super::*;

#[derive(Clone)]
pub(crate) struct Filter {
  pub(crate) alpha: f32,
  pub(crate) base: f32,
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) position: Mat3f,
  pub(crate) wrap: bool,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      alpha: 1.0,
      base: 1.0,
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::default(),
      position: Mat3f::identity(),
      wrap: false,
    }
  }
}

impl Filter {
  pub(crate) fn icon(&self) -> char {
    self.field.icon()
  }
}
