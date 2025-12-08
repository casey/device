use super::*;

pub(crate) struct State {
  pub(crate) alpha: Parameter,
  pub(crate) callback: Option<Box<dyn FnMut(&mut Self, f32)>>,
  pub(crate) db: f32,
  pub(crate) filter: Filter,
  pub(crate) filters: Vec<Filter>,
  pub(crate) fit: bool,
  pub(crate) fps: Option<Fps>,
  pub(crate) interpolate: bool,
  pub(crate) parameter: Parameter,
  pub(crate) position: Vec4f,
  pub(crate) repeat: bool,
  pub(crate) resolution: Option<NonZeroU32>,
  pub(crate) spread: bool,
  pub(crate) status: bool,
  pub(crate) text: Option<Text>,
  pub(crate) tile: bool,
  pub(crate) velocity: Vec4f,
  pub(crate) wrap: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      alpha: Parameter::default(),
      callback: None,
      db: 0.0,
      filter: Filter::default(),
      filters: Vec::new(),
      fit: false,
      fps: None,
      interpolate: false,
      parameter: Parameter::default(),
      position: Vec4f::new(0.0, 0.0, 1.0, 0.0),
      repeat: false,
      resolution: None,
      spread: false,
      status: false,
      text: None,
      tile: false,
      velocity: Vec4f::zeros(),
      wrap: true,
    }
  }
}

impl State {
  pub(crate) fn all(&mut self) -> &mut Self {
    self.filter.field = Field::All;
    self
  }

  #[allow(unused)]
  pub(crate) fn base(&mut self, base: f32) -> &mut Self {
    self.filter.base = base;
    self
  }

  pub(crate) fn bottom(&mut self) -> &mut Self {
    self.filter.field = Field::Bottom;
    self
  }

  pub(crate) fn callback(&mut self, callback: impl FnMut(&mut State, f32) + 'static) -> &mut Self {
    self.callback = Some(Box::new(callback));
    self
  }

  pub(crate) fn circle(&mut self) -> &mut Self {
    self.filter.field = Field::Circle { size: None };
    self
  }

  #[cfg(test)]
  pub(crate) fn coordinates(&mut self, coordinates: bool) -> &mut Self {
    self.filter.coordinates = coordinates;
    self
  }

  #[cfg(test)]
  pub(crate) fn cross(&mut self) -> &mut Self {
    self.filter.field = Field::Cross;
    self
  }

  pub(crate) fn db(&mut self, db: f32) -> &mut Self {
    self.db = db;
    self
  }

  pub(crate) fn field(&mut self, field: Field) -> &mut Self {
    self.filter.field = field;
    self
  }

  pub(crate) fn frequencies(&mut self) -> &mut Self {
    self.filter.field = Field::Frequencies;
    self
  }

  pub(crate) fn identity(&mut self) -> &mut Self {
    self.filter.color = Mat4f::identity();
    self
  }

  pub(crate) fn interpolate(&mut self, interpolate: bool) -> &mut Self {
    self.interpolate = interpolate;
    self
  }

  pub(crate) fn invert(&mut self) -> &mut Self {
    self.filter.color = invert_color();
    self
  }

  pub(crate) fn invert_r(&mut self) -> &mut Self {
    self.filter.color = Mat4f::from_diagonal(&Vec4f::new(-1.0, 1.0, 1.0, 1.0));
    self
  }

  #[cfg(test)]
  pub(crate) fn left(&mut self) -> &mut Self {
    self.filter.field = Field::Left;
    self
  }

  pub(crate) fn none(&mut self) -> &mut Self {
    self.filter.field = Field::None;
    self
  }

  pub(crate) fn position(&mut self, position: Mat3f) -> &mut Self {
    self.filter.position = position;
    self
  }

  pub(crate) fn push(&mut self) -> &mut Self {
    self.filters.push(self.filter.clone());
    self
  }

  pub(crate) fn repeat(&mut self, repeat: bool) -> &mut Self {
    self.repeat = repeat;
    self
  }

  pub(crate) fn rotate_color(&mut self, axis: Axis, angle: f32) -> &mut Self {
    self.filter.color = Mat4f::from_axis_angle(&axis.axis(), angle);
    self
  }

  pub(crate) fn rotate_position(&mut self, angle: f32) -> &mut Self {
    self.filter.position = Mat3f::new_rotation(-angle);
    self
  }

  pub(crate) fn samples(&mut self) -> &mut Self {
    self.filter.field = Field::Samples;
    self
  }

  pub(crate) fn scale(&mut self, n: f32) -> &mut Self {
    self.filter.position *= Mat3f::new_scaling(n);
    self
  }

  pub(crate) fn spread(&mut self, spread: bool) -> &mut Self {
    self.spread = spread;
    self
  }

  #[cfg(test)]
  pub(crate) fn square(&mut self) -> &mut Self {
    self.filter.field = Field::Square;
    self
  }

  pub(crate) fn text(&mut self, text: Option<Text>) -> &mut Self {
    self.text = text;
    self
  }

  pub(crate) fn tick(&mut self, elapsed: Duration) {
    let elapsed = elapsed.as_secs_f32();
    self.position.x -= self.velocity.x * 4.0 * elapsed;
    self.position.y -= self.velocity.y * 4.0 * elapsed;
    self.position.z -= self.velocity.z * elapsed;
    self.position.w -= self.velocity.w * elapsed;
    let mut callback = self.callback.take();
    if let Some(callback) = &mut callback {
      callback(self, elapsed);
    }
    self.callback = callback;
  }

  #[cfg(test)]
  pub(crate) fn tile(&mut self, tile: bool) -> &mut Self {
    self.tile = tile;
    self
  }

  pub(crate) fn times(&mut self, n: usize) -> &mut Self {
    for _ in 0..n {
      self.push();
    }
    self
  }

  pub(crate) fn top(&mut self) -> &mut Self {
    self.filter.field = Field::Top;
    self
  }

  pub(crate) fn transform(&mut self, rotation: f32, scaling: f32) -> &mut Self {
    self.position(Mat3f::new_rotation(-rotation).prepend_scaling(scaling))
  }

  pub(crate) fn transient(&self) -> Filter {
    Filter {
      field: Field::All,
      position: Mat3f::new_rotation(self.position.w)
        * Mat3f::new_translation(&self.position.xy()).prepend_scaling(self.position.z),
      wrap: self.wrap,
      ..default()
    }
  }

  #[cfg(test)]
  pub(crate) fn triangle(&mut self) -> &mut Self {
    self.filter.field = Field::Triangle;
    self
  }

  pub(crate) fn truncate(&mut self, len: usize) -> &mut Self {
    self.filters.truncate(len);
    self
  }

  pub(crate) fn vz(&mut self, vz: f32) -> &mut Self {
    self.velocity.z = vz;
    self
  }

  pub(crate) fn wrap(&mut self, wrap: bool) -> &mut Self {
    self.filter.wrap = wrap;
    self
  }

  pub(crate) fn x(&mut self) -> &mut Self {
    self.filter.field = Field::X;
    self
  }

  pub(crate) fn z(&mut self, z: f32) -> &mut Self {
    self.position.z = z;
    self
  }
}
