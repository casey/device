use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureField {
  pub(crate) position: Vec2f,
  pub(crate) scale: f32,
  pub(crate) text: SmallString,
  pub(crate) weight: FontWeight,
}

impl Default for TextureField {
  fn default() -> Self {
    Self {
      position: Vec2f::zeros(),
      scale: 1.0,
      text: "".into(),
      weight: FontWeight::NORMAL,
    }
  }
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
