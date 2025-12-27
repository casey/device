use super::*;

const DEFAULT_SAMPLE_RATE: u32 = 48_000;

const PROGRESS_CHARS: &str = "█▉▊▋▌▍▎▏ ";

const TICK_CHARS: &str = concat!(
  "⠀⠁⠂⠃⠄⠅⠆⠇⡀⡁⡂⡃⡄⡅⡆⡇",
  "⠈⠉⠊⠋⠌⠍⠎⠏⡈⡉⡊⡋⡌⡍⡎⡏",
  "⠐⠑⠒⠓⠔⠕⠖⠗⡐⡑⡒⡓⡔⡕⡖⡗",
  "⠘⠙⠚⠛⠜⠝⠞⠟⡘⡙⡚⡛⡜⡝⡞⡟",
  "⠠⠡⠢⠣⠤⠥⠦⠧⡠⡡⡢⡣⡤⡥⡦⡧",
  "⠨⠩⠪⠫⠬⠭⠮⠯⡨⡩⡪⡫⡬⡭⡮⡯",
  "⠰⠱⠲⠳⠴⠵⠶⠷⡰⡱⡲⡳⡴⡵⡶⡷",
  "⠸⠹⠺⠻⠼⠽⠾⠿⡸⡹⡺⡻⡼⡽⡾⡿",
  "⢀⢁⢂⢃⢄⢅⢆⢇⣀⣁⣂⣃⣄⣅⣆⣇",
  "⢈⢉⢊⢋⢌⢍⢎⢏⣈⣉⣊⣋⣌⣍⣎⣏",
  "⢐⢑⢒⢓⢔⢕⢖⢗⣐⣑⣒⣓⣔⣕⣖⣗",
  "⢘⢙⢚⢛⢜⢝⢞⢟⣘⣙⣚⣛⣜⣝⣞⣟",
  "⢠⢡⢢⢣⢤⢥⢦⢧⣠⣡⣢⣣⣤⣥⣦⣧",
  "⢨⢩⢪⢫⢬⢭⢮⢯⣨⣩⣪⣫⣬⣭⣮⣯",
  "⢰⢱⢲⢳⢴⢵⢶⢷⣰⣱⣲⣳⣴⣵⣶⣷",
  "⢸⢹⢺⢻⢼⢽⢾⢿⣸⣹⣺⣻⣼⣽⣾⣿",
);

#[derive(Parser)]
pub(crate) struct Capture {
  #[arg(long)]
  duration: Option<NonZeroU32>,
  #[arg(long)]
  stem: Option<String>,
}

impl Capture {
  pub(crate) fn run(self, options: Options, config: Config) -> Result {
    let interrupt = Interrupt::register()?;

    let sound_format = SoundFormat {
      sample_rate: DEFAULT_SAMPLE_RATE,
      channels: Tap::CHANNELS,
    };

    let mut tap = Tap::new(&options, sound_format.sample_rate);

    options.add_source(&config, &mut tap)?;

    let mut analyzer = Analyzer::new();

    let mut rng = options.rng();

    let mut state = options.state(&config, &mut rng)?;

    let script = options.script();

    if let Some(script) = &script {
      for CommandEntry { name, command } in script.commands() {
        match command {
          Command::App(_) | Command::AppEventLoop(_) | Command::AppFallible(_) => {
            return Err(error::CaptureScriptAppCommand { command: name }.build());
          }
          Command::History(_)
          | Command::HistoryState(_)
          | Command::RngState(_)
          | Command::State(_) => {}
        }
      }
    }

    let mut history = History::default();

    let resolution = options.resolution.unwrap_or(DEFAULT_RESOLUTION);

    let size = Size::new(
      options.width.unwrap_or(resolution),
      options.height.unwrap_or(resolution),
    );

    let (size, resolution) = options.size(size);

    let mut renderer = pollster::block_on(Renderer::new(
      options.image_format,
      None,
      resolution,
      size,
      None,
    ))?;

    let fps = options.fps.unwrap_or(DEFAULT_FPS.into());

    let spf = fps.spf(tap.format())?;

    let (tx, rx) = mpsc::channel();

    let frames = self
      .duration
      .map(|duration| u64::from(duration.get()) * u64::from(fps.fps().get()));

    let progress = if options.verbose {
      ProgressBar::hidden()
    } else if let Some(frames) = frames {
      ProgressBar::new(frames)
        .with_style(ProgressStyle::default_bar().progress_chars(PROGRESS_CHARS))
    } else {
      ProgressBar::new_spinner().with_style(ProgressStyle::default_spinner().tick_chars(TICK_CHARS))
    };

    let mut samples = vec![0.0; spf];

    let mut recorder = Recorder::new(fps, &options, false, renderer.size(), sound_format)?;

    let mut done = false;
    for frame in 0.. {
      if frames.map_or(done, |frames| frame == frames) || interrupt.interrupted() {
        break;
      }

      progress.inc(1);

      done = tap.is_done();

      tap.write(&mut samples);

      let last = tap.position();

      let sound = tap.drain();
      analyzer.update(&sound, done, &state);
      renderer.render(&analyzer, &state, None)?;

      let tx = tx.clone();
      renderer.capture(move |image| {
        if let Err(err) = tx.send(image) {
          eprintln!("failed to send captured frame: {err}");
        }
      })?;

      renderer.poll()?;

      let image = rx.recv().unwrap();

      recorder.frame(frame, image, sound)?;

      history.tick(&mut state);

      let position = tap.position();

      let tick = Tick {
        dt: fps.dt(),
        last,
        position,
        tempo: tap.tempo(),
        time: tap.time(),
      };

      if let Some(script) = &script {
        for CommandEntry { name, command } in script.tick(tick) {
          log::info!("dispatching script command {name}");
          match command {
            Command::App(_) | Command::AppEventLoop(_) | Command::AppFallible(_) => unreachable!(),
            Command::History(command) => command(&mut history),
            Command::HistoryState(command) => command(&mut history, &mut state),
            Command::RngState(command) => {
              history.states.push(state.clone());
              command(&mut rng, &mut state);
            }
            Command::State(command) => {
              history.states.push(state.clone());
              command(&mut state);
            }
          }
        }
      }

      state.tick(tick);
    }

    progress.finish();

    recorder.finish(&options, &config, self.stem.as_deref())?;

    Ok(())
  }
}
