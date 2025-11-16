use super::*;

#[derive(Clone)]
pub struct Filter {
  pub color: Mat4f,
  pub coordinates: bool,
  pub field: Field,
  pub position: Mat3f,
  pub wrap: bool,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      color: Mat4f::identity(),
      coordinates: false,
      field: Field::default(),
      position: Mat3f::identity(),
      wrap: false,
    }
  }
}

impl Filter {
  pub fn icon(&self) -> char {
    self.field.icon()
  }
}
