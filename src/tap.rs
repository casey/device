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
  rubato::{FftFixedIn, Resampler},
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
  buffer: BufferVec,
  done: f64,
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
    backend.sequencer.time() >= backend.done
  }

  // todo:
  // - deal with delay
  // - deal with there still being chunks in the resampler
  pub(crate) fn load_wave(&self, path: &Utf8Path) -> Result<Wave> {
    const CHUNK: usize = 1024;
    let sample_rate = self.0.lock().unwrap().sample_rate;

    let mut wave = Wave::load(path).context(error::WaveLoad)?;

    for channel in Self::CHANNELS.into_usize()..wave.channels() {
      wave.remove_channel(channel);
    }

    dbg!(wave.channels());
    dbg!(wave.sample_rate());

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let mut resampler = FftFixedIn::<f32>::new(
      wave.sample_rate() as usize,
      sample_rate.into_usize(),
      CHUNK,
      2,
      wave.channels(),
    )
    .context(error::WaveResamplerConstruction)?;

    let mut output_buffer = resampler.output_buffer_allocate(true);
    let mut input_buffer = resampler.input_buffer_allocate(true);

    let mut output_channels = vec![Vec::<f32>::new(); wave.channels()];

    for chunk in 0.. {
      let start = chunk * CHUNK;
      let end = start + CHUNK;
      let remaining = wave.len() - start;

      if remaining == 0 {
        break;
      } else if remaining < CHUNK {
        let samples = wave.len() - start;

        for (channel, buffer) in input_buffer.iter_mut().enumerate() {
          buffer[0..samples].copy_from_slice(&wave.channel(channel)[start..start + samples]);
          buffer.truncate(samples);
        }

        let (_input, output) = resampler
          .process_partial_into_buffer(Some(&input_buffer), &mut output_buffer, None)
          .context(error::WaveResample)?;

        for channel in 0..wave.channels() {
          output_channels[channel].extend(&output_buffer[channel][0..output]);
        }

        break;
      }

      for (channel, buffer) in input_buffer.iter_mut().enumerate() {
        buffer[0..CHUNK].copy_from_slice(&wave.channel(channel)[start..end]);
      }

      let (input, output) = resampler
        .process_into_buffer(&input_buffer, &mut output_buffer, None)
        .context(error::WaveResample)?;

      assert_eq!(input, CHUNK);

      for channel in 0..wave.channels() {
        output_channels[channel].extend(&output_buffer[channel][0..output]);
      }
    }

    let mut output_wave = Wave::new(0, sample_rate as f64);

    for channel in output_channels {
      output_wave.push_channel(&channel);
    }

    Ok(output_wave)
  }

  pub(crate) fn new(sample_rate: u32) -> Self {
    let mut sequencer = Sequencer::new(false, Self::CHANNELS.into());
    sequencer.set_sample_rate(sample_rate.into());
    Self(Arc::new(Mutex::new(Backend {
      buffer: BufferVec::new(2),
      done: 0.0,
      sample: 0,
      sample_rate,
      samples: VecDeque::new(),
      sequencer,
    })))
  }

  pub(crate) fn sample_rate(&self) -> u32 {
    self.0.lock().unwrap().sample_rate
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

  pub(crate) fn write(&self, buffer: &mut [f32]) {
    self.0.lock().unwrap().write(buffer);
  }
}

impl Backend {
  fn write(&mut self, buffer: &mut [f32]) {
    for x in buffer {
      if self
        .sample
        .is_multiple_of(MAX_BUFFER_SIZE.into_u64() * u64::from(Tap::CHANNELS))
      {
        self.sequencer.process(
          MAX_BUFFER_SIZE,
          &BufferRef::empty(),
          &mut self.buffer.buffer_mut(),
        );
      }

      let channel = self.sample.into_usize() % Tap::CHANNELS.into_usize();
      let sample = (self.sample.into_usize() / Tap::CHANNELS.into_usize()) % MAX_BUFFER_SIZE;

      *x = self.buffer.at_f32(channel, sample);

      self.samples.push_back(*x);

      self.sample += 1;
    }
  }
}
