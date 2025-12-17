use super::*;

#[derive(Clone, Copy, Debug, Default, ValueEnum, IntoStaticStr)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum ImageFormat {
  Bgra8Unorm,
  #[default]
  Bgra8UnormSrgb,
}

impl ImageFormat {
  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for ImageFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.write_str(self.name())
  }
}

impl ImageFormat {
  pub(crate) fn swizzle(self, src: [u8; COLOR_CHANNELS], dst: &mut [u8; COLOR_CHANNELS]) {
    match self {
      Self::Bgra8Unorm | Self::Bgra8UnormSrgb => {
        let [b, g, r, a] = src;
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
