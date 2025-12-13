use super::*;

pub(crate) struct App {
  analyzer: Analyzer,
  bindings: Bindings,
  capture_rx: mpsc::Receiver<Result>,
  capture_tx: mpsc::Sender<Result>,
  captures_pending: u64,
  commands: Commands,
  config: Config,
  deadline: Instant,
  errors: Vec<Error>,
  hub: Hub,
  input: Option<Input>,
  last: Instant,
  macro_recording: Option<Vec<(Key, bool)>>,
  makro: Vec<(Key, bool)>,
  mode: Mode,
  options: Options,
  patch: Patch,
  present_mode: Option<PresentMode>,
  recorder: Option<Arc<Mutex<Recorder>>>,
  renderer: Option<Renderer>,
  state: State,
  tap: Tap,
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

  pub(crate) fn errors(self) -> Result {
    let mut errors = self.errors.into_iter();

    if let Some(source) = errors.next() {
      Err(
        error::AppExit {
          additional: errors.collect::<Vec<Error>>(),
        }
        .into_error(Box::new(source)),
      )
    } else {
      Ok(())
    }
  }

  fn exit(&mut self) -> Result {
    self.tap.pause();

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

  pub(crate) fn new(
    options: Options,
    present_mode: Option<PresentMode>,
    record: bool,
    config: Config,
  ) -> Result<Self> {
    let host = cpal::default_host();

    let output_device = host
      .default_output_device()
      .context(error::AudioDefaultOutputDevice)?;

    let supported_stream_config = Self::select_stream_config(
      output_device
        .supported_output_configs()
        .context(error::AudioSupportedStreamConfigs)?,
      Tap::CHANNELS,
      Tap::CHANNELS,
    )?;

    let mut stream_config = supported_stream_config.config();
    stream_config.buffer_size = match supported_stream_config.buffer_size() {
      SupportedBufferSize::Range { min, max } => {
        BufferSize::Fixed(DEFAULT_BUFFER_SIZE.clamp(*min, *max))
      }
      SupportedBufferSize::Unknown => BufferSize::Default,
    };

    let mut tap = Tap::new(&options, stream_config.sample_rate.0);

    tap.pause();

    tap.stream(&output_device, &stream_config)?;

    let input = if options.input {
      let input_device = host
        .default_input_device()
        .context(error::AudioDefaultInputDevice)?;

      let stream_config = Self::select_stream_config(
        input_device
          .supported_input_configs()
          .context(error::AudioSupportedStreamConfigs)?,
        1,
        2,
      )?;

      Some(Input::new(input_device, stream_config)?)
    } else {
      None
    };

    options.add_source(&config, &mut tap)?;

    let recorder = record
      .then(|| Ok(Arc::new(Mutex::new(Recorder::new()?))))
      .transpose()?;

    let state = options.state();

    let (capture_tx, capture_rx) = mpsc::channel();

    let now = Instant::now();

    Ok(Self {
      analyzer: Analyzer::new(),
      bindings: Bindings::new(),
      capture_rx,
      capture_tx,
      captures_pending: 0,
      commands: Commands::new(),
      config,
      deadline: now,
      errors: Vec::new(),
      hub: Hub::new()?,
      input,
      last: now,
      macro_recording: None,
      makro: Vec::new(),
      mode: Mode::Normal,
      options,
      patch: Patch::default(),
      present_mode,
      recorder,
      renderer: None,
      state,
      tap,
      window: None,
    })
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key, repeat: bool) {
    let mut capture = true;

    match self.mode {
      Mode::Command(_) => self.press_command(&key),
      Mode::Normal => self.press_normal(&mut capture, event_loop, &key),
      Mode::Play => self.press_play(&key, repeat),
    }

    if capture && let Some(recording) = &mut self.macro_recording {
      recording.push((key, repeat));
    }
  }

  fn press_command(&mut self, key: &Key) {
    let Mode::Command(command) = &mut self.mode else {
      panic!("press_command called in wrong mode: {:?}", self.mode);
    };

    match &key {
      Key::Character(c) => command.push(c.as_str().into()),
      Key::Named(NamedKey::Backspace) => {
        if command.pop().is_none() {
          self.mode = Mode::Normal;
        }
      }
      Key::Named(NamedKey::Enter) => {
        let command = command.iter().flat_map(|c| c.chars()).collect::<String>();
        if let Some(command) = self.commands.name(command.as_str()) {
          command(&mut self.state);
        } else {
          eprintln!("unknown command: {command}");
        }
        self.mode = Mode::Normal;
      }
      Key::Named(NamedKey::Tab) => {
        let prefix = command.iter().flat_map(|c| c.chars()).collect::<String>();

        if let Some(suffix) = self.commands.complete(&prefix) {
          if !suffix.is_empty() {
            eprintln!("completion: {prefix}{suffix}");
            command.push(suffix.into());
          }
        } else {
          eprintln!("no completion found for: {prefix}");
        }
      }
      _ => {}
    }
  }

  fn press_normal(&mut self, capture: &mut bool, event_loop: &ActiveEventLoop, key: &Key) {
    if let Some(command) = self.bindings.key(key) {
      command(&mut self.state);
    } else if let Key::Character(c) = &key {
      match c.as_str() {
        ":" => {
          self.mode = Mode::Command(Vec::new());
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
          *capture = false;
        }
        "p" => self.mode = Mode::Play,
        "q" => {
          if let Some(recording) = self.macro_recording.take() {
            self.makro = recording;
          } else {
            self.macro_recording = Some(Vec::new());
          }
          *capture = false;
        }
        "R" => {
          if let Err(err) = self.renderer.as_mut().unwrap().reload_shader() {
            eprintln!("failed to reload shader: {err}");
          }
        }
        _ => {}
      }
    }
  }

  fn press_play(&mut self, key: &Key, repeat: bool) {
    if !repeat {
      match &key {
        Key::Named(NamedKey::Escape) => self.mode = Mode::Normal,
        Key::Character(c) => match c.as_str() {
          "1" => self.patch = Patch::Sine,
          "2" => self.patch = Patch::Saw,
          _ => {
            if let Some(semitones) = Self::semitones(c) {
              self.patch.sequence(semitones, &mut self.tap);
            }
          }
        },
        _ => {}
      }
    }
  }

  fn process_messages(&mut self) {
    for message in self.hub.messages().lock().unwrap().drain(..) {
      match message.event {
        Event::Button(press) => {
          if let Some(command) = self
            .bindings
            .button(message.controller, message.control, press)
          {
            command(&mut self.state);
          }
        }
        Event::Encoder(parameter) => {
          if let Some(command) = self.bindings.encoder(message.controller, message.control) {
            command(&mut self.state, parameter);
          }
        }
      }
    }
  }

  fn redraw(&mut self) -> Result {
    self.process_messages();

    let sound = if let Some(input) = &self.input {
      input.drain()
    } else {
      self.tap.drain()
    };

    self.analyzer.update(
      &sound,
      self.input.is_none() && self.tap.is_done(),
      &self.state,
    );

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

  fn select_stream_config(
    configs: impl Iterator<Item = SupportedStreamConfigRange>,
    min_channels: u16,
    max_channels: u16,
  ) -> Result<SupportedStreamConfig> {
    let config = configs
      .filter(|config| {
        config.channels() >= min_channels && config.sample_format() == cpal::SampleFormat::F32
      })
      .max_by_key(SupportedStreamConfigRange::max_sample_rate)
      .context(error::AudioSupportedStreamConfig)?;

    Ok(SupportedStreamConfig::new(
      config.channels().clamp(min_channels, max_channels),
      config.max_sample_rate(),
      *config.buffer_size(),
      config.sample_format(),
    ))
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
            .with_title("device")
            .with_platform_attributes(),
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

      let renderer = match pollster::block_on(Renderer::new(
        self.options.format(),
        self.present_mode,
        resolution,
        size,
        Some(window),
      )) {
        Ok(renderer) => renderer,
        Err(err) => {
          self.errors.push(err);
          event_loop.exit();
          return;
        }
      };

      self.renderer = Some(renderer);

      self.last = Instant::now();

      if self.input.is_none() {
        self.tap.play();
      }
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
        if let Err(err) = self.redraw() {
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
