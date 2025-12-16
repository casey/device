use super::*;

pub(crate) struct App {
  pub(crate) analyzer: Analyzer,
  pub(crate) bindings: Bindings,
  pub(crate) capture_rx: mpsc::Receiver<Result>,
  pub(crate) capture_tx: mpsc::Sender<Result>,
  pub(crate) captures_pending: u64,
  pub(crate) commands: Commands,
  pub(crate) config: Config,
  pub(crate) cursor_moved: Instant,
  pub(crate) cursors: HashSet<DeviceId>,
  pub(crate) deadline: Instant,
  pub(crate) errors: Vec<Error>,
  pub(crate) fullscreen: bool,
  pub(crate) history: Vec<State>,
  pub(crate) hub: Hub,
  pub(crate) input: Option<Input>,
  pub(crate) last: Instant,
  pub(crate) mode: Mode,
  pub(crate) modifiers: Modifiers,
  pub(crate) options: Options,
  pub(crate) patch: Patch,
  pub(crate) present_mode: Option<PresentMode>,
  pub(crate) record: Option<Fps>,
  pub(crate) recorder: Option<Arc<Mutex<Recorder>>>,
  pub(crate) renderer: Option<Renderer>,
  pub(crate) state: State,
  pub(crate) tap: Tap,
  pub(crate) unwind: bool,
  pub(crate) window: Option<Arc<Window>>,
}

impl App {
  pub(crate) fn dispatch(&mut self, event_loop: &ActiveEventLoop, command: Command) {
    match command {
      Command::App(command) => command(self),
      Command::AppEventLoop(command) => command(self, event_loop),
      Command::AppFallible(command) => {
        if let Err(err) = command(self) {
          self.errors.push(err);
          event_loop.exit();
        }
      }
      Command::State(command) => {
        self.history.push(self.state.clone());
        command(&mut self.state);
      }
    }
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

    if let Some(recorder) = self.recorder.take() {
      Arc::try_unwrap(recorder)
        .ok()
        .unwrap()
        .into_inner()
        .unwrap()
        .finish(&self.options, &self.config)?;
    }

    Ok(())
  }

  pub(crate) fn new(
    options: Options,
    present_mode: Option<PresentMode>,
    record: Option<Fps>,
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

    let state = options.state();

    let (capture_tx, capture_rx) = mpsc::channel();

    let now = Instant::now();

    Ok(Self {
      analyzer: Analyzer::new(),
      bindings: Bindings::new(),
      recorder: None,
      capture_rx,
      capture_tx,
      captures_pending: 0,
      commands: Commands::new(),
      config,
      cursor_moved: now,
      cursors: HashSet::new(),
      deadline: now,
      errors: Vec::new(),
      fullscreen: false,
      history: Vec::new(),
      hub: Hub::new()?,
      input,
      last: now,
      mode: Mode::Normal,
      record,
      modifiers: Modifiers::default(),
      options,
      patch: Patch::default(),
      present_mode,
      renderer: None,
      state,
      tap,
      unwind: false,
      window: None,
    })
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key, repeat: bool) {
    if let Mode::Play = self.mode
      && let Key::Character(c) = &key
      && let Some(semitones) = Self::semitones(c)
    {
      if !repeat {
        self.patch.sequence(semitones, &mut self.tap);
      }
      return;
    }

    if let Mode::Command(command) = &mut self.mode
      && let Key::Character(c) = &key
    {
      command.push(c.as_str().into());
      self.print_command();
      return;
    }

    if let Some(command) = self.bindings.key((&self.mode).into(), &key, self.modifiers) {
      self.dispatch(event_loop, command);
    }
  }

  pub(crate) fn print_command(&self) {
    let Mode::Command(command) = &self.mode else {
      return;
    };

    eprintln!(
      ":{}",
      command.iter().flat_map(|c| c.chars()).collect::<String>(),
    );
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
            let value = command(&mut self.state, parameter);
            self.state.encoder = value;
          }
        }
      }
    }
  }

  fn redraw(&mut self, event_loop: &ActiveEventLoop) -> Result {
    self.window().set_cursor_visible(
      self.cursors.is_empty() || self.cursor_moved.elapsed().as_secs_f32() < 2.0,
    );

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
    let dt = now - self.last;
    self.last = now;

    if self.unwind {
      if let Some(last) = self.history.pop() {
        self.state = last;
      } else {
        self.unwind = false;
      }
    }

    self.state.tick(dt);

    let renderer = self.renderer.as_mut().unwrap();

    let frame = renderer.frame();

    renderer.render(&self.analyzer, &self.state, now)?;

    if let Some(fps) = self.record
      && self.recorder.is_none()
    {
      self.recorder = Some(Arc::new(Mutex::new(Recorder::new(
        &self.options,
        renderer.resolution(),
        fps,
      )?)));
    }

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

  pub(crate) fn window(&self) -> &Window {
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
      WindowEvent::CursorEntered { device_id } => {
        self.cursors.remove(&device_id);
      }
      WindowEvent::CursorLeft { device_id } => {
        self.cursors.insert(device_id);
      }
      WindowEvent::CursorMoved { device_id, .. } => {
        self.cursors.insert(device_id);
        self.cursor_moved = Instant::now();
      }
      WindowEvent::Destroyed => {
        log::info!("window destroyed");
      }
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event_loop, event.logical_key, event.repeat);
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
