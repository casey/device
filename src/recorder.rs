use super::*;

pub(crate) struct Recorder {
  frames: Vec<(Instant, usize)>,
  tempdir: TempDir,
}

impl Recorder {
  pub(crate) fn frame(&mut self, frame: Image, time: Instant, samples: usize) -> Result {
    let path = self
      .tempdir
      .path()
      .join(format!("{}.png", self.frames.len()));
    log::trace!("saving frame to {}", path.display());
    frame.save(&path)?;
    self.frames.push((time, samples));
    Ok(())
  }

  pub(crate) fn new() -> Result<Self> {
    Ok(Self {
      frames: Vec::new(),
      tempdir: TempDir::new().context(error::TempdirIo)?,
    })
  }

  pub(crate) fn save(&self, analyzer: &Analyzer) -> Result {
    const AUDIO: &str = "audio.wav";
    const FRAMES: &str = "frames.txt";
    const RECORDING: &str = "recording.mp4";

    let tempdir = self.tempdir.path();

    for (channels, samples) in analyzer.history() {
      let spec = hound::WavSpec {
        bits_per_sample: 32,
        channels: *channels,
        sample_format: hound::SampleFormat::Float,
        // todo: don't assume this frame rate
        sample_rate: 48000,
      };

      let mut writer = hound::WavWriter::create(tempdir.join(AUDIO), spec).unwrap();
      for sample in samples {
        writer.write_sample(*sample).unwrap();
      }
      writer.finalize().unwrap();
    }

    log::info!(
      "saving {} frame recording to {RECORDING}",
      self.frames.len(),
    );

    let mut concat = "ffconcat version 1.0\n".to_owned();
    let mut last = 0;
    for (i, (time, samples)) in self.frames.iter().enumerate() {
      writeln!(&mut concat, "file {i}.png").unwrap();
      writeln!(&mut concat, "option framerate 1000000").unwrap();
      writeln!(
        &mut concat,
        "duration {:0.6}",
        (*samples - last) as f64 / 48000.0
      )
      .unwrap();
      last = *samples;
      // if let Some((next, _samples)) = self.frames.get(i + 1) {
      //   writeln!(
      //     &mut concat,
      //     "duration {}us",
      //     next.duration_since(*time).as_micros()
      //   )
      //   .unwrap();
      // }
    }

    eprintln!("{concat}");

    let path = tempdir.join(FRAMES);
    fs::write(&path, concat).context(error::FilesystemIo { path })?;

    let output = Command::new("ffmpeg")
      .args(["-safe", "0"])
      .args(["-i", FRAMES])
      .args(["-i", AUDIO])
      .args(["-c:v", "libx264"])
      .args(["-pix_fmt", "yuv420p"])
      .args(["-fps_mode", "passthrough"])
      .args(["-video_track_timescale", "1000000"])
      .args(["-c:a", "aac"])
      .args(["-b:a", "192k"])
      .arg(RECORDING)
      .current_dir(tempdir)
      .output()
      .context(error::RecordingInvoke)?;

    eprintln!("{}", String::from_utf8_lossy(&output.stdout));
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));

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

    fs::rename(tempdir.join(RECORDING), RECORDING)
      .context(error::FilesystemIo { path: RECORDING })?;

    Ok(())
  }
}
