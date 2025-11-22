use super::*;

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
  pub(crate) fn run(self, options: Options) -> Result {
    let mut stream = options.stream()?;

    let recorder = Arc::new(Mutex::new(Recorder::new()?));

    let mut analyzer = Analyzer::new();

    let state = options.state();

    let resolution = state.resolution.unwrap_or(RESOLUTION);

    let mut renderer = pollster::block_on(Renderer::new(
      None,
      Vector2::new(resolution, resolution),
      resolution,
    ))?;

    let fps = state.fps.unwrap_or(FPS.try_into().unwrap());

    let spf = fps.spf(stream.sample_rate())?;

    let (tx, rx) = mpsc::channel();

    let frames = self
      .duration
      .map(|duration| u64::from(duration.get()) * u64::from(fps.fps().get()));

    let progress = if let Some(frames) = frames {
      ProgressBar::new(frames)
        .with_style(ProgressStyle::default_bar().progress_chars(PROGRESS_CHARS))
    } else {
      ProgressBar::new_spinner().with_style(ProgressStyle::default_spinner().tick_chars(TICK_CHARS))
    };

    let mut done = false;
    for frame in 0.. {
      if frames.map_or(done, |frames| frame == frames) {
        break;
      }

      progress.inc(1);

      for _ in 0..spf * u32::from(stream.channels()) {
        done |= stream.next().is_none();
      }

      let sound = stream.drain();
      analyzer.update(&sound, done, &state);
      renderer.render(&analyzer, &state, Instant::now())?;

      let recorder = recorder.clone();
      let tx = tx.clone();
      renderer.capture(move |image| {
        if let Err(err) = tx.send(recorder.lock().unwrap().frame(frame, image, sound)) {
          eprintln!("failed to send captured frame: {err}");
        }
      })?;

      renderer.poll()?;

      rx.recv().unwrap()?;
    }

    progress.finish();

    recorder.lock().unwrap().save(&options)?;

    Ok(())
  }
}
