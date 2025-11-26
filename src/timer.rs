#[derive(Default)]
pub(crate) struct Timer {
  time: f32,
}

impl Timer {
  pub(crate) fn reset(&mut self) {
    self.time = 0.0;
  }

  pub(crate) fn tick(&mut self) -> f32 {
    let next = self.time;
    self.time += 1.0 / 48_000.0;
    next
  }
}
