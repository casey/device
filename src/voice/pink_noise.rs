use super::*;

const ROWS: usize = 16;

pub(crate) struct PinkNoise {
  counter: u32,
  distribution: Uniform<f32>,
  rng: SmallRng,
  rows: [f32; ROWS],
}

impl PinkNoise {
  pub(crate) fn new() -> Self {
    let distribution = distribution();
    let mut rng = SmallRng::from_rng(&mut rand::rng());
    let rows = array::from_fn(|_| rng.sample(distribution));
    PinkNoise {
      distribution,
      rng,
      rows,
      counter: 0,
    }
  }
}

impl Voice for PinkNoise {
  fn sample(&mut self, _t: f32) -> f32 {
    self.counter = self.counter.wrapping_add(1);

    let mut c = self.counter;
    let mut i = 0;

    while c & 1 == 0 && i < ROWS {
      self.rows[i] = self.rng.sample(self.distribution);
      c >>= 1;
      i += 1;
    }

    self.rows.iter().copied().sum::<f32>() / ROWS as f32
  }
}
