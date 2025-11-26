use super::*;

pub(crate) struct Sum {
  voices: Vec<Box<dyn Voice>>,
}

impl Sum {
  pub(crate) fn add<T: Voice + Sized + 'static>(mut self, voice: T) -> Self {
    self.voices.push(Box::new(voice));
    self
  }

  pub(crate) fn new() -> Self {
    Self { voices: Vec::new() }
  }
}

impl Voice for Sum {
  fn reset(&mut self) {
    for voice in &mut self.voices {
      voice.reset();
    }
  }

  fn sample(&mut self) -> Option<f32> {
    let mut sum = None;
    for voice in &mut self.voices {
      if let Some(sample) = voice.sample() {
        sum = Some(sum.unwrap_or_default() + sample);
      }
    }
    sum
  }
}
