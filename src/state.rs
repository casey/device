use super::*;

#[derive(Default)]
pub struct State {
  pub alpha: Parameter,
  pub db: f32,
  pub filter: Filter,
  pub filters: Vec<Filter>,
  pub parameter: Parameter,
  pub text: Option<Text>,
}

impl State {
  pub fn all(mut self) -> Self {
    self.filter.field = Field::All;
    self
  }

  pub fn bottom(mut self) -> Self {
    self.filter.field = Field::Bottom;
    self
  }

  pub fn circle(mut self) -> Self {
    self.filter.field = Field::Circle;
    self
  }

  pub fn db(mut self, db: i8) -> Self {
    self.db = db as f32;
    self
  }

  pub fn frequencies(mut self) -> Self {
    self.filter.field = Field::Frequencies;
    self
  }

  pub fn invert(mut self) -> Self {
    self.filter.color = invert_color();
    self
  }

  pub fn invert_r(mut self) -> Self {
    self.filter.color = Mat4f::from_diagonal(&Vec4f::new(-1.0, 1.0, 1.0, 1.0));
    self
  }

  pub fn push(mut self) -> Self {
    self.filters.push(self.filter.clone());
    self
  }

  pub fn samples(mut self) -> Self {
    self.filter.field = Field::Samples;
    self
  }

  pub fn scale(mut self, n: f32) -> Self {
    self.filter.position *= Mat3f::new_scaling(n);
    self
  }

  pub fn text(mut self, text: Option<Text>) -> Self {
    self.text = text;
    self
  }

  pub fn times(mut self, n: usize) -> Self {
    for _ in 0..n {
      self = self.push();
    }
    self
  }

  pub fn top(mut self) -> Self {
    self.filter.field = Field::Top;
    self
  }

  pub fn x(mut self) -> Self {
    self.filter.field = Field::X;
    self
  }
}
