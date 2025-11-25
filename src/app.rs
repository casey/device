use super::*;

// todo:
// - return None from emitter so it can be dropped or pruned
// - figure out which notes each key should be

// - not getting output
//   - store Option<Synthesizer> when output is a synth
//   - instead of adding to mixer, add to synth
//   - can only play output when output is synthesizer, no playing over input or songs
//   - adding to synth makes the most sense, it already holds a voice
//   - can instead hold a vec voices
//   - voices/synth/patches don't actually have sample rates, so can mix into anything
//   - alternative is some kind of mixer for streams
//   - mixer for streams would let me do more complex dj stuff
//     - add song
//     - remove song
//     - play sound
//   - probably more complicated though

pub(crate) struct App {
  analyzer: Analyzer,
  capture_rx: mpsc::Receiver<Result>,
  capture_tx: mpsc::Sender<Result>,
  captures_pending: u64,
  command: Option<Vec<String>>,
  config: Config,
  deadline: Instant,
  errors: Vec<Error>,
  hub: Hub,
  last: Instant,
  macro_recording: Option<Vec<(Key, bool)>>,
  makro: Vec<(Key, bool)>,
  options: Options,
  output_stream: OutputStream,
  patch: Patch,
  play: bool,
  recorder: Option<Arc<Mutex<Recorder>>>,
  renderer: Option<Renderer>,
  sink: Sink,
  state: State,
  stream: Box<dyn Stream>,
  window: Option<Arc<Window>>,
}

impl App {
  fn capture(&mut self) -> Result {
    let destination = self.config.capture("png");
    let tx = self.capture_tx.clone();
    self.renderer.as_ref().unwrap().capture(move |capture| {
      if let Err(err) = tx.send(capture.save(&destination)) {
        eprintln!("failed to send capture result: {err}");
      }
    })?;

    self.captures_pending += 1;

    Ok(())
  }

  pub(crate) fn errors(mut self) -> Result {
    if self.errors.is_empty() {
      Ok(())
    } else {
      let source = self.errors.remove(0);
      Err(
        error::AppExit {
          additional: self.errors,
        }
        .into_error(Box::new(source)),
      )
    }
  }

  fn exit(&mut self) -> Result {
    self.sink.stop();

    if let Some(renderer) = &self.renderer {
      renderer.poll()?;
    }

    for _ in 0..self.captures_pending {
      match self.capture_rx.recv() {
        Ok(Ok(())) => {}
        Ok(Err(err)) => return Err(err),
        Err(mpsc::RecvError) => return Err(Error::internal("capture channel unexpectedly closed")),
      }
    }

    if let Some(recorder) = &self.recorder {
      recorder.lock().unwrap().save(&self.options, &self.config)?;
    }

    Ok(())
  }

  pub(crate) fn new(options: Options, record: bool, config: Config) -> Result<Self> {
    let host = cpal::default_host();

    let output_device = host
      .default_output_device()
      .context(error::AudioDefaultOutputDevice)?;

    let stream_config = Self::stream_config(
      output_device
        .supported_output_configs()
        .context(error::AudioSupportedStreamConfigs)?,
    )?;

    let mut output_stream = rodio::OutputStreamBuilder::from_device(output_device)
      .context(error::AudioBuildOutputStream)?
      .with_supported_config(&stream_config)
      .with_buffer_size(match stream_config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => {
          cpal::BufferSize::Fixed(128.clamp(*min, *max))
        }
        cpal::SupportedBufferSize::Unknown => cpal::BufferSize::Default,
      })
      .open_stream()
      .context(error::AudioBuildOutputStream)?;

    log::info!(
      "output stream opened: {}x{}x{}|{}",
      output_stream.config().channel_count(),
      output_stream.config().sample_rate(),
      output_stream.config().sample_format(),
      match output_stream.config().buffer_size() {
        cpal::BufferSize::Default => "default",
        cpal::BufferSize::Fixed(n) => &n.to_string(),
      }
    );

    output_stream.log_on_drop(false);

    let sink = Sink::connect_new(output_stream.mixer());

    sink.pause();

    if let Some(volume) = options.volume {
      sink.set_volume(volume);
    }

