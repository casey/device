use super::*;

#[derive(Clone)]
pub(crate) struct Synthesizer(Arc<Mutex<Inner>>);

struct Inner {
  buffer: Vec<f32>,
  drained: usize,
  sample: usize,
  voice: Box<dyn Voice>,
}

impl Synthesizer {
  const CHANNELS: u16 = 2;
  const SAMPLE_RATE: u32 = 48_000;

  pub(crate) fn new(voice: Box<dyn Voice>) -> Self {
    Self(Arc::new(Mutex::new(Inner {
      buffer: Vec::new(),
      drained: 0,
      sample: 0,
      voice,
    })))
  }
}

impl<T: Voice + Sized + 'static> From<T> for Synthesizer {
  fn from(voice: T) -> Self {
    Self::new(Box::new(voice))
  }
}

impl Source for Synthesizer {
  fn channels(&self) -> u16 {
    Self::CHANNELS
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    Self::SAMPLE_RATE
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    None
  }
}

impl Stream for Synthesizer {
  fn append(&self, sink: &Sink) {
    sink.append(self.clone());
  }

  fn channels(&self) -> u16 {
    Self::CHANNELS
  }

  fn drain_samples(&mut self, samples: &mut Vec<f32>) {
    let mut inner = self.0.lock().unwrap();
    samples.extend(&inner.buffer[inner.drained..]);
    inner.drained = inner.buffer.len();
  }

  fn is_done(&self) -> bool {
    false
  }

  fn sample_rate(&self) -> u32 {
    Self::SAMPLE_RATE
  }
}

impl Iterator for Synthesizer {
  type Item = f32;

  fn next(&mut self) -> Option<f32> {
    let mut inner = self.0.lock().unwrap();

    if let Some(&sample) = inner.buffer.get(inner.sample) {
      inner.sample += 1;
      return Some(sample);
    }

    let i = inner.buffer.len().into_u64() / u64::from(Self::CHANNELS);

    let t = i as f32 / Self::SAMPLE_RATE as f32;

    let sample = inner.voice.sample(t);

    for _ in 0..Self::CHANNELS {
      inner.buffer.push(sample);
    }

    inner.sample += 1;

    Some(sample)
  }
}
