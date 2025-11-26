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
  fn sample(&mut self) -> Option<f32> {
    let mut sum = 0.0;

    self
      .voices
      .retain_mut(|voice| voice.sample().inspect(|sample| sum += sample).is_some());

    if self.voices.is_empty() {
      return None;
    }

    Some(sum)
  }
}
