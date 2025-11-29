use super::*;

#[derive(Clone)]
pub(crate) struct Sound {
  pub(crate) channels: u16,
  pub(crate) sample_rate: u32,
  pub(crate) samples: Vec<f32>,
}

impl Sound {
  pub(crate) fn downmix(&self) -> impl Iterator<Item = f32> {
    self
      .samples
      .chunks(self.channels.into())
      .map(|chunk| chunk.iter().sum::<f32>() / self.channels as f32)
  }

  pub(crate) fn channel(&self, channel: u16) -> impl Iterator<Item = f32> {
    self
      .samples
      .chunks(self.channels.into())
      .map(move |chunk| chunk[channel.into_usize()])
  }

  pub(crate) fn duration_micros(&self) -> u128 {
    if self.channels == 0 || self.sample_rate == 0 {
      return 0;
    }
    self.samples.len().into_u128() / u128::from(self.channels) * 1_000_000
      / u128::from(self.sample_rate)
  }

  pub(crate) fn save<'a>(path: &Utf8Path, mut sounds: impl Iterator<Item = &'a Sound>) -> Result {
    let first = sounds.next();

    let mut writer = WavWriter::create(
      path,
      WavSpec {
        channels: first.map_or(2, |first| first.channels),
        sample_rate: first.map_or(48_000, |first| first.sample_rate),
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
      },
    )
    .context(error::WavCreate { path })?;

    for sound in first.into_iter().chain(sounds) {
      for sample in &sound.samples {
        writer
          .write_sample(*sample)
          .context(error::WavWrite { path })?;
      }
    }

    writer.finalize().context(error::WavFinalize { path })?;

    Ok(())
  }
}
