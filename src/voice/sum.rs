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
  fn sample(&mut self, t: f32) -> f32 {
    self.voices.iter_mut().map(|voice| voice.sample(t)).sum()
  }
}
