use super::*;

pub(crate) trait Stream: Iterator<Item = f32> {
  fn append(&self, sink: &Sink);

  fn channels(&self) -> u16;

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

  fn is_done(&self) -> bool;

  fn sample_rate(&self) -> u32;
}
