use super::*;

pub(crate) struct Recorder {
  sounds: Vec<Sound>,
  tempdir: TempDir,
  tempdir_path: Utf8PathBuf,
}

impl Recorder {
  pub(crate) fn frame(&mut self, frame: Image, sound: Sound) -> Result {
    let index = self.sounds.len();
    let path = self.tempdir_path.join(format!("{index}.png"));
    log::trace!("saving frame to {path}");
    frame.save(&path)?;
    self.sounds.push(sound);
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

  pub(crate) fn save(&self) -> Result {
    const AUDIO: &str = "audio.wav";
    const FRAMES: &str = "frames.text";
    const RECORDING: &str = "recording.mp4";

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.sounds.len(),
    );

    let Some(first) = self.sounds.first() else {
      return Ok(());
    };

    {
      let mut concat = "ffconcat version 1.0\n".to_owned();
      for (i, sound) in self.sounds.iter().enumerate() {
        concat.push_str(&format!(
          "file {i}.png\noption framerate 1000000\nduration {}us\n",
          sound.duration_micros(),
        ));
      }

      let path = self.tempdir_path.join(FRAMES);
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

      for sound in &self.sounds {
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
      .args(["-i", FRAMES])
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

    fs::rename(self.tempdir.path().join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
