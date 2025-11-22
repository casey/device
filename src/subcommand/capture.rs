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
      &tempdir_path.join("audio.wav"),
      media.iter().map(|(_image, sound)| sound),
    )?;

    let mut child = Command::new("ffmpeg")
      .args(["-f", "rawvideo"])
      .args(["-pixel_format", "rgb24"])
      .args(["-video_size", &format!("{resolution}x{resolution}")])
      .args(["-framerate", &fps.to_string()])
      .args(["-i", "-"])
      .args(["-i", AUDIO])
      .args(["-c:v", "libx264"])
      .args(["-crf", "18"])
      .args(["-movflags", "+faststart"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-preset", "slow"])
      .args(["-c:a", "aac"])
      .arg(RECORDING)
      .current_dir(&tempdir_path)
      .stdin(Stdio::piped())
      .stderr(if options.verbose {
        Stdio::inherit()
      } else {
        Stdio::piped()
      })
      .stdout(if options.verbose {
        Stdio::inherit()
      } else {
        Stdio::piped()
      })
      .spawn()
      .context(error::RecordingInvoke)?;

    let mut stdin = BufWriter::new(child.stdin.as_ref().unwrap());

    for (image, _sound) in media {
      for pixel in image.data().chunks(4) {
        stdin.write_all(&pixel[0..3]).unwrap();
      }
    }

    stdin.flush().unwrap();

    drop(stdin);

    child.wait().unwrap();

    fs::rename(tempdir_path.join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    // todo:
    // - captureinvoke error
    // - error handling

    Ok(())
  }
}
