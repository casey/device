use super::*;

const CHANNELS: u16 = 2;
const SAMPLE_RATE: u32 = 48_000;

#[derive(Clone)]
pub(crate) struct Synthesizer(Arc<Mutex<Inner>>);

#[derive(Default)]
struct Inner {
  buffer: Vec<f32>,
  drained: usize,
  sample: usize,
}

impl Synthesizer {
  pub(crate) fn new() -> Self {
    Self(Arc::new(Mutex::new(Inner::default())))
  }
}

impl Source for Synthesizer {
  fn channels(&self) -> u16 {
    CHANNELS
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    SAMPLE_RATE
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    None
  }
}

impl Stream for Synthesizer {
  fn channels(&self) -> u16 {
    CHANNELS
  }

  fn done(&self) -> bool {
    false
  }

  fn drain(&mut self, samples: &mut Vec<f32>) {
    let mut inner = self.0.lock().unwrap();
    samples.extend(&inner.buffer[inner.drained..]);
    inner.drained = inner.buffer.len();
  }

  fn sample_rate(&self) -> u32 {
    SAMPLE_RATE
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

    let i = inner.buffer.len().into_u64() / u64::from(CHANNELS);

    let t = i as f32 / SAMPLE_RATE as f32;

    let sample = if t.fract() < 0.5 {
      let a = (t * 480.0 * f32::consts::TAU).sin();
      let b = (t * 620.0 * f32::consts::TAU).sin();
      a + b
    } else {
      0.0
    };

    for _ in 0..CHANNELS {
      inner.buffer.push(sample);
    }

    inner.sample += 1;

    Some(sample)
  }
}
