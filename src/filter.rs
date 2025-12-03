use super::*;

#[derive(Clone)]
pub(crate) struct Filter {
  pub(crate) base: f32,
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) position: Mat3f,
  pub(crate) repeat: bool,
  pub(crate) wrap: bool,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      base: 1.0,
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::default(),
      position: Mat3f::identity(),
      repeat: true,
      wrap: true,
    }
  }
}

impl Filter {
  pub(crate) fn icon(&self) -> char {
    self.field.icon()
  }
}
