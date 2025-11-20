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

  pub(crate) fn save(&mut self) -> Result {
    const AUDIO: &str = "audio.wav";
    const CONCAT: &str = "concat.txt";
    const RECORDING: &str = "recording.mp4";

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.sounds.len(),
    );

    self.sounds.sort_by_key(|(frame, _sound)| *frame);

    let Some((_frame, first)) = self.sounds.first() else {
      return Ok(());
    };

    {
      let mut concat = "ffconcat version 1.0\n".to_owned();
      for (i, (_frame, sound)) in self.sounds.iter().enumerate() {
        concat.push_str(&format!(
          "file {i}.png\noption framerate 1000000\nduration {}us\n",
          sound.duration_micros(),
        ));
      }

      let path = self.tempdir_path.join(CONCAT);
      fs::write(&path, concat).context(error::FilesystemIo { path })?;
    }

    {
      let path = self.tempdir_path.join(AUDIO);
      let mut writer = WavWriter::create(
        &path,
        WavSpec {
          channels: first.channels,
          sample_rate: first.sample_rate,
          bits_per_sample: 32,
          sample_format: hound::SampleFormat::Float,
        },
      )
      .context(error::WavCreate { path: &path })?;

      for (_frame, sound) in &self.sounds {
        for sample in &sound.samples {
          writer
            .write_sample(*sample)
            .context(error::WavWrite { path: &path })?;
        }
      }

      writer.finalize().context(error::WavFinalize { path })?;
    }

    let output = Command::new("ffmpeg")
      .args(["-safe", "0"])
      .args(["-i", CONCAT])
      .args(["-i", AUDIO])
      .args(["-c:v", "libx264"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-fps_mode:v", "passthrough"])
      .args(["-video_track_timescale", "1000000"])
      .args(["-c:a", "aac"])
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

    fs::rename(self.tempdir_path.join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
