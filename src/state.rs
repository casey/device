use super::*;

#[derive(Clone)]
pub(crate) struct State {
  pub(crate) alpha: f32,
  pub(crate) beat: u64,
  pub(crate) callback: Option<Box<dyn Callback>>,
  pub(crate) complexity: f32,
  pub(crate) db: f32,
  pub(crate) encoder: f32,
  pub(crate) filter: Filter,
  pub(crate) filters: Vec<Filter>,
  pub(crate) fit: bool,
  pub(crate) interpolate: bool,
  pub(crate) rng: SmallRng,
  pub(crate) spread: bool,
  pub(crate) status: bool,
  pub(crate) text: Option<Text>,
  pub(crate) tile: bool,
  pub(crate) transient: Transformation2,
  pub(crate) velocity: Vec4f,
  pub(crate) wrap: bool,
}

impl Default for State {
  fn default() -> Self {
    Self {
      alpha: 0.5,
      beat: 0,
      callback: None,
      db: 0.0,
      filter: Filter::default(),
      filters: Vec::new(),
      fit: false,
      interpolate: false,
      encoder: 0.0,
      rng: SmallRng::from_rng(&mut rand::rng()),
      spread: false,
      status: false,
      text: None,
      tile: false,
      transient: Transformation2::default(),
      velocity: Vec4f::zeros(),
      complexity: 0.0,
      wrap: true,
    }
  }
}

impl State {
  pub(crate) fn all(&mut self) -> &mut Self {
    self.filter.field = Field::All;
    self
  }

  pub(crate) fn alpha(&mut self, alpha: f32) -> &mut Self {
    self.filter.alpha = alpha;
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

  pub(crate) fn callback(
    &mut self,
    callback: impl FnMut(&mut State, Duration) + Clone + 'static,
  ) -> &mut Self {
    self.callback = Some(Box::new(callback));
    self
  }

  pub(crate) fn circle(&mut self) -> &mut Self {
    self.filter.field = Field::Circle { size: None };
    self
  }

  #[cfg(false)]
  pub(crate) fn clear(&mut self) -> &mut Self {
    self.filters.clear();
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
    self.filter.color = color::invert();
    self
  }

  pub(crate) fn invert_r(&mut self) -> &mut Self {
    self.filter.color = Axis::Red.invert();
    self
  }

  #[cfg(test)]
  pub(crate) fn left(&mut self) -> &mut Self {
    self.filter.field = Field::Left;
    self
  }

  #[cfg(test)]
  pub(crate) fn mirror_x(&mut self, mirror: Mirror) -> &mut Self {
    self.filter.mirror.x = mirror;
    self
  }

  #[cfg(test)]
  pub(crate) fn mirror_y(&mut self, mirror: Mirror) -> &mut Self {
    self.filter.mirror.y = mirror;
    self
  }

  pub(crate) fn none(&mut self) -> &mut Self {
    self.filter.field = Field::None;
    self
  }

  pub(crate) fn pop(&mut self) -> &mut Self {
    self.filters.pop();
    self
  }

  pub(crate) fn position(&mut self, position: Mat3f) -> &mut Self {
    self.filter.position = position;
    self
  }

  pub(crate) fn preset_limit(&self) -> usize {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let preset_limit = ((self.complexity + 1.0) * Preset::LIMIT as f32) as usize;
    preset_limit
  }

  pub(crate) fn push(&mut self) -> &mut Self {
    self.filters.push(self.filter.clone());
    self
  }

  pub(crate) fn repeat(&mut self, repeat: bool) -> &mut Self {
    self.filter.repeat = repeat;
    self
  }

  #[cfg(test)]
  pub(crate) fn rms(&mut self, rms: Mat1x2f) -> &mut Self {
    self.filter.rms = rms;
    self
  }

  pub(crate) fn rotate_color(&mut self, axis: Axis, angle: f32) -> &mut Self {
    self.filter.color = axis.rotate(angle);
    self
  }

  pub(crate) fn rotate_position(&mut self, angle: f32) -> &mut Self {
    self.filter.position = Mat3f::new_rotation(-angle);
    self
  }

  #[cfg(test)]
  pub(crate) fn rotation(&mut self, rotation: f32) -> &mut Self {
    self.filter.position_response.rotation = rotation;
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

  pub(crate) fn scaling(&mut self, z: f32) -> &mut Self {
    self.transient.scaling = Vec2f::new(z, z);
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

  pub(crate) fn tick(&mut self, dt: Duration) {
    {
      let dt = dt.as_secs_f32();
      let ds = self.velocity.z * dt;
      self.transient.translation -= self.velocity.xy() * dt;
      self.transient.scaling -= Vec2f::new(ds, ds);
      self.transient.rotation -= self.velocity.w * dt;
    }

    let mut callback = self.callback.take();
    if let Some(callback) = &mut callback {
      callback(self, dt);
    }
    self.callback = callback;
    for filter in &mut self.filters {
      filter.tick(dt);
    }
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

  pub(crate) fn transient(&self) -> Option<Filter> {
    if self.transient == Transformation2::default() {
      None
    } else {
      Some(Filter {
        position: self.transient.response(1.0),
        wrap: self.wrap,
        ..default()
      })
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
}
