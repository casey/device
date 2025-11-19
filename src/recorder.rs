use super::*;

pub(crate) struct Recorder {
  frames: Vec<Instant>,
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  pub(crate) fn frame(&mut self, frame: Image, time: Instant) -> Result {
    let path = self.tempdir_path.join(format!("{}.png", self.frames.len()));
    log::trace!("saving frame to {path}");
    frame.save(&path)?;
    self.frames.push(time);
    Ok(())
  }

  pub(crate) fn new() -> Result<Self> {
    let tempdir = TempDir::new().context(error::TempdirIo)?;
    let tempdir_path = tempdir.path().into_utf8_path()?.into();
    Ok(Self {
      frames: Vec::new(),
      tempdir,
      tempdir_path,
    })
  }

  pub(crate) fn save(&self) -> Result {
    const FRAMES: &str = "frames.text";
    const RECORDING: &str = "recording.mp4";

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.frames.len(),
    );

    let mut concat = "ffconcat version 1.0\n".to_owned();
    for (i, time) in self.frames.iter().enumerate() {
      writeln!(&mut concat, "file {i}.png").unwrap();
      writeln!(&mut concat, "option framerate 1000000").unwrap();
      if let Some(next) = self.frames.get(i + 1) {
        writeln!(
          &mut concat,
          "duration {}us",
          next.duration_since(*time).as_micros()
        )
        .unwrap();
      }
    }

    let path = self.tempdir_path.join(FRAMES);
    fs::write(&path, concat).context(error::FilesystemIo { path })?;

    let output = Command::new("ffmpeg")
      .args(["-safe", "0"])
      .args(["-i", FRAMES])
      .args(["-c:v", "libx264"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-fps_mode:v", "passthrough"])
      .args(["-video_track_timescale", "1000000"])
      .arg(RECORDING)
      .current_dir(&self.tempdir_path)
      .output()
      .context(error::RecordingInvoke)?;

    if !output.status.success() {
      eprintln!("{}", String::from_utf8_lossy(&output.stdout));
      eprintln!("{}", String::from_utf8_lossy(&output.stderr));
      return Err(
        error::RecordingStatus {
          status: output.status,
        }
        .build(),
      );
    }

    fs::rename(self.tempdir.path().join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
