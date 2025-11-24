use super::*;

pub(crate) struct State {
  pub(crate) alpha: Parameter,
  pub(crate) db: f32,
  pub(crate) filter: Filter,
  pub(crate) filters: Vec<Filter>,
  pub(crate) fit: bool,
  pub(crate) fps: Option<Fps>,
  pub(crate) interpolate: bool,
  pub(crate) parameter: Parameter,
  pub(crate) repeat: bool,
  pub(crate) resolution: Option<NonZeroU32>,
  pub(crate) spread: bool,
  pub(crate) status: bool,
  pub(crate) text: Option<Text>,
  pub(crate) tile: bool,
  pub(crate) wrap: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      alpha: Parameter::default(),
      db: 0.0,
      filter: Filter::default(),
      filters: Vec::new(),
      fit: false,
      fps: None,
      interpolate: false,
      parameter: Parameter::default(),
      repeat: false,
      resolution: None,
      spread: false,
      status: false,
      text: None,
      tile: false,
      wrap: true,
    }
  }
}

impl State {
  pub(crate) fn all(mut self) -> Self {
    self.filter.field = Field::All;
    self
  }

  pub(crate) fn bottom(mut self) -> Self {
    self.filter.field = Field::Bottom;
    self
  }

  pub(crate) fn circle(mut self) -> Self {
    self.filter.field = Field::Circle;
    self
  }

  pub(crate) fn frequencies(mut self) -> Self {
    self.filter.field = Field::Frequencies;
    self
  }

  #[cfg(test)]
  pub(crate) fn interpolate(mut self, interpolate: bool) -> Self {
    self.interpolate = interpolate;
    self
  }

  pub(crate) fn invert(mut self) -> Self {
    self.filter.color = invert_color();
    self
  }

  pub(crate) fn invert_r(mut self) -> Self {
    self.filter.color = Mat4f::from_diagonal(&Vec4f::new(-1.0, 1.0, 1.0, 1.0));
    self
  }

  #[cfg(test)]
  pub(crate) fn left(mut self) -> Self {
    self.filter.field = Field::Left;
    self
  }

  pub(crate) fn push(mut self) -> Self {
    self.filters.push(self.filter.clone());
    self
  }

  pub(crate) fn samples(mut self) -> Self {
    self.filter.field = Field::Samples;
    self
  }

  pub(crate) fn scale(mut self, n: f32) -> Self {
    self.filter.position *= Mat3f::new_scaling(n);
    self
  }

  pub(crate) fn text(mut self, text: Option<Text>) -> Self {
    self.text = text;
    self
  }

  #[cfg(test)]
  pub(crate) fn tile(mut self, tile: bool) -> Self {
    self.tile = tile;
    self
  }

  pub(crate) fn times(mut self, n: usize) -> Self {
    for _ in 0..n {
      self = self.push();
    }
    self
  }

  pub(crate) fn top(mut self) -> Self {
    self.filter.field = Field::Top;
    self
  }

  pub(crate) fn x(mut self) -> Self {
    self.filter.field = Field::X;
    self
  }
}
