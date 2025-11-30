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
  rubato::FftFixedIn,
};

pub(crate) struct Tap {
  backend: Arc<Mutex<Backend>>,
  done: f64,
  paused: Arc<AtomicBool>,
  sample_rate: u32,
  sequencer: Sequencer,
  stream: Option<Stream>,
  time: f64,
}

impl Tap {
  pub(crate) const CHANNELS: u16 = 2;

  pub(crate) fn drain(&mut self) -> Sound {
    let samples = self
      .backend
      .lock()
      .unwrap()
      .samples
      .drain(..)
      .collect::<Vec<f32>>();
    self.time += (samples.len() / Self::CHANNELS.into_usize()) as f64 / self.sample_rate as f64;
    Sound {
      channels: Self::CHANNELS,
      sample_rate: self.sample_rate,
      samples,
    }
  }

  pub(crate) fn is_done(&self) -> bool {
    self.time >= self.done
  }

  pub(crate) fn load_wave(&self, path: &Utf8Path) -> Result<Arc<Wave>> {
    let mut input = Wave::load(path).context(error::WaveLoad)?;

    if input.is_empty() {
      return Ok(Arc::new(Wave::new(1, self.sample_rate as f64)));
    }

    for channel in Self::CHANNELS.into_usize()..input.channels() {
      input.remove_channel(channel);
    }

    let sample_rate = input.sample_rate();

    if sample_rate.fract() != 0.0 {
      return Err(error::WaveSampleRate { sample_rate }.build());
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let sample_rate = sample_rate as usize;

    let start = Instant::now();

    let mut resampler = FftFixedIn::<f32>::new(
      sample_rate,
      self.sample_rate.into_usize(),
      1024,
      2,
      input.channels(),
    )
    .context(error::WaveResamplerConstruction)?;

    let input_channels = (0..input.channels())
      .map(|channel| input.channel(channel).as_slice())
      .collect::<Vec<&[f32]>>();

    let output_channels = resampler
      .process_all(
        &input_channels,
        None,
        self.sample_rate as f64 / sample_rate as f64,
      )
      .context(error::WaveResample)?;

    log::info!("resampled {path} in {:.2}", start.elapsed().as_secs_f64());

    let mut output = Wave::new(0, self.sample_rate as f64);

    for channel in output_channels {
      output.push_channel(&channel);
    }

    Ok(Arc::new(output))
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
      stream: None,
      time: 0.0,
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

  pub(crate) fn stream(
    &mut self,
    output_device: &cpal::Device,
    stream_config: &StreamConfig,
  ) -> Result {
    let backend = self.backend.clone();

    self.stream = None;

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

    self.stream = Some(stream);

    log::info!(
      "output stream opened: {}",
      StreamConfigDisplay(stream_config),
    );

    Ok(())
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
