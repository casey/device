use super::*;

pub(crate) struct Emitter<T> {
  sample: u64,
  voice: T,
}

impl<T> Emitter<T> {
  pub(crate) fn new(voice: T) -> Self {
    Self { voice, sample: 0 }
  }
}

impl<T: Voice> Iterator for Emitter<T> {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    let sample = self
      .voice
      .sample(self.sample as f32 / self.sample_rate() as f32);
    self.sample += 1;
    Some(sample)
  }
}

impl<T: Voice> Source for Emitter<T> {
  fn channels(&self) -> u16 {
    1
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    48_000
  }

  fn total_duration(&self) -> Option<Duration> {
    None
  }
}
