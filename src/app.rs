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
  fullscreen: bool,
  hub: Hub,
  input: Option<Input>,
  last: Instant,
  mode: Mode,
  modifiers: Modifiers,
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
  pub(crate) fn capture(&mut self) -> Result {
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

  fn dispatch(&mut self, event_loop: &ActiveEventLoop, command: Command) {
    match command {
      Command::App(command) => command(self),
      Command::AppEventLoop(command) => command(self, event_loop),
      Command::AppFallible(command) => {
        if let Err(err) = command(self) {
          self.errors.push(err);
          event_loop.exit();
        }
      }
      Command::State(command) => command(&mut self.state),
    }
  }

  pub(crate) fn enter_mode(&mut self, mode: Mode) {
    self.mode = mode;
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
      fullscreen: false,
      hub: Hub::new()?,
      input,
      last: now,
      mode: Mode::Normal,
      modifiers: Modifiers::default(),
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

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key) {
    if let Mode::Play = self.mode
      && let Key::Character(c) = &key
      && let Some(semitones) = Self::semitones(c)
    {
      self.patch.sequence(semitones, &mut self.tap);
      return;
    }

    if let Mode::Command(command) = &mut self.mode
      && let Key::Character(c) = &key
    {
      command.push(c.as_str().into());
      return;
    }

    if let Some(command) = self.bindings.key((&self.mode).into(), &key, self.modifiers) {
      self.dispatch(event_loop, command);
    }
  }

  pub(crate) fn pop_command(&mut self) {
    let Mode::Command(command) = &mut self.mode else {
      return;
    };

    if command.pop().is_none() {
      self.mode = Mode::Normal;
    }
  }

  pub(crate) fn complete_command(&mut self) {
    let Mode::Command(command) = &mut self.mode else {
      return;
    };

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

  pub(crate) fn execute_command(&mut self, event_loop: &ActiveEventLoop) {
    let Mode::Command(command) = &mut self.mode else {
      return;
    };

    let command = command.iter().flat_map(|c| c.chars()).collect::<String>();
    if let Some(command) = self.commands.name(command.as_str()) {
      self.dispatch(event_loop, command);
    } else {
      eprintln!("unknown command: {command}");
    }
    self.mode = Mode::Normal;
  }

  fn process_messages(&mut self, event_loop: &ActiveEventLoop) {
    for message in self.hub.drain() {
      match message.event {
        Event::Button(press) => {
          if let Some(command) = self
            .bindings
            .button(message.controller, message.control, press)
          {
            self.dispatch(event_loop, command);
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

  fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result {
    self.process_messages(event_loop);

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

  pub(crate) fn reload_shaders(&mut self) {
    if let Err(err) = self.renderer.as_mut().unwrap().reload_shaders() {
      eprintln!("failed to reload shader: {err}");
    }
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

  pub(crate) fn set_patch(&mut self, patch: Patch) {
    self.patch = patch;
  }

  pub(crate) fn toggle_fullscreen(&mut self) {
    self.fullscreen.toggle();
    self
      .window()
      .set_fullscreen(self.fullscreen.then_some(Fullscreen::Borderless(None)));
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
      WindowEvent::KeyboardInput { event, .. }
        if event.state == ElementState::Pressed && !event.repeat =>
      {
        self.press(event_loop, event.logical_key);
      }
      WindowEvent::ModifiersChanged(modifiers) => self.modifiers = modifiers,
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
