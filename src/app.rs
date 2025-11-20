use super::*;

pub(crate) struct App {
  analyzer: Analyzer,
  captures: Vec<mpsc::Receiver<Result>>,
  command: Option<Vec<String>>,
  errors: Vec<Error>,
  horizontal: f32,
  hub: Hub,
  macro_recording: Option<Vec<Key>>,
  makro: Vec<Key>,
  #[allow(unused)]
  output_stream: OutputStream,
  recorder: Option<Arc<Mutex<Recorder>>>,
  renderer: Option<Renderer>,
  scaling: f32,
  #[allow(unused)]
  sink: Sink,
  start: Instant,
  state: State,
  stream: Box<dyn Stream>,
  translation: Vec2f,
  vertical: f32,
  window: Option<Arc<Window>>,
  zoom: f32,
}

impl App {
  fn capture(&mut self) -> Result {
    let (tx, rx) = mpsc::channel();

    self.renderer.as_ref().unwrap().capture(move |capture| {
      if let Err(err) = tx.send(capture.save("capture.png".as_ref())) {
        eprintln!("failed to send capture result: {err}");
      }
    })?;

    self.captures.push(rx);

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

  fn find_song(song: &str) -> Result<Utf8PathBuf> {
    let song = RegexBuilder::new(song)
      .case_insensitive(true)
      .build()
      .context(error::SongRegex)?;

    let mut matches = Vec::<Utf8PathBuf>::new();

    let home = dirs::home_dir().context(error::Home)?;

    let music = home.join("Music/Music/Media.localized/Music");

    for entry in WalkDir::new(&music) {
      let entry = entry.context(error::SongWalk)?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = entry.path();

      let haystack = path.strip_prefix(&music).unwrap().with_extension("");

      let Some(haystack) = haystack.to_str() else {
        continue;
      };

      if song.is_match(haystack) {
        matches.push(path.into_utf8_path()?.into());
      }
    }

    if matches.len() > 1 {
      return Err(error::SongAmbiguous { matches }.build());
    }

    match matches.into_iter().next() {
      Some(path) => Ok(path),
      None => Err(error::SongMatch { song }.build()),
    }
  }

  pub(crate) fn new(options: Options) -> Result<Self> {
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
      .open_stream()
      .context(error::AudioBuildOutputStream)?;

    output_stream.log_on_drop(false);

    let sink = Sink::connect_new(output_stream.mixer());

    if let Some(volume) = options.volume {
      sink.set_volume(volume);
    }

    let stream: Box<dyn Stream> = if options.input {
      let input_device = host
        .default_input_device()
        .context(error::AudioDefaultInputDevice)?;

      let stream_config = Self::stream_config(
        input_device
          .supported_input_configs()
          .context(error::AudioSupportedStreamConfigs)?,
      )?;

      Box::new(Input::new(input_device, stream_config)?)
    } else if let Some(song) = &options.song {
      let track = Track::new(&Self::find_song(song)?)?;
      sink.append(track.clone());
      Box::new(track)
    } else if options.synthesizer {
      let synthesizer = Synthesizer::busy_signal();
      sink.append(synthesizer.clone());
      Box::new(synthesizer)
    } else if let Some(track) = &options.track {
      let track = Track::new(track)?;
      sink.append(track.clone());
      Box::new(track)
    } else {
      let synthesizer = Synthesizer::silence();
      sink.append(synthesizer.clone());
      Box::new(synthesizer)
    };

    let state = options.program.map(Program::state).unwrap_or_default();

    Ok(Self {
      analyzer: Analyzer::new(),
      captures: Vec::new(),
      command: None,
      errors: Vec::new(),
      horizontal: 0.0,
      hub: Hub::new()?,
      macro_recording: None,
      makro: Vec::new(),
      output_stream,
      recorder: options
        .record
        .then(|| Ok(Arc::new(Mutex::new(Recorder::new()?))))
        .transpose()?,
      renderer: None,
      scaling: 1.0,
      sink,
      start: Instant::now(),
      state,
      stream,
      translation: Vec2f::zeros(),
      vertical: 0.0,
      window: None,
      zoom: 0.0,
    })
  }

  fn press(&mut self, event_loop: &ActiveEventLoop, key: Key) {
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
            for key in self.makro.clone() {
              self.press(event_loop, key);
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
      recording.push(key);
    }
  }

  fn redraw(&mut self, event_loop: &ActiveEventLoop) {
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
          4 => self.translation.x = 0.0,
          5 => self.translation.y = 0.0,
          6 => self.scaling = 1.0,
          _ => {}
        },
        (Controller::Twister, control, Event::Encoder(parameter)) => {
          self.state.parameter = parameter;
          match control {
            0 => self.state.alpha = parameter,
            1 => self.state.db = parameter.value() as f32,
            4 => self.horizontal = parameter.bipolar(),
            5 => self.vertical = parameter.bipolar(),
            6 => self.zoom = parameter.bipolar(),
            _ => {}
          }
        }
        _ => {}
      }
    }

    let sound = self.stream.drain();
    self
      .analyzer
      .update(&sound, self.stream.done(), &self.state);

    let now = Instant::now();
    let elapsed = (now - self.start).as_secs_f32();
    self.start = now;

    self.scaling -= self.zoom * elapsed;
    self.translation.x -= self.horizontal * 4.0 * elapsed;
    self.translation.y -= self.vertical * 4.0 * elapsed;

    self.state.filters.push(Filter {
      position: Mat3f::new_translation(&self.translation).prepend_scaling(self.scaling),
      wrap: self.state.wrap,
      ..default()
    });

    let renderer = self.renderer.as_mut().unwrap();

    if let Err(err) = renderer.render(&self.analyzer, &self.state, now) {
      self.errors.push(err);
      event_loop.exit();
      return;
    }

    if let Some(recorder) = &self.recorder {
      let recorder = recorder.clone();
      let (tx, rx) = mpsc::channel();
      if let Err(err) = renderer.capture({
        move |frame| {
          if let Err(err) = tx.send(recorder.lock().unwrap().frame(frame, sound)) {
            eprintln!("failed to send captured frame: {err}");
          }
        }
      }) {
        self.errors.push(err);
        event_loop.exit();
        return;
      }
      self.captures.push(rx);
    }

    self.state.filters.pop();

    self.window().request_redraw();
  }

  fn resolution(&self, size: PhysicalSize<u32>) -> (Vector2<NonZeroU32>, NonZeroU32) {
    let size = Vector2::<NonZeroU32>::new(
      size.width.max(1).try_into().unwrap(),
      size.height.max(1).try_into().unwrap(),
    );

    let resolution = self.state.resolution.unwrap_or(size.x.max(size.y));

    (size, resolution)
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
      match config.buffer_size() {
        SupportedBufferSize::Range { min, .. } => SupportedBufferSize::Range {
          min: *min,
          max: *min,
        },
        SupportedBufferSize::Unknown => SupportedBufferSize::Unknown,
      },
      config.sample_format(),
    ))
  }

  fn window(&self) -> &Window {
    self.window.as_ref().unwrap()
  }
}

impl ApplicationHandler for App {
  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    if let Some(renderer) = &self.renderer {
      if let Err(err) = renderer.poll() {
        eprintln!("failed to poll renderer: {err}");
      }
    }

    for capture in &self.captures {
      if let Err(err) = capture.recv() {
        eprintln!("capture failed: {err}");
      }
    }

    if let Some(recorder) = &self.recorder
      && let Err(err) = recorder.lock().unwrap().save()
    {
      eprintln!("failed to save recording: {err}");
    }
  }

  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    if self.window.is_none() {
      assert!(self.renderer.is_none());

      let window = match event_loop
        .create_window(
          WindowAttributes::default()
            .with_inner_size(PhysicalSize {
              width: 1024,
              height: 1024,
            })
            .with_min_inner_size(PhysicalSize {
              width: 256,
              height: 256,
            })
            .with_title("x"),
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
      WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
        self.press(event_loop, event.logical_key);
      }
      WindowEvent::RedrawRequested => {
        self.redraw(event_loop);
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
