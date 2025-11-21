enum Error {
  Fractional,
  Infinite,
  Nan,
  Negative,
  Range,
}

trait TryIntoU32 {
  fn try_into_u32(self) -> Result<u32, Error>;
}

impl TryIntoU32 for f32 {
  fn try_into_u32(self) -> Result<u32, Error> {
    if self.is_nan() {
      return Err(Error::Nan);
    }

    if !self.is_finite() {
      return Err(Error::Infinite);
    }

    if self < 0.0 {
      return Err(Error::Negative);
    }

    let truncated = self.trunc();

    if truncated != self {
      return Err(Error::Fractional);
    }

    let integer = self as u32;

    if integer as f32 != truncated {
      return Err(Error::Range);
    }

    Ok(integer)
  }
}
