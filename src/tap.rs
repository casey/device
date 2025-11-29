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
    wave::{Wave, WavePlayer},
  },
};

pub(crate) trait IntoStereo<Outputs> {
  fn into_stereo(self) -> Box<dyn AudioUnit>;
}

impl<T> IntoStereo<U1> for T
where
  T: AudioNode<Inputs = U0, Outputs = U1> + 'static,
{
  fn into_stereo(self) -> Box<dyn AudioUnit> {
    Box::new(An(self) >> split::<U2>())
  }
}

impl<T> IntoStereo<U2> for T
where
  T: AudioNode<Inputs = U0, Outputs = U2> + 'static,
{
  fn into_stereo(self) -> Box<dyn AudioUnit> {
    Box::new(An(self))
  }
}

#[derive(Clone)]
pub(crate) struct Tap(Arc<Mutex<Backend>>);

struct Backend {
  active: Vec<Box<dyn Source + Send>>,
  buffer: BufferVec,
  done: f64,
  pending: Vec<Box<dyn Source + Send>>,
  sample: u64,
  sample_rate: u32,
  samples: VecDeque<f32>,
  sequencer: Sequencer,
}

impl Tap {
  pub(crate) const CHANNELS: u16 = 2;

  pub(crate) fn drain(&mut self) -> Sound {
    let mut backend = self.0.lock().unwrap();
    let samples = backend.samples.len() - backend.samples.len() % Self::CHANNELS.into_usize();
    Sound {
      channels: Self::CHANNELS,
      sample_rate: backend.sample_rate,
      samples: backend.samples.drain(0..samples).collect(),
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
      samples: VecDeque::new(),
      sequencer,
    })))
  }

  pub(crate) fn sequence<T>(&self, node: An<T>, duration: f64, fade_in: f64, fade_out: f64)
  where
    T: AudioNode<Inputs = U0> + IntoStereo<T::Outputs> + 'static,
  {
    let mut backend = self.0.lock().unwrap();
    backend.done = backend.sequencer.time() + duration;

    backend.sequencer.push_relative(
      0.0,
      duration,
      Fade::default(),
      fade_in,
      fade_out,
      node.0.into_stereo(), // Box<dyn AudioUnit>
    );
  }

  pub(crate) fn sequence_indefinite<T>(&self, audio_node: An<T>)
  where
    T: AudioNode<Inputs = U0> + IntoStereo<T::Outputs> + 'static,
  {
    self.sequence(audio_node, f64::INFINITY, 0.0, 0.0);
  }

  pub(crate) fn sequence_wave(&self, wave: Wave) {
    // todo: set correct time
    let wave = Arc::new(wave);
    let duration = wave.duration();
    if wave.channels() == 0 {
    } else if wave.channels() == 1 {
      let mono = WavePlayer::new(&wave, 0, 0, wave.len(), None);
      self.sequence(An(mono), duration, 0.0, 0.0);
    } else {
      let left = WavePlayer::new(&wave, 0, 0, wave.len(), None);
      let right = WavePlayer::new(&wave, 1, 0, wave.len(), None);
      self.sequence(An(left) | An(right), duration, 0.0, 0.0);
    }
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

    self.samples.push_back(sum);

    self.sample += 1;

    Some(sum)
  }
}
