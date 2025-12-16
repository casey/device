use super::*;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub(crate) enum ImageFormat {
  #[value(name("bgra8unorm"))]
  Bgra8Unorm,
  #[default]
  #[value(name("bgra8unorm-srgb"))]
  Bgra8UnormSrgb,
}

impl Display for ImageFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Bgra8Unorm => write!(f, "bgra8unorm"),
      Self::Bgra8UnormSrgb => write!(f, "bgra8unorm-srgb"),
    }
  }
}

impl ImageFormat {
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

impl TryFrom<TextureFormat> for ImageFormat {
  type Error = Error;

  fn try_from(texture_format: TextureFormat) -> Result<Self> {
    match texture_format {
      TextureFormat::Bgra8Unorm => Ok(Self::Bgra8Unorm),
      TextureFormat::Bgra8UnormSrgb => Ok(Self::Bgra8UnormSrgb),
      _ => Err(error::UnsupportedTextureFormat { texture_format }.build()),
    }
  }
}

impl From<ImageFormat> for TextureFormat {
  fn from(format: ImageFormat) -> Self {
    match format {
      ImageFormat::Bgra8Unorm => Self::Bgra8Unorm,
      ImageFormat::Bgra8UnormSrgb => Self::Bgra8UnormSrgb,
    }
  }
}
