use super::*;

static KEY: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct TextureField {
  pub(crate) font_stack: FontStack<'static>,
  key: u64,
  pub(crate) position: Vec2f,
  pub(crate) scale: f32,
  pub(crate) text: SmallString,
  pub(crate) weight: FontWeight,
}

impl TextureField {
  pub(crate) fn font_stack(mut self, font_stack: FontStack<'static>) -> Self {
    self.font_stack = font_stack;
    self
  }

  pub(crate) fn key(&self) -> u64 {
    self.key
  }

  pub(crate) fn position(mut self, position: Vec2f) -> Self {
    self.position = position;
    self
  }

  pub(crate) fn scale(mut self, scale: f32) -> Self {
    self.scale = scale;
    self
  }

  pub(crate) fn text(mut self, text: impl Into<SmallString>) -> Self {
    self.text = text.into();
    self
  }

  pub(crate) fn weight(mut self, weight: FontWeight) -> Self {
    self.weight = weight;
    self
  }
}

impl Default for TextureField {
  fn default() -> Self {
    Self {
      font_stack: DEFAULT_FONT_STACK,
      key: KEY.fetch_add(1, atomic::Ordering::Relaxed),
      position: Vec2f::zeros(),
      scale: 1.0,
      text: "".into(),
      weight: FontWeight::NORMAL,
    }
  }
}
