use super::*;

pub(crate) struct Recorder {
  sounds: Vec<(u64, Sound)>,
  #[allow(unused)]
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  pub(crate) fn frame(&mut self, frame: u64, image: Image, sound: Sound) -> Result {
    let path = self.tempdir_path.join(format!("{frame}.png"));
    log::trace!("saving frame to {path}");
    image.save(&path)?;
    self.sounds.push((frame, sound));
    Ok(())
  }

  pub(crate) fn new() -> Result<Self> {
    let tempdir = TempDir::new().context(error::TempdirIo)?;
    let tempdir_path = tempdir.path().into_utf8_path()?.into();
    Ok(Self {
      sounds: Vec::new(),
      tempdir,
      tempdir_path,
    })
  }

  pub(crate) fn save(&mut self, options: &Options) -> Result {
    const CONCAT: &str = "concat.txt";

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.sounds.len(),
    );

    if self.sounds.is_empty() {
      return Ok(());
    }

    self.sounds.sort_by_key(|(frame, _sound)| *frame);

    {
      let mut concat = "ffconcat version 1.0\n".to_owned();
      for (frame, sound) in &self.sounds {
        let duration = sound.duration_micros();
        if duration == 0 {
          log::warn!("frame {frame} has duration 0");
        }
        concat.push_str(&format!(
          "file {frame}.png\noption framerate 1000000\nduration {duration}us\n",
        ));
      }

      let path = self.tempdir_path.join(CONCAT);
      fs::write(&path, concat).context(error::FilesystemIo { path })?;
    }

    Sound::save(
      &self.tempdir_path.join(AUDIO),
      self.sounds.iter().map(|(_frame, sound)| sound),
    )?;

    let output = Command::new("ffmpeg")
      .args(["-safe", "0"])
      .args(["-i", CONCAT])
      .args(["-i", AUDIO])
      .args(["-c:v", "libx264"])
      .args(["-crf", "18"])
      .args(["-fps_mode:v", "passthrough"])
      .args(["-movflags", "+faststart"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-preset", "slow"])
      .args(["-video_track_timescale", "1000000"])
      .args(["-c:a", "aac"])
      .arg(RECORDING)
      .current_dir(&self.tempdir_path)
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
      .output()
      .context(error::RecordingInvoke)?;

    if !output.status.success() {
      if !options.verbose {
        eprintln!("{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
      }

      return Err(
        error::RecordingStatus {
          status: output.status,
        }
        .build(),
      );
    }

    fs::rename(self.tempdir_path.join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
