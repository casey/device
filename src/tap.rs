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
  rubato::{Fft, FixedSync},
};

pub(crate) struct Tap {
  backend: Arc<Mutex<Backend>>,
  done: f64,
  format: SoundFormat,
  last: Option<Position>,
  muted: Arc<AtomicBool>,
  paused: Arc<AtomicBool>,
  sequencer: Sequencer,
  stream: Option<Stream>,
  tempo: Option<Tempo>,
  time: f64,
}

impl Tap {
  pub(crate) const CHANNELS: u16 = 2;

  pub(crate) fn drain(&mut self) -> Sound {
    self.drain_exact(None).unwrap()
  }

  pub(crate) fn drain_exact(&mut self, count: Option<usize>) -> Option<Sound> {
    let mut backend = self.backend.lock().unwrap();

    let count = count.unwrap_or(backend.samples.len());

    if backend.samples.len() < count {
      return None;
    }

    let sound = Sound::new(self.format, backend.samples.drain(..count).collect());

    self.time += sound.duration().as_secs_f64();

    Some(sound)
  }

  pub(crate) fn format(&self) -> SoundFormat {
    self.format
  }

  pub(crate) fn is_done(&self) -> bool {
    self.time >= self.done
  }

  pub(crate) fn load_track(&self, path: &Utf8Path) -> Result<Track> {
    Ok(Track {
      tempo: Tempo::detect(path)?,
      wave: self.load_wave(path)?,
    })
  }

  pub(crate) fn load_wave(&self, path: &Utf8Path) -> Result<Arc<Wave>> {
    let mut input = Wave::load(path).context(error::WaveLoad)?;

    if input.is_empty() {
      return Ok(Arc::new(Wave::new(1, self.format.sample_rate as f64)));
    }

    while input.channels() > Self::CHANNELS.into_usize() {
      input.remove_channel(input.channels() - 1);
    }

    let sample_rate = input.sample_rate();

    if sample_rate.fract() != 0.0 {
      return Err(error::WaveSampleRate { sample_rate }.build());
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let sample_rate = sample_rate as usize;

    if sample_rate == self.format.sample_rate.into_usize() {
      return Ok(Arc::new(input));
    }

    let start = Instant::now();

    let mut resampler = Fft::<f32>::new(
      sample_rate,
      self.format.sample_rate.into_usize(),
      1024,
      2,
      input.channels(),
      FixedSync::Both,
    )
    .context(error::WaveResamplerConstruction)?;

    let needed = resampler.process_all_needed_output_len(input.len());

    let mut output = Wave::new(input.channels(), self.format.sample_rate as f64);
    output.resize(needed);

    let input = WaveAdapter(input);
    let mut output = WaveAdapter(output);

    resampler
      .process_all_into_buffer(&input, &mut output, input.0.len(), None)
      .context(error::WaveResample)?;

    log::info!("resampled {path} in {:.2}s", start.elapsed().as_secs_f32());

    Ok(Arc::new(output.into_inner()))
  }

  pub(crate) fn new(options: &Options, sample_rate: u32) -> Self {
    let mut sequencer = Sequencer::new(false, Self::CHANNELS.into());
    sequencer.set_sample_rate(sample_rate.into());
    let sequencer_backend = sequencer.backend();
    let paused = Arc::new(AtomicBool::new(false));
    let muted = Arc::new(AtomicBool::new(options.mute));
    Self {
      backend: Arc::new(Mutex::new(Backend {
        buffer: BufferVec::new(Self::CHANNELS.into()),
        muted: muted.clone(),
        paused: paused.clone(),
        sample: 0,
        samples: Vec::new(),
        sequencer_backend,
      })),
      done: 0.0,
      format: SoundFormat {
        channels: Self::CHANNELS,
        sample_rate,
      },
      last: None,
      muted,
      paused,
      sequencer,
      stream: None,
      tempo: None,
      time: 0.0,
    }
  }

  pub(crate) fn pause(&self) {
    self.paused.store(true, atomic::Ordering::Relaxed);
  }

  pub(crate) fn play(&self) {
    self.paused.store(false, atomic::Ordering::Relaxed);
  }

  fn position(&self) -> Option<Position> {
    let tempo = self.tempo?;

    if self.time < tempo.offset {
      return None;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let quarter = (tempo.beats(self.time) * 4.0) as u64;

    Some(Position::from_quarter(quarter))
  }

  pub(crate) fn sequence<T>(&mut self, node: An<T>, duration: f64, fade_in: f64, fade_out: f64)
  where
    T: AudioNode<Inputs = U0> + IntoStereo<T::Outputs> + 'static,
  {
    self.done = self.done.max(self.time + duration);
    self.sequencer.push_relative(
      0.0,
      duration,
      Fade::default(),
      fade_in,
      fade_out,
      node.0.into_stereo(),
    );
  }

  pub(crate) fn sequence_track(&mut self, track: &Track, fade_in: f64, fade_out: f64) {
    self.sequence_wave(&track.wave, fade_in, fade_out);
    self.tempo = Some(Tempo {
      bpm: track.tempo.bpm,
      offset: self.time + track.tempo.offset,
    });
  }

  pub(crate) fn sequence_wave(&mut self, wave: &Arc<Wave>, fade_in: f64, fade_out: f64) {
    if wave.channels() == 0 {
      return;
    }

    let duration = wave.duration();

    if wave.channels() == 1 {
      let mono = WavePlayer::new(wave, 0, 0, wave.len(), None);
      self.sequence(An(mono), duration, fade_in, fade_out);
    } else {
      let left = WavePlayer::new(wave, 0, 0, wave.len(), None);
      let right = WavePlayer::new(wave, 1, 0, wave.len(), None);
      self.sequence(An(left) | An(right), duration, fade_in, fade_out);
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

  pub(crate) fn tick(&mut self, dt: Duration) -> Tick {
    let last = self.last;
    let position = self.position();
    self.last = position;
    Tick {
      dt,
      last,
      position,
      tempo: self.tempo,
      time: self.time,
    }
  }

  pub(crate) fn toggle_muted(&self) {
    self.muted.fetch_xor(true, atomic::Ordering::Relaxed);
  }

  pub(crate) fn toggle_paused(&self) {
    self.paused.fetch_xor(true, atomic::Ordering::Relaxed);
  }

  pub(crate) fn write(&self, buffer: &mut [f32]) {
    self.backend.lock().unwrap().write(buffer);
  }
}

struct Backend {
  buffer: BufferVec,
  muted: Arc<AtomicBool>,
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

    let muted = self.muted.load(atomic::Ordering::Relaxed);

    for slot in buffer {
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

      let sample = self.buffer.at_f32(
        self.sample.into_usize() % Tap::CHANNELS.into_usize(),
        (self.sample.into_usize() / Tap::CHANNELS.into_usize()) % MAX_BUFFER_SIZE,
      );

      *slot = if muted { 0.0 } else { sample };

      self.samples.push(sample);

      self.sample += 1;
    }
  }
}
