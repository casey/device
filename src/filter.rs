use super::*;

#[derive(Clone, Debug)]
pub(crate) struct Filter {
  pub(crate) alpha: f32,
  pub(crate) base: f32,
  pub(crate) color: Mat4f,
  pub(crate) coordinates: bool,
  pub(crate) field: Field,
  pub(crate) grid: f32,
  pub(crate) grid_alpha: f32,
  pub(crate) mirror: Vector2<Mirror>,
  pub(crate) position: Mat3f,
  pub(crate) preset: Option<Preset>,
  pub(crate) repeat: bool,
  pub(crate) rms: Mat1x2f,
  pub(crate) rotation: f32,
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
      grid: 1.0,
      grid_alpha: 0.0,
      mirror: Vector2::default(),
      position: Mat3f::identity(),
      preset: None,
      repeat: true,
      rms: Mat1x2f::identity(),
      rotation: 0.0,
      wrap: false,
    }
  }
}

impl Filter {
  pub(crate) fn icon(&self) -> char {
    self.field.icon()
  }

  pub(crate) fn mirror_uniform(&self) -> Vec4f {
    Vec4f::new(
      self.mirror.x.is_on().into_f32(),
      self.mirror.y.is_on().into_f32(),
      self.mirror.x.select().into(),
      self.mirror.y.select().into(),
    )
  }

  pub(crate) fn position_uniform(&self, response: f32) -> Mat2x3f {
    (Rot2f::new(self.rotation * response).to_homogeneous() * self.position).to_affine()
  }
}
