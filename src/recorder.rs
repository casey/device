use {
  super::*,
  std::process::{Child, ChildStdin, Command},
};

const VIDEO: &str = "video.mp4";

pub(crate) struct Recorder {
  audio: Vec<Sound>,
  encoder: Child,
  end: Option<u64>,
  frames: HashMap<u64, (Image, Sound)>,
  heap: BinaryHeap<Reverse<u64>>,
  next: u64,
  size: Vector2<NonZeroU32>,
  stdin: BufWriter<ChildStdin>,
  #[allow(unused)]
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  pub(crate) fn finish(mut self, options: &Options, config: &Config) -> Result {
    assert!(self.heap.is_empty());

    self.stdin.flush().context(error::RecordingFlush)?;
    drop(self.stdin);

    let output = self
      .encoder
      .wait_with_output()
      .context(error::RecordingWait)?;

    Self::process_output(options, &output)?;

    Sound::save(&self.tempdir_path.join(AUDIO), self.audio.iter())?;

    let output = Command::new("ffmpeg")
      .args(["-i", VIDEO])
      .args(["-i", AUDIO])
      .args(["-c:v", "copy"])
      .args(["-movflags", "+faststart"])
      .args(["-c:a", "aac"])
      .arg(RECORDING)
      .current_dir(&self.tempdir_path)
      .stderr(options.stdio())
      .stdout(options.stdio())
      .output()
      .context(error::RecordingInvoke)?;

    Self::process_output(options, &output)?;

    let path = config.capture("mp4");

    fs::rename(self.tempdir_path.join(RECORDING), &path).context(error::FilesystemIo { path })?;

    Ok(())
  }

  pub(crate) fn frame(&mut self, frame: u64, image: Image, sound: Sound) -> Result {
    if image.width() != self.size.x.get() || image.height() != self.size.y.get() {
      log::warn!("recording resolution changed");
      match self.end {
        Some(end) => self.end = Some(end.min(frame)),
        None => self.end = Some(frame),
      }
    }

    if let Some(end) = self.end
      && frame >= end
    {
      return Ok(());
    }

    self.heap.push(Reverse(frame));
    self.frames.insert(frame, (image, sound));

    while self
      .heap
      .peek()
      .is_some_and(|Reverse(frame)| *frame == self.next)
    {
      let Reverse(frame) = self.heap.pop().unwrap();
      assert_eq!(frame, self.next);
      let (image, sound) = self.frames.remove(&frame).unwrap();
      self
        .stdin
        .write_all(image.data())
        .context(error::RecordingWrite)?;
      self.audio.push(sound);
      self.next += 1;
    }

    Ok(())
  }

  pub(crate) fn new(options: &Options, size: Vector2<NonZeroU32>, fps: Fps) -> Result<Self> {
    let (tempdir, tempdir_path) = tempdir()?;

    let mut encoder = Command::new("ffmpeg")
      .args(["-f", "rawvideo"])
      .args(["-pixel_format", "rgba"])
      .args(["-video_size", &format!("{}x{}", size.x, size.y)])
      .args(["-framerate", &fps.to_string()])
      .args(["-i", "-"])
      .args(["-c:v", "libx264"])
      .args(["-crf", "18"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-preset", "slow"])
      .arg(VIDEO)
      .current_dir(&tempdir_path)
      .stdin(Stdio::piped())
      .stderr(options.stdio())
      .stdout(options.stdio())
      .spawn()
      .context(error::RecordingInvoke)?;

    let stdin = BufWriter::new(encoder.stdin.take().unwrap());

    Ok(Self {
      audio: Vec::new(),
      encoder,
      end: None,
      frames: HashMap::new(),
      heap: BinaryHeap::new(),
      next: 0,
      size,
      stdin,
      tempdir,
      tempdir_path,
    })
  }

  fn process_output(options: &Options, output: &process::Output) -> Result {
    if output.status.success() {
      return Ok(());
    }

    if !options.verbose {
      eprintln!("{}", String::from_utf8_lossy(&output.stdout));
      eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Err(
      error::RecordingStatus {
        status: output.status,
      }
      .build(),
    )
  }
}
