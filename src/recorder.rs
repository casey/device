use {
  super::*,
  std::process::{Child, ChildStdin, Command},
};

const VIDEO: &str = "video.mp4";

pub(crate) struct Recorder {
  audio: Sound,
  encoder: Child,
  end: Option<u64>,
  fps: Fps,
  frames: HashMap<u64, (Image, Sound)>,
  frames_encoded: u64,
  heap: BinaryHeap<Reverse<u64>>,
  size: Size,
  stdin: BufWriter<ChildStdin>,
  #[allow(unused)]
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  fn encoders() -> Result<BTreeSet<String>> {
    let output = Command::new("ffmpeg")
      .args(["-hide_banner", "-encoders"])
      .output()
      .context(error::RecordingInvoke)?;

    Self::process_output(&output)?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut encoders = BTreeSet::new();

    let mut body = false;
    for line in stdout.lines() {
      let line = line.trim();

      if line == "------" {
        body = true;
        continue;
      }

      if line.is_empty() || !body {
        continue;
      }

      if let Some(encoder) = line.split_whitespace().nth(1) {
        encoders.insert(encoder.into());
      }
    }

    Ok(encoders)
  }

  pub(crate) fn finish(mut self, options: &Options, config: &Config) -> Result {
    assert!(self.heap.is_empty());

    let frame_imbalance = self.frame_imbalance();

    if frame_imbalance.abs() > 0.0 {
      log::warn!("frame imbalance: {frame_imbalance:+.2}");
    }

    self.stdin.flush().context(error::RecordingFlush)?;
    drop(self.stdin);

    let output = self
      .encoder
      .wait_with_output()
      .context(error::RecordingWait)?;

    Self::process_output(&output)?;

    self.audio.save(&self.tempdir_path.join(AUDIO))?;

    let output = Command::new("ffmpeg")
      .arg("-hide_banner")
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

    Self::process_output(&output)?;

    let path = config.capture("mp4");

    fs::rename(self.tempdir_path.join(RECORDING), &path).context(error::FilesystemIo { path })?;

    Ok(())
  }

  pub(crate) fn frame(&mut self, frame: u64, image: Image, sound: Sound) -> Result {
    let change = if image.width() != self.size.x.get() || image.height() != self.size.y.get() {
      log::warn!("recording resolution changed");
      true
    } else if sound.format() != self.audio.format() {
      log::warn!("sound format changed");
      true
    } else {
      false
    };

    if change {
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
      .is_some_and(|Reverse(frame)| *frame == self.frames_encoded)
    {
      let Reverse(frame) = self.heap.pop().unwrap();
      assert_eq!(frame, self.frames_encoded);
      let (image, sound) = self.frames.remove(&frame).unwrap();
      self
        .stdin
        .write_all(image.data())
        .context(error::RecordingWrite)?;
      self.audio.append(sound).unwrap();
      self.frames_encoded += 1;
    }

    if self.heap.len() > 1 {
      log::warn!("pending frames: {}", self.heap.len());
    }

    let frame_imbalance = self.frame_imbalance();

    if frame_imbalance.abs() > 1.0 {
      log::warn!("frame imbalance: {frame_imbalance:+.2}");
    }

    Ok(())
  }

  fn frame_imbalance(&self) -> f64 {
    let audio_duration = self.audio.duration();
    let expected_frames = audio_duration.div_duration_f64(self.fps.duration());
    self.frames_encoded as f64 - expected_frames
  }

  pub(crate) fn new(
    fps: Fps,
    options: &Options,
    size: Size,
    sound_format: SoundFormat,
  ) -> Result<Self> {
    let (tempdir, tempdir_path) = tempdir()?;

    let encoders = Self::encoders()?;

    let encoder_options: &[[&str; 2]] = if encoders.contains("h264_videotoolbox") {
      &[
        ["-c:v", "h264_videotoolbox"],
        ["-q:v", "100"],
        ["-realtime", "true"],
      ]
    } else {
      &[["-c:v", "libx264"], ["-crf", "18"], ["-preset", "slow"]]
    };

    let mut encoder = Command::new("ffmpeg")
      .arg("-hide_banner")
      .args(["-f", "rawvideo"])
      .args(["-color_primaries", "bt709"])
      .args(["-color_range", "pc"])
      .args(["-color_trc", "bt709"])
      .args(["-colorspace", "bt709"])
      .args(["-framerate", &fps.to_string()])
      .args(["-pixel_format", "rgba"])
      .args(["-video_size", &format!("{}x{}", size.x, size.y)])
      .args(["-i", "-"])
      .args(encoder_options.iter().flatten())
      .args(["-color_range", "pc"])
      .args(["-colorspace", "bt709"])
      .args(["-color_primaries", "bt709"])
      .args(["-color_trc", "bt709"])
      .args(["-pix_fmt", "yuv420p"])
      .arg(VIDEO)
      .current_dir(&tempdir_path)
      .stdin(Stdio::piped())
      .stderr(options.stdio())
      .stdout(options.stdio())
      .spawn()
      .context(error::RecordingInvoke)?;

    let stdin = BufWriter::new(encoder.stdin.take().unwrap());

    Ok(Self {
      audio: Sound::empty(sound_format),
      encoder,
      end: None,
      fps,
      frames: HashMap::new(),
      frames_encoded: 0,
      heap: BinaryHeap::new(),
      size,
      stdin,
      tempdir,
      tempdir_path,
    })
  }

  fn process_output(output: &process::Output) -> Result {
    if output.status.success() {
      return Ok(());
    }

    if !output.stdout.is_empty() {
      eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
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

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]
  #[test]
  fn encoders() {
    let encoders = Recorder::encoders().unwrap();
    assert!(encoders.contains("h264_videotoolbox"), "{encoders:?}");
  }
}
