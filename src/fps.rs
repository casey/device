use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Fps {
  duration: Duration,
  fps: f32,
}

impl Fps {
  pub(crate) fn duration(self) -> Duration {
    self.duration
  }

  pub(crate) fn fps(self) -> f32 {
    self.fps
  }

  pub(crate) fn spf(self, sample_rate: u32) -> Result<u32> {
    let spf = sample_rate as f32 / self.fps();

    if spf.fract() != 0.0 {
      return Err(
        error::SamplesPerFrame {
          fps: self,
          sample_rate,
          spf,
        }
        .build(),
      );
    }

    // todo: check for errors
    Ok(spf as u32)
  }
}

impl FromStr for Fps {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, String> {
    s.parse::<f32>()
      .map_err(|err| format!("failed to parse fps: {err}"))?
      .try_into()
  }
}

impl TryFrom<f32> for Fps {
  type Error = String;

  fn try_from(fps: f32) -> Result<Self, String> {
    let duration = Duration::try_from_secs_f32(1.0 / fps)
      .map_err(|err| format!("failed to calculate fps duration: {err}"))?;

    Ok(Self { duration, fps })
  }
}

impl Display for Fps {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.fps)
  }
}
