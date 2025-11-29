use {
  super::*,
  fundsp::{
    MAX_BUFFER_SIZE,
    audionode::AudioNode,
    audiounit::AudioUnit,
    buffer::{BufferRef, BufferVec},
    combinator::An,
    prelude::{U0, U1, U2, split},
    sequencer::{Fade, Sequencer},
  },
};

#[derive(Clone)]
pub(crate) struct Tap(Arc<Mutex<Backend>>);

struct Backend {
  active: Vec<Box<dyn Source + Send>>,
  buffer: BufferVec,
  done: f64,
  pending: Vec<Box<dyn Source + Send>>,
  sample: u64,
  sample_rate: u32,
  samples: Vec<f32>,
  sequencer: Sequencer,
}

impl Tap {
  pub(crate) const CHANNELS: u16 = 2;

  pub(crate) fn sequence_wave(&self, wave: fundsp::wave::Wave) {
    // todo: set correct time
    let wave = Arc::new(wave);
    if wave.channels() == 0 {
    } else if wave.channels() == 1 {
      let mono = fundsp::wave::WavePlayer::new(&wave, 0, 0, wave.len(), None);
      self.sequence_indefinite(An(mono) >> split::<U2>());
    } else {
      let l = fundsp::wave::WavePlayer::new(&wave, 0, 0, wave.len(), None);
      let r = fundsp::wave::WavePlayer::new(&wave, 1, 0, wave.len(), None);
      self.sequence_indefinite(An(l) | An(r));
    }
  }

  pub(crate) fn add<T: Source + Send + 'static>(&self, source: T) {
    let mut backend = self.0.lock().unwrap();
    let sample_rate = backend.sample_rate;
    backend.pending.push(Box::new(UniformSourceIterator::new(
      source,
      Self::CHANNELS,
      sample_rate,
    )));
  }

  pub(crate) fn drain(&mut self) -> Sound {
    let mut backend = self.0.lock().unwrap();
    Sound {
      channels: Self::CHANNELS,
      sample_rate: backend.sample_rate,
      samples: mem::take(&mut backend.samples),
    }
  }

  pub(crate) fn is_done(&self) -> bool {
    let backend = self.0.lock().unwrap();
    backend.active.is_empty()
      && backend.pending.is_empty()
      && backend.sequencer.time() >= backend.done
  }

  pub(crate) fn new(sample_rate: u32) -> Self {
    let mut sequencer = Sequencer::new(false, Self::CHANNELS.into());
    sequencer.set_sample_rate(sample_rate.into());
    Self(Arc::new(Mutex::new(Backend {
      active: Vec::new(),
      buffer: BufferVec::new(2),
      done: 0.0,
      pending: Vec::new(),
      sample: 0,
      sample_rate,
      samples: Vec::new(),
      sequencer,
    })))
  }

  pub(crate) fn sequence<T: AudioNode<Inputs = U0, Outputs = U2> + 'static>(
    &self,
    audio_node: An<T>,
    duration: f64,
    fade_in: f64,
    fade_out: f64,
  ) {
    let mut backend = self.0.lock().unwrap();
    backend.done = backend.sequencer.time() + duration;
    backend.sequencer.push_relative(
      0.0,
      duration,
      Fade::default(),
      fade_in,
      fade_out,
      Box::new(audio_node),
    );
  }

  pub(crate) fn sequence_indefinite<T: AudioNode<Inputs = U0, Outputs = U2> + 'static>(
    &self,
    audio_node: An<T>,
  ) {
    self.sequence(audio_node, f64::INFINITY, 0.0, 0.0);
  }
}

impl Source for Tap {
  fn channels(&self) -> u16 {
    Self::CHANNELS
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

impl Iterator for Backend {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.sample.is_multiple_of(Tap::CHANNELS.into()) {
      self.active.append(&mut self.pending);
    }

    if self.sample.is_multiple_of(MAX_BUFFER_SIZE.into_u64() * 2) {
      self.sequencer.process(
        MAX_BUFFER_SIZE,
        &BufferRef::empty(),
        &mut self.buffer.buffer_mut(),
      );
    }

    let channel = self.sample.into_usize() % Tap::CHANNELS.into_usize();
    let sample = (self.sample.into_usize() / Tap::CHANNELS.into_usize()) % MAX_BUFFER_SIZE;

    let mut sum = self.buffer.at_f32(channel, sample);

    self
      .active
      .retain_mut(|source| source.next().inspect(|sample| sum += sample).is_some());

    self.samples.push(sum);

    self.sample += 1;

    Some(sum)
  }
}
