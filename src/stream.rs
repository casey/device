use super::*;

pub(crate) trait Stream {
  fn channels(&self) -> u16;

  fn is_done(&self) -> bool;

  fn drain(&mut self) -> Sound {
    let mut samples = Vec::new();
    self.drain_samples(&mut samples);
    Sound {
      channels: self.channels(),
      sample_rate: self.sample_rate(),
      samples,
    }
  }

  fn drain_samples(&mut self, samples: &mut Vec<f32>);

  fn sample_rate(&self) -> u32;
}
