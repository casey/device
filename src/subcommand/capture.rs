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
    let sound_format = SoundFormat {
      sample_rate: DEFAULT_SAMPLE_RATE,
      channels: Tap::CHANNELS,
    };

    let mut tap = Tap::new(&options, sound_format.sample_rate);

    options.add_source(&config, &mut tap)?;

    let mut analyzer = Analyzer::new();

    let mut state = options.state();

    let resolution = options.resolution.unwrap_or(DEFAULT_RESOLUTION);

    let size = Vector2::new(
      options.width.unwrap_or(resolution),
      options.height.unwrap_or(resolution),
    );

    let (size, resolution) = options.size(size);

    let mut renderer = pollster::block_on(Renderer::new(
      options.image_format(),
      None,
      resolution,
      size,
      None,
    ))?;

    let fps = options.fps.unwrap_or(DEFAULT_FPS.try_into().unwrap());

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

    let mut samples = vec![0.0; spf.into_usize() * sound_format.channels.into_usize()];

    let mut recorder = Recorder::new(fps, &options, renderer.size(), sound_format)?;

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
