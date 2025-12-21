use super::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TextureField {
  pub(crate) position: Vec2f,
  pub(crate) scale: f32,
  pub(crate) text: &'static str,
  pub(crate) weight: FontWeight,
}

impl TextureField {
  pub(crate) fn key(self) -> TextureFieldKey {
    (
      Vector2::new(self.position.x.into(), self.position.y.into()),
      self.scale.into(),
      self.text,
      self.weight.value().into(),
    )
  }
}
