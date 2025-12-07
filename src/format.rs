use super::*;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum Format {
  Bgra8Unorm,
  #[default]
  Bgra8UnormSrgb,
}

impl Display for Format {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Bgra8Unorm => write!(f, "bgra8-unorm"),
      Self::Bgra8UnormSrgb => write!(f, "bgra8-unorm-srgb"),
    }
  }
}

impl Format {
  pub(crate) fn swizzle(self, src: &[u8], dst: &mut [u8]) {
    match self {
      Self::Bgra8Unorm | Self::Bgra8UnormSrgb => {
        let [b, g, r, a] = src.try_into().unwrap();
        let dst = <&mut [u8; 4]>::try_from(dst).unwrap();
        *dst = [r, g, b, a];
      }
    }
  }
}

impl TryFrom<TextureFormat> for Format {
  type Error = Error;

  fn try_from(texture_format: TextureFormat) -> Result<Self> {
    match texture_format {
      TextureFormat::Bgra8Unorm => Ok(Self::Bgra8Unorm),
      TextureFormat::Bgra8UnormSrgb => Ok(Self::Bgra8UnormSrgb),
      _ => Err(error::UnsupportedTextureFormat { texture_format }.build()),
    }
  }
}

impl From<Format> for TextureFormat {
  fn from(format: Format) -> Self {
    match format {
      Format::Bgra8Unorm => Self::Bgra8Unorm,
      Format::Bgra8UnormSrgb => Self::Bgra8UnormSrgb,
    }
  }
}
