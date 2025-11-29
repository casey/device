use {
  super::*,
  fundsp::{
    MAX_BUFFER_SIZE,
    audionode::AudioNode,
    audiounit::AudioUnit,
    buffer::{BufferRef, BufferVec},
    combinator::An,
    prelude::U0,
    realseq::SequencerBackend,
    sequencer::{Fade, Sequencer},
    wave::{Wave, WavePlayer},
  },
  rubato::{FftFixedIn, Resampler},
};

pub(crate) struct Tap {
  backend: Arc<Mutex<Backend>>,
  done: f64,
  paused: Arc<AtomicBool>,
  sample_rate: u32,
  sequencer: Sequencer,
}

impl Tap {
  pub(crate) const CHANNELS: u16 = 2;

  pub(crate) fn build_output_stream(
    &self,
    output_device: &cpal::Device,
    stream_config: &StreamConfig,
  ) -> Result<Stream> {
    let backend = self.backend.clone();

    let stream = output_device
      .build_output_stream(
        stream_config,
        move |data: &mut [f32], _info| {
          backend.lock().unwrap().write(data);
        },
        |err| eprintln!("output stream error: {err}"),
        None,
      )
      .context(error::AudioBuildOutputStream)?;

    log::info!(
      "output stream opened: {}",
      StreamConfigDisplay(stream_config),
    );

    Ok(stream)
  }

  pub(crate) fn drain(&self) -> Sound {
    Sound {
      channels: Self::CHANNELS,
      sample_rate: self.sample_rate,
      samples: self.backend.lock().unwrap().samples.drain(..).collect(),
    }
  }

  pub(crate) fn is_done(&self) -> bool {
    self.sequencer.time() >= self.done
  }

  pub(crate) fn load_wave(&self, path: &Utf8Path) -> Result<Arc<Wave>> {
    const CHUNK: usize = 1024;

    let mut wave = Wave::load(path).context(error::WaveLoad)?;

    for channel in Self::CHANNELS.into_usize()..wave.channels() {
      wave.remove_channel(channel);
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let mut resampler = FftFixedIn::<f32>::new(
      wave.sample_rate() as usize,
      self.sample_rate.into_usize(),
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

    let mut output_wave = Wave::new(0, self.sample_rate as f64);

    for channel in output_channels {
      output_wave.push_channel(&channel);
    }

    Ok(Arc::new(output_wave))
  }

  pub(crate) fn new(sample_rate: u32) -> Self {
    let mut sequencer = Sequencer::new(false, Self::CHANNELS.into());
    sequencer.set_sample_rate(sample_rate.into());
    let sequencer_backend = sequencer.backend();
    let paused = Arc::new(AtomicBool::new(false));
    Self {
      backend: Arc::new(Mutex::new(Backend {
        buffer: BufferVec::new(Self::CHANNELS.into()),
        paused: paused.clone(),
        sample: 0,
        samples: Vec::new(),
        sequencer_backend,
      })),
      done: 0.0,
      paused,
      sample_rate,
      sequencer,
    }
  }

  pub(crate) fn pause(&self) {
    self.paused.store(true, atomic::Ordering::Relaxed);
  }

  pub(crate) fn play(&self) {
    self.paused.store(false, atomic::Ordering::Relaxed);
  }

  pub(crate) fn sample_rate(&self) -> u32 {
    self.sample_rate
  }

  pub(crate) fn sequence<T>(&mut self, node: An<T>, duration: f64, fade_in: f64, fade_out: f64)
  where
    T: AudioNode<Inputs = U0> + IntoStereo<T::Outputs> + 'static,
  {
    self.done = self.sequencer.time() + duration;
    self.sequencer.push_relative(
      0.0,
      duration,
      Fade::default(),
      fade_in,
      fade_out,
      node.0.into_stereo(),
    );
  }

  pub(crate) fn sequence_wave(&mut self, wave: &Arc<Wave>) {
    if wave.channels() == 0 {
      return;
    }

    let duration = wave.duration();

    if wave.channels() == 1 {
      let mono = WavePlayer::new(wave, 0, 0, wave.len(), None);
      self.sequence(An(mono), duration, 0.0, 0.0);
    } else {
      let left = WavePlayer::new(wave, 0, 0, wave.len(), None);
      let right = WavePlayer::new(wave, 1, 0, wave.len(), None);
      self.sequence(An(left) | An(right), duration, 0.0, 0.0);
    }
  }

  pub(crate) fn write(&self, buffer: &mut [f32]) {
    self.backend.lock().unwrap().write(buffer);
  }
}

struct Backend {
  buffer: BufferVec,
  paused: Arc<AtomicBool>,
  sample: u64,
  samples: Vec<f32>,
  sequencer_backend: SequencerBackend,
}

impl Backend {
  fn write(&mut self, buffer: &mut [f32]) {
    if self.paused.load(atomic::Ordering::Relaxed) {
      buffer.fill(0.0);
      return;
    }

    for sample in buffer {
      if self
        .sample
        .is_multiple_of(MAX_BUFFER_SIZE.into_u64() * u64::from(Tap::CHANNELS))
      {
        self.sequencer_backend.process(
          MAX_BUFFER_SIZE,
          &BufferRef::empty(),
          &mut self.buffer.buffer_mut(),
        );
      }

      *sample = self.buffer.at_f32(
        self.sample.into_usize() % Tap::CHANNELS.into_usize(),
        (self.sample.into_usize() / Tap::CHANNELS.into_usize()) % MAX_BUFFER_SIZE,
      );

      self.samples.push(*sample);

      self.sample += 1;
    }
  }
}
