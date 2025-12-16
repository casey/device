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
        self.format, other.format
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

  pub(crate) fn frames(&self) -> usize {
    self.samples.len() / self.format.channels.into_usize()
  }

  pub(crate) fn empty(format: SoundFormat) -> Self {
    Self {
      format,
      samples: Vec::new(),
    }
  }

  pub(crate) fn new(channels: u16, sample_rate: u32, samples: Vec<f32>) -> Self {
    Self {
      samples,
      format: SoundFormat {
        channels,
        sample_rate,
      },
    }
  }

  pub(crate) fn format(&self) -> SoundFormat {
    self.format
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
