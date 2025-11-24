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
  const AUDIO: &str = "audio.wav";

  pub(crate) fn run(self, options: Options) -> Result {
    let config = Config::load()?;

    let mut stream = options.stream(&config)?;

    let mut analyzer = Analyzer::new();

    let state = options.state();

    let resolution = state.resolution.unwrap_or(DEFAULT_RESOLUTION);

    let mut renderer = pollster::block_on(Renderer::new(
      None,
      Vector2::new(resolution, resolution),
      resolution,
    ))?;

    let fps = state.fps.unwrap_or(DEFAULT_FPS.try_into().unwrap());

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

    let mut media = Vec::new();

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

      let tx = tx.clone();
      renderer.capture(move |image| {
        if let Err(err) = tx.send(image) {
          eprintln!("failed to send captured frame: {err}");
        }
      })?;

      renderer.poll()?;

      let image = rx.recv().unwrap();
      media.push((image, sound));
    }

    progress.finish();

    let tempdir = TempDir::new().context(error::TempdirIo)?;
    let tempdir_path = tempdir.path().into_utf8_path()?;

    Sound::save(
      &tempdir_path.join(Self::AUDIO),
      media.iter().map(|(_image, sound)| sound),
    )?;

    let child = Command::new("ffmpeg")
      .args(["-f", "rawvideo"])
      .args(["-pixel_format", "rgb24"])
      .args(["-video_size", &format!("{resolution}x{resolution}")])
      .args(["-framerate", &fps.to_string()])
      .args(["-i", "-"])
      .args(["-i", Self::AUDIO])
      .args(["-c:v", "libx264"])
      .args(["-crf", "18"])
      .args(["-movflags", "+faststart"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-preset", "slow"])
      .args(["-c:a", "aac"])
      .args(["-b:a", "192k"])
      .arg(RECORDING)
      .current_dir(tempdir_path)
      .stdin(Stdio::piped())
      .stderr(options.stdio())
      .stdout(options.stdio())
      .spawn()
      .context(error::CaptureInvoke)?;

    let mut stdin = BufWriter::new(child.stdin.as_ref().unwrap());

    for (image, _sound) in media {
      for pixel in image.data().chunks(4) {
        stdin.write_all(&pixel[0..3]).context(error::CaptureWrite)?;
      }
    }

    stdin.flush().context(error::CaptureFlush)?;

    drop(stdin);

    let output = child.wait_with_output().context(error::CaptureWait)?;

    if !output.status.success() {
      if !options.verbose {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
      }

      return Err(
        error::CaptureStatus {
          status: output.status,
        }
        .build(),
      );
    }

    let path = if let Some(captures) = config.captures() {
      captures.join(format!(
        "{}.mp4",
        SystemTime::now()
          .duration_since(UNIX_EPOCH)
          .unwrap_or_default()
          .as_secs()
      ))
    } else {
      RECORDING.into()
    };

    fs::rename(tempdir_path.join(RECORDING), &path).context(error::FilesystemIo { path })?;

    Ok(())
  }
}