    let stream = if options.input {
      let input_device = host
        .default_input_device()
        .context(error::AudioDefaultInputDevice)?;

      let stream_config = Self::stream_config(
        input_device
          .supported_input_configs()
          .context(error::AudioSupportedStreamConfigs)?,
      )?;

      Box::new(Input::new(input_device, stream_config)?)
    } else {
      let stream = options.stream(&config)?;
      stream.append(&sink);
      stream
    };

    let recorder = record
      .then(|| Ok(Arc::new(Mutex::new(Recorder::new()?))))
      .transpose()?;

    let state = options.state();

    let (capture_tx, capture_rx) = mpsc::channel();

    let now = Instant::now();

    Ok(Self {
      analyzer: Analyzer::new(),
      capture_rx,
      capture_tx,
      captures_pending: 0,
      command: None,
      config,
      deadline: now,
      errors: Vec::new(),
      hub: Hub::new()?,
      last: now,
      macro_recording: None,
      makro: Vec::new(),
      options,
      output_stream,
      patch: Patch::default(),
      play: false,
      recorder,
      renderer: None,
      sink,
      state,
      stream,
      window: None,
    })
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key, repeat: bool) {
    let mut capture = true;

    if let Some(command) = self.command.as_mut() {
      match &key {
        Key::Character(c) => command.push(c.as_str().into()),
        Key::Named(NamedKey::Backspace) => {
          if command.pop().is_none() {
            self.command = None;
          }
        }
        Key::Named(NamedKey::Enter) => {
          let command = command.iter().flat_map(|c| c.chars()).collect::<String>();
          match command.as_str() {
            "left" => self.state.filters.push(Filter {
              color: invert_color(),
              field: Field::Left,
              wrap: self.state.wrap,
              ..default()
            }),
            "right" => self.state.filters.push(Filter {
              color: invert_color(),
              field: Field::Right,
              wrap: self.state.wrap,
              ..default()
            }),
            "spread" => self.state.spread = !self.state.spread,
            "status" => self.state.status = !self.state.status,
            _ => eprintln!("unknown command: {command}"),
          }
          self.command = None;
        }
        _ => {}
      }
    } else if self.play {
      if !repeat {
        match &key {
          Key::Named(NamedKey::Escape) => self.play = false,
          Key::Character(c) => match c.as_str() {
            "1" => self.patch = Patch::Sine,
            "2" => self.patch = Patch::Saw,
            _ => {
              if let Some(semitones) = Self::semitones(c) {
                self.patch.add(semitones, self.output_stream.mixer());
              }
            }
          },
          _ => {}
        }
      }
    } else {
      match &key {
        Key::Character(c) => match c.as_str() {
          "+" => {
            self.state.db += 1.0;
          }
          "-" => {
            self.state.db -= 1.0;
          }
          ":" => {
            self.command = Some(Vec::new());
          }
          ">" => {
            if let Err(err) = self.capture() {
              self.errors.push(err);
              event_loop.exit();
            }
          }
          "@" => {
            for (key, repeat) in self.makro.clone() {
              self.press(event_loop, key, repeat);
            }
            capture = false;
          }
          "a" => self.state.filters.push(Filter {
            color: invert_color(),
            field: Field::All,
            wrap: self.state.wrap,
            ..default()
          }),
          "c" => self.state.filters.push(Filter {
            color: invert_color(),
            field: Field::Circle,
            wrap: self.state.wrap,
            ..default()
          }),
          "d" => self.state.filters.push(Filter {
            coordinates: true,
            wrap: self.state.wrap,
            ..default()
          }),
          "f" => {
            self.state.fit = !self.state.fit;
          }
          "i" => {
            self.state.interpolate = !self.state.interpolate;
          }
          "l" => self.state.filters.push(Filter {
            color: invert_color(),
            field: Field::Frequencies,
            wrap: self.state.wrap,
            ..default()
          }),
          "n" => self.state.filters.push(Filter {
            field: Field::None,
            wrap: self.state.wrap,
            ..default()
          }),
          "p" => self.play = true,
          "q" => {
            if let Some(recording) = self.macro_recording.take() {
              self.makro = recording;
            } else {
              self.macro_recording = Some(Vec::new());
            }
            capture = false;
          }
          "r" => {
            self.state.repeat = !self.state.repeat;
          }
          "s" => self.state.filters.push(Filter {
            color: invert_color(),
            field: Field::Samples,
            wrap: self.state.wrap,
            ..default()
          }),
          "t" => {
            self.state.tile = !self.state.tile;
          }
          "w" => {
            self.state.wrap = !self.state.wrap;
          }
          "x" => self.state.filters.push(Filter {
            color: invert_color(),
            field: Field::X,
            wrap: self.state.wrap,
            ..default()
          }),
          "z" => self.state.filters.push(Filter {
            position: Mat3f::new_scaling(2.0),
            wrap: self.state.wrap,
            ..default()
          }),
          _ => {}
        },
        Key::Named(key) => match key {
          NamedKey::Backspace => {
            self.state.filters.pop();
          }
          NamedKey::ArrowLeft => {
            self.state.filters.push(Filter {
              position: Mat3f::new_rotation(-0.01),
              ..default()
            });
          }
          NamedKey::ArrowRight => {
            self.state.filters.push(Filter {
              position: Mat3f::new_rotation(0.01),
              ..default()
            });
          }
          _ => {}
        },
        _ => {}
      }
    }

    if capture && let Some(recording) = &mut self.macro_recording {
      recording.push((key, repeat));
    }
  }

  fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result {
    for message in self.hub.messages().lock().unwrap().drain(..) {
      match message.tuple() {
        (Controller::Spectra, 0, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Top,
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 1, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Bottom,
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 2, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::X,
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 3, Event::Button(true)) => self.state.filters.push(Filter {
          color: invert_color(),
          field: Field::Circle,
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 4, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_scaling(2.0),
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 5, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_scaling(0.5),
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 6, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_translation(&Vec2f::new(-0.1, 0.0)),
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 7, Event::Button(true)) => self.state.filters.push(Filter {
          position: Mat3f::new_translation(&Vec2f::new(0.1, 0.0)),
          wrap: self.state.wrap,
          ..default()
        }),
        (Controller::Spectra, 8, Event::Button(true)) => {
          self.state.filters.pop();
        }
        (Controller::Twister, control, Event::Button(true)) => match control {
          4 => self.state.position.x = 0.0,
          5 => self.state.position.y = 0.0,
          6 => self.state.position.z = 0.0,
          _ => {}
        },
        (Controller::Twister, control, Event::Encoder(parameter)) => {
          self.state.parameter = parameter;
          match control {
            0 => self.state.alpha = parameter,
            1 => self.state.db = parameter.value() as f32,
            4 => self.state.velocity.x = parameter.bipolar(),
            5 => self.state.velocity.y = parameter.bipolar(),
            6 => self.state.velocity.z = parameter.bipolar(),
            _ => {}
          }
        }
        _ => {}
      }
    }

    let sound = self.stream.drain();
    self
      .analyzer
      .update(&sound, self.stream.is_done(), &self.state);

    let now = Instant::now();
    let elapsed = now - self.last;
    self.last = now;

    self.state.tick(elapsed);

    let renderer = self.renderer.as_mut().unwrap();

    let frame = renderer.frame();

    renderer.render(&self.analyzer, &self.state, now)?;

    if let Some(recorder) = &self.recorder {
      let recorder = recorder.clone();
      let tx = self.capture_tx.clone();
      renderer.capture({
        move |image| {
          if let Err(err) = tx.send(recorder.lock().unwrap().frame(frame, image, sound)) {
            eprintln!("failed to send captured frame: {err}");
          }
        }
      })?;
      self.captures_pending += 1;
    }

    if self.captures_pending > 0 {
      loop {
        match self.capture_rx.try_recv() {
          Err(mpsc::TryRecvError::Empty) => {
            break;
          }
          Err(mpsc::TryRecvError::Disconnected) => {
            return Err(Error::internal("capture channel unexpectedly closed"));
          }
          Ok(result) => result?,
        }

        self.captures_pending -= 1;
      }
    }

    if self.recorder.is_some() && self.stream.is_done() {
      event_loop.exit();
    }

    Ok(())
  }

  fn resolution(&self, size: PhysicalSize<u32>) -> (Vector2<NonZeroU32>, NonZeroU32) {
    let size = Vector2::<NonZeroU32>::new(
      size.width.max(1).try_into().unwrap(),
      size.height.max(1).try_into().unwrap(),
    );

    let resolution = self.state.resolution.unwrap_or(size.x.max(size.y));

    (size, resolution)
  }

  fn semitones(key: &str) -> Option<u8> {
    #[allow(clippy::identity_op, clippy::match_same_arms)]
    match key {
      "z" => Some(0),
      "x" => Some(1),
      "c" => Some(2),
      "v" => Some(3),
      "b" => Some(4),
      "n" => Some(5),
      "m" => Some(6),
      "," => Some(7),
      "." => Some(8),
      "/" => Some(9),
      "a" => Some(0 + 5),
      "s" => Some(1 + 5),
      "d" => Some(2 + 5),
      "f" => Some(3 + 5),
      "g" => Some(4 + 5),
      "h" => Some(5 + 5),
      "j" => Some(6 + 5),
      "k" => Some(7 + 5),
      "l" => Some(8 + 5),
      ";" => Some(9 + 5),
      "'" => Some(10 + 5),
      "q" => Some(0 + 10),
      "w" => Some(1 + 10),
      "e" => Some(2 + 10),
      "r" => Some(3 + 10),
      "t" => Some(4 + 10),
      "y" => Some(5 + 10),
      "u" => Some(6 + 10),
      "i" => Some(7 + 10),
      "o" => Some(8 + 10),
      "p" => Some(9 + 10),
      "[" => Some(10 + 10),
      "]" => Some(11 + 10),
      "\\" => Some(12 + 10),
      _ => None,
    }
  }

  fn stream_config(
    configs: impl Iterator<Item = SupportedStreamConfigRange>,
  ) -> Result<SupportedStreamConfig> {
    let config = configs
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::AudioSupportedStreamConfig)?;

    Ok(SupportedStreamConfig::new(
      config.channels(),
      config.max_sample_rate(),
      *config.buffer_size(),
      config.sample_format(),
    ))
  }

  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }
}

