use super::*;

const ROWS: usize = 16;

pub(crate) struct PinkNoise {
  counter: u32,
  inner: WhiteNoise,
  rows: [f32; ROWS],
}

impl PinkNoise {
  pub(crate) fn new() -> Self {
    let mut inner = WhiteNoise::new();
    let rows = array::from_fn(|_| inner.sample().unwrap());
    PinkNoise {
      counter: 0,
      inner,
      rows,
    }
  }
}

impl Voice for PinkNoise {
  fn reset(&mut self) {
    self.inner.reset();
    self.rows = array::from_fn(|_| self.inner.sample().unwrap());
  }

  fn sample(&mut self) -> Option<f32> {
    self.counter = self.counter.wrapping_add(1);

    let mut c = self.counter;
    let mut i = 0;

    while c & 1 == 0 && i < ROWS {
      self.rows[i] = self.inner.sample()?;
      c >>= 1;
      i += 1;
    }

    Some(self.rows.iter().copied().sum::<f32>() / ROWS as f32)
  }
}
