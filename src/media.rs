use super::*;

#[derive(Debug)]
pub(crate) struct Media {
  pub(crate) font_stack: FontStack<'static>,
  pub(crate) image: Option<ImageData>,
  pub(crate) position: Vec2f,
  pub(crate) scale: f32,
  pub(crate) text: String,
  pub(crate) weight: FontWeight,
}

impl Media {
  pub(crate) fn font_stack(mut self, font_stack: FontStack<'static>) -> Self {
    self.font_stack = font_stack;
    self
  }

  pub(crate) fn handle(self) -> MediaHandle {
    self.into()
  }

  pub(crate) fn image(mut self, image: ImageData) -> Self {
    self.image = Some(image);
    self
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  pub(crate) fn position(mut self, position: Vec2f) -> Self {
    self.position = position;
    self
  }

  pub(crate) fn scale(mut self, scale: f32) -> Self {
    self.scale = scale;
    self
  }

  pub(crate) fn text(mut self, text: impl Into<String>) -> Self {
    self.text = text.into();
    self
  }

  pub(crate) fn weight(mut self, weight: FontWeight) -> Self {
    self.weight = weight;
    self
  }
}

impl Default for Media {
  fn default() -> Self {
    Self {
      font_stack: DEFAULT_FONT_STACK,
      image: None,
      position: Vec2f::default(),
      scale: 1.0,
      text: String::new(),
      weight: FontWeight::NORMAL,
    }
  }
}
