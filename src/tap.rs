use super::*;

trait Foo {
  fn into_source(self, sample_rate: u32, channels: u16) -> Box<dyn Source + Send>;
}

impl<T: fundsp::hacker32::AudioNode<Inputs = fundsp::hacker32::U0, Outputs = fundsp::hacker32::U1>>
  Foo for fundsp::hacker32::An<T>
{
  fn into_source(mut self, sample_rate: u32, channels: u16) -> Box<dyn Source + Send> {
    self.set_sample_rate(sample_rate as f64);
    let x = self >> fundsp::hacker32::split::<fundsp::hacker32::U2>();
    Wrapper(x)
  }
}

struct Wrapper<T>(T);

impl<T: fundsp::hacker32::AudioNode<Inputs = fundsp::hacker32::U0, Outputs = fundsp::hacker32::U2>
impl<T> Source for Wrapper {
}

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
  fn foo<T: Foo + 'static>(&self, mut foo: T) {
    let mut inner = self.0.lock().unwrap();
    let source = foo.into_source(inner.sample_rate, inner.channels);
    inner.pending.push(Box::new(source));
  }

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
