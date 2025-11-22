use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fps {
  duration: Duration,
  fps: NonZeroU32,
}

impl Fps {
  pub(crate) fn duration(self) -> Duration {
    self.duration
  }

  pub(crate) fn fps(self) -> NonZeroU32 {
    self.fps
  }

  pub(crate) fn spf(self, sample_rate: u32) -> Result<u32> {
    if !sample_rate.is_multiple_of(self.fps.get()) {
      return Err(
        error::SamplesPerFrame {
          fps: self,
          sample_rate,
        }
        .build(),
      );
    }

    Ok(sample_rate / self.fps)
  }
}

impl FromStr for Fps {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, String> {
    s.parse::<NonZeroU32>()
      .map_err(|err| format!("failed to parse fps: {err}"))?
      .try_into()
  }
}

impl TryFrom<NonZeroU32> for Fps {
  type Error = String;

  fn try_from(fps: NonZeroU32) -> Result<Self, String> {
    let duration = Duration::try_from_secs_f64(1.0 / fps.get() as f64)
      .map_err(|err| format!("failed to calculate fps duration: {err}"))?;

    Ok(Self { duration, fps })
  }
}

impl Display for Fps {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.fps)
  }
}
