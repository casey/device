use super::*;

#[derive(Clone, Debug)]
pub(crate) struct Filter {
  pub(crate) alpha: f32,
  pub(crate) base: f32,
  pub(crate) color: Mat4f,
  pub(crate) color_response: Transformation3,
  pub(crate) color_velocity: Transformation3,
  pub(crate) coordinates: bool,
  pub(crate) elapsed: Duration,
  pub(crate) field: Field,
  pub(crate) grid: f32,
  pub(crate) grid_alpha: f32,
  pub(crate) mirror: Vector2<Mirror>,
  pub(crate) position: Mat3f,
  pub(crate) position_response: Transformation2,
  pub(crate) position_velocity: Transformation2,
  pub(crate) preset: Option<Preset>,
  pub(crate) repeat: bool,
  pub(crate) rms: Mat1x2f,
  pub(crate) wrap: bool,
}

impl Default for Filter {
  fn default() -> Self {
    Self {
      alpha: 1.0,
      base: 1.0,
      color: Mat4f::identity(),
      color_response: Transformation3::default(),
      color_velocity: Transformation3::default(),
      coordinates: false,
      elapsed: Duration::ZERO,
      field: Field::default(),
      grid: 1.0,
      grid_alpha: 0.0,
      mirror: Vector2::default(),
      position: Mat3f::identity(),
      position_response: Transformation2::default(),
      position_velocity: Transformation2::default(),
      preset: None,
      repeat: true,
      rms: Mat1x2f::identity(),
      wrap: false,
    }
  }
}

impl Filter {
  pub(crate) fn color_uniform(&self, response: f32) -> Mat3x4f {
    (self.color_response.response(response)
      * self.color_velocity.response(self.elapsed.as_secs_f32())
      * self.color)
      .to_affine()
  }

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
    (self.position_response.response(response)
      * self.position_velocity.response(self.elapsed.as_secs_f32())
      * self.position)
      .to_affine()
  }

  pub(crate) fn texture_key(&self) -> Option<TextureFieldKey> {
    if let Field::Texture(texture_field) = self.field {
      Some(texture_field.key())
    } else {
      None
    }
  }

  pub(crate) fn tick(&mut self, tick: Tick) {
    self.elapsed += tick.dt;
  }
}
