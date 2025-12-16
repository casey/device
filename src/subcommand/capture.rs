use super::*;

const DEFAULT_FPS: NonZeroU32 = NonZeroU32::new(60).unwrap();

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
}

impl Capture {
  pub(crate) fn run(self, options: Options, config: Config) -> Result {
    let mut tap = Tap::new(&options, DEFAULT_SAMPLE_RATE);

    options.add_source(&config, &mut tap)?;

    let mut analyzer = Analyzer::new();

    let mut state = options.state();

    let resolution = state.resolution.unwrap_or(DEFAULT_RESOLUTION);

    let mut renderer = pollster::block_on(Renderer::new(
      options.format(),
      None,
      resolution,
      Vector2::new(resolution, resolution),
      None,
    ))?;

    let fps = state.fps.unwrap_or(DEFAULT_FPS.try_into().unwrap());

    let spf = fps.spf(tap.sample_rate())?;

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

    let mut samples = vec![0.0; spf.into_usize() * Tap::CHANNELS.into_usize()];

    let mut recorder = Recorder::new(&options, resolution, fps)?;

    let mut done = false;
    for frame in 0.. {
      if frames.map_or(done, |frames| frame == frames) {
        break;
      }

      progress.inc(1);

      done = tap.is_done();

      tap.write(&mut samples);

      let sound = tap.drain();
      analyzer.update(&sound, done, &state);
      renderer.render(&analyzer, &state, Instant::now())?;

      let tx = tx.clone();
      renderer.capture(move |image| {
        if let Err(err) = tx.send(image) {
          eprintln!("failed to send captured frame: {err}");
        }
      })?;

      renderer.poll()?;

      let image = rx.recv().unwrap();

      recorder.frame(frame, image, sound)?;

      state.tick(fps.duration());
    }

    progress.finish();

    recorder.finish(&options, &config)?;

    Ok(())
  }
}