impl ApplicationHandler for App {
  fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
    if let Some(fps) = self.state.fps {
      let now = Instant::now();

      while self.deadline <= now {
        self.deadline += fps.duration();
        self.window.as_ref().unwrap().request_redraw();
      }

      event_loop.set_control_flow(ControlFlow::WaitUntil(self.deadline));
    } else {
      self.window().request_redraw();
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    if let Err(err) = self.exit() {
      self.errors.push(err);
    }
  }

  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      assert!(self.renderer.is_none());

      let window = match event_loop
        .create_window(
          WindowAttributes::default()
            .with_inner_size(PhysicalSize {
              width: DEFAULT_RESOLUTION.get(),
              height: DEFAULT_RESOLUTION.get(),
            })
            .with_min_inner_size(PhysicalSize {
              width: 256,
              height: 256,
            })
            .with_title("device"),
        )
        .context(error::CreateWindow)
      {
        Ok(window) => Arc::new(window),
        Err(err) => {
          self.errors.push(err);
          event_loop.exit();
          return;
        }
      };

      let (size, resolution) = self.resolution(window.inner_size());

      self.window = Some(window.clone());

      let renderer = match pollster::block_on(Renderer::new(Some(window), size, resolution)) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.errors.push(err);
          event_loop.exit();
          return;
        }
      };

      self.renderer = Some(renderer);

      self.last = Instant::now();

      self.sink.play();
    }
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
    if self.renderer.is_none() {
      self.errors.push(Error::internal(format!(
        "window event received before renderer initialization: {event:?}",
      )));
      event_loop.exit();
      return;
    }

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }
      WindowEvent::Destroyed => {
        log::info!("window destroyed");
      }
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event_loop, event.logical_key, event.repeat);
      }
      WindowEvent::RedrawRequested => {
        if let Err(err) = self.redraw(event_loop) {
          self.errors.push(err);
          event_loop.exit();
        }
      }
      WindowEvent::Resized(size) => {
        let (size, resolution) = self.resolution(size);
        self.renderer.as_mut().unwrap().resize(size, resolution);
        self.window().request_redraw();
      }
      _ => {}
    }
  }
}
