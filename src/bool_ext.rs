pub(crate) trait BoolExt {
  fn into_f32(self) -> f32;

  fn toggle(&mut self);
}

impl BoolExt for bool {
  fn into_f32(self) -> f32 {
    u8::from(self) as f32
  }

  fn toggle(&mut self) {
    *self = !*self;
  }
}
