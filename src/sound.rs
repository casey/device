use super::*;

#[derive(Clone)]
pub(crate) struct Sound {
  format: SoundFormat,
  samples: Vec<f32>,
}

impl Sound {
  pub(crate) fn append(&mut self, mut other: Self) -> Result<(), String> {
    if other.format != self.format {
      return Err(format!(
        "format changed from {} to {}",
        self.format, other.format,
      ));
    }

    self.samples.append(&mut other.samples);

    Ok(())
  }

  pub(crate) fn downmix(&self) -> impl Iterator<Item = f32> {
    self
      .samples
      .chunks(self.format.channels.into())
      .map(|chunk| chunk.iter().sum::<f32>() / self.format.channels as f32)
  }

  pub(crate) fn duration(&self) -> Duration {
    const NANOS_PER_SECOND: u128 = 1_000_000_000;
    let nanoseconds =
      self.frames().into_u128() * NANOS_PER_SECOND / u128::from(self.format.sample_rate);
    Duration::new(
      (nanoseconds / NANOS_PER_SECOND).try_into().unwrap(),
      (nanoseconds % NANOS_PER_SECOND).try_into().unwrap(),
    )
  }

  pub(crate) fn empty(format: SoundFormat) -> Self {
    Self::new(format, Vec::new())
  }

  pub(crate) fn format(&self) -> SoundFormat {
    self.format
  }

  pub(crate) fn frames(&self) -> usize {
    self.samples.len() / self.format.channels.into_usize()
  }

  pub(crate) fn new(format: SoundFormat, samples: Vec<f32>) -> Self {
    assert!(samples.len().is_multiple_of(format.channels.into()));
    Self { format, samples }
  }

  pub(crate) fn samples(&self) -> usize {
    self.samples.len()
  }

  pub(crate) fn save(&self, path: &Utf8Path) -> Result {
    let mut writer = WavWriter::create(
      path,
      WavSpec {
        channels: self.format.channels,
        sample_rate: self.format.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
      },
    )
    .context(error::WaveCreate { path })?;

    for sample in &self.samples {
      writer
        .write_sample(*sample)
        .context(error::WaveWrite { path })?;
    }

    writer.finalize().context(error::WaveFinalize { path })?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duration() {
    let format = SoundFormat {
      channels: 2,
      sample_rate: 10,
    };
    assert_eq!(Sound::empty(format).duration(), Duration::from_secs(0));
    assert_eq!(
      Sound::new(format, vec![0.0; 10]).duration(),
      Duration::from_millis(500)
    );
    assert_eq!(
      Sound::new(format, vec![0.0; 20]).duration(),
      Duration::from_secs(1)
    );
  }
}
