use {
  super::*,
  std::process::{Child, ChildStdin, Command},
};

const VIDEO: &str = "video.mp4";

pub(crate) struct Recorder {
  audio: Vec<Sound>,
  end: Option<u64>,
  ffmpeg: Child,
  frames: HashMap<u64, (Image, Sound)>,
  heap: BinaryHeap<Reverse<u64>>,
  next: u64,
  resolution: NonZeroU32,
  stdin: BufWriter<ChildStdin>,
  #[allow(unused)]
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  pub(crate) fn finish(mut self, options: &Options, config: &Config) -> Result {
    assert!(self.heap.is_empty());

    self.stdin.flush().context(error::CaptureFlush)?;
    drop(self.stdin);

    let output = self.ffmpeg.wait_with_output().context(error::CaptureWait)?;

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

    let path = config.capture("mp4");

    fs::rename(self.tempdir_path.join(RECORDING), &path).context(error::FilesystemIo { path })?;

    Ok(())
  }

  pub(crate) fn frame(&mut self, frame: u64, image: Image, sound: Sound) -> Result {
    if image.width() != self.resolution.get() || image.height() != self.resolution.get() {
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
        .context(error::CaptureWrite)?;
      self.audio.push(sound);
      self.next += 1;
    }

    Ok(())
  }

  pub(crate) fn new(options: &Options, resolution: NonZeroU32, fps: Fps) -> Result<Self> {
    let (tempdir, tempdir_path) = tempdir()?;

    let mut ffmpeg = Command::new("ffmpeg")
      .args(["-f", "rawvideo"])
      .args(["-pixel_format", "rgba"])
      .args(["-video_size", &format!("{resolution}x{resolution}")])
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
      .context(error::CaptureInvoke)?;

    let stdin = BufWriter::new(ffmpeg.stdin.take().unwrap());

    Ok(Self {
      audio: Vec::new(),
      end: None,
      ffmpeg,
      frames: HashMap::new(),
      heap: BinaryHeap::new(),
      next: 0,
      resolution,
      stdin,
      tempdir,
      tempdir_path,
    })
  }
}
