use super::*;

pub(crate) struct Emitter<T> {
  voice: T,
}

impl<T> Emitter<T> {
  pub(crate) fn new(voice: T) -> Self {
    Self { voice }
  }
}

impl<T: Voice> Iterator for Emitter<T> {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    self.voice.sample()
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
    DEFAULT_SAMPLE_RATE
  }

  fn total_duration(&self) -> Option<Duration> {
    None
  }
}
