use super::*;

#[derive(Clone)]
pub(crate) struct Tap(Arc<Mutex<Inner>>);

struct Inner {
  active: Vec<Box<dyn Source + Send>>,
  channels: u16,
  pending: Vec<Box<dyn Source + Send>>,
  sample: u64,
  sample_rate: u32,
  samples: Vec<f32>,
}

impl Tap {
  pub(crate) fn add<T: Source + Send + 'static>(&self, source: T) {
    let mut inner = self.0.lock().unwrap();
    let channels = inner.channels;
    let sample_rate = inner.sample_rate;
    inner.pending.push(Box::new(UniformSourceIterator::new(
      source,
      channels,
      sample_rate,
    )));
  }

  pub(crate) fn drain(&mut self) -> Sound {
    let mut inner = self.0.lock().unwrap();
    Sound {
      channels: inner.channels,
      sample_rate: inner.sample_rate,
      samples: mem::take(&mut inner.samples),
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    let inner = self.0.lock().unwrap();
    inner.active.is_empty() && inner.pending.is_empty()
  }

  pub(crate) fn new(channels: u16, sample_rate: u32) -> Self {
    Self(Arc::new(Mutex::new(Inner {
      active: Vec::new(),
      channels,
      pending: Vec::new(),
      sample: 0,
      sample_rate,
      samples: Vec::new(),
    })))
  }
}

impl Source for Tap {
  fn channels(&self) -> u16 {
    self.0.lock().unwrap().channels
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    self.0.lock().unwrap().sample_rate
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    None
  }
}

impl Iterator for Tap {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.lock().unwrap().next()
  }
}

impl Iterator for Inner {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.sample.is_multiple_of(self.channels.into()) {
      self.active.append(&mut self.pending);
    }

    let mut sum = 0.0;

    eprintln!("sampling {} voices", self.active.len());

    self
      .active
      .retain_mut(|source| source.next().inspect(|sample| sum += sample).is_some());

    self.samples.push(sum);

    self.sample += 1;

    Some(sum)
  }
}
