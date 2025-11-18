use {
  super::*,
  png::{BitDepth, ColorType, Compression, Decoder, Encoder},
};

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Image {
  data: Vec<u8>,
  height: u32,
  width: u32,
}

const OPAQUE: u8 = u8::MAX;

impl Image {
  pub(crate) fn data_mut(&mut self) -> &mut [u8] {
    &mut self.data
  }

  #[allow(unused)]
  pub(crate) fn load(path: &Utf8Path) -> Result<Self> {
    let decoder = Decoder::new(BufReader::new(
      File::open(path).context(error::FilesystemIo { path })?,
    ));

    let mut reader = decoder.read_info().context(error::PngDecode { path })?;

    let mut buffer = vec![
      0;
      reader
        .output_buffer_size()
        .context(error::PngDecodeSize { path })?
    ];

    let info = reader
      .next_frame(&mut buffer)
      .context(error::PngDecode { path })?;

    let bytes = &buffer[..info.buffer_size()];

    let data = match (info.color_type, info.bit_depth) {
      (ColorType::Grayscale, BitDepth::One) => {
        let width = info.width.into_usize();
        let height = info.height.into_usize();
        let stride = width.div_ceil(8);

        let mut data = Vec::with_capacity(width * height * 4);

        for y in 0..height {
          for x in 0..width {
            let byte = y * stride + x / 8;
            let bit = 7 - (x % 8);
            let value = if bytes[byte] & (1 << bit) == 0 {
              0
            } else {
              u8::MAX
            };

            data.extend([value, value, value, OPAQUE]);
          }
        }

        data
      }
      (ColorType::Grayscale, BitDepth::Eight) => bytes
        .iter()
        .copied()
        .flat_map(|value| [value, value, value, OPAQUE])
        .collect(),
      (ColorType::GrayscaleAlpha, BitDepth::Eight) => bytes
        .chunks(2)
        .flat_map(|pixel| {
          let [value, alpha] = pixel.try_into().unwrap();
          [value, value, value, alpha]
        })
        .collect(),
      (ColorType::Rgb, BitDepth::Eight) => bytes
        .chunks(3)
        .flat_map(|pixel| {
          let [r, g, b] = pixel.try_into().unwrap();
          [r, g, b, OPAQUE]
        })
        .collect(),
      (ColorType::Rgba, BitDepth::Eight) => bytes.into(),
      (color_type, bit_depth) => {
        return Err(
          error::PngDecodeFormat {
            bit_depth,
            color_type,
            path,
          }
          .build(),
        );
      }
    };

    Ok(Self {
      data,
      height: info.height,
      width: info.width,
    })
  }

  pub(crate) fn resize(&mut self, width: u32, height: u32) {
    self.height = height;
    self.width = width;
    self.data.resize((width * height * 4).into_usize(), 0);
  }

  pub(crate) fn save(&self, path: &Utf8Path) -> Result {
    let file = File::create(path).context(error::FilesystemIo { path })?;

    let writer = BufWriter::new(file);

    let mut alpha = false;
    let mut color = false;
    let mut continuous = false;
    for chunk in self.data.chunks(4) {
      let chunk: [u8; 4] = chunk.try_into().unwrap();
      let [r, g, b, a] = chunk;

      if a != OPAQUE {
        alpha = true;
      }

      if r != g || r != b {
        color = true;
      }

      for channel in chunk {
        if channel > 0 && channel < u8::MAX {
          continuous = true;
        }
      }
    }

    let color_type = match (color, alpha) {
      (false, false) => ColorType::Grayscale,
      (false, true) => ColorType::GrayscaleAlpha,
      (true, false) => ColorType::Rgb,
      (true, true) => ColorType::Rgba,
    };

    let mut encoder = Encoder::new(writer, self.width, self.height);
    encoder.set_color(color_type);
    encoder.set_compression(Compression::High);

    let data = if !continuous && !alpha {
      assert!(!color);
      assert_eq!(color_type, ColorType::Grayscale);

      encoder.set_depth(BitDepth::One);

      let width = self.width.into_usize();
      let height = self.height.into_usize();
      let stride = width.div_ceil(8);
      let mut data = vec![0; stride * height];

      for (index, chunk) in self.data.chunks(4).enumerate() {
        let value = chunk[0];

        assert_eq!(chunk.len(), 4);
        assert!(value == 0 || value == u8::MAX);

        if value == u8::MAX {
          let x = index % width;
          let y = index / width;
          let byte = y * stride + x / 8;
          let bit = 7 - (x % 8);
          data[byte] |= 1 << bit;
        }
      }

      Cow::Owned(data)
    } else {
      match color_type {
        ColorType::Grayscale => Cow::Owned(
          self
            .data
            .chunks(4)
            .map(|chunk| chunk[0])
            .collect::<Vec<u8>>(),
        ),
        ColorType::GrayscaleAlpha => Cow::Owned(
          self
            .data
            .chunks(4)
            .flat_map(|chunk| [chunk[0], chunk[3]])
            .collect::<Vec<u8>>(),
        ),
        ColorType::Rgb => Cow::Owned(
          self
            .data
            .chunks(4)
            .flat_map(|chunk| &chunk[0..3])
            .copied()
            .collect::<Vec<u8>>(),
        ),
        ColorType::Rgba => Cow::Borrowed(&self.data),
        ColorType::Indexed => unreachable!(),
      }
    };

    let mut writer = encoder.write_header().context(error::PngEncode { path })?;

    writer
      .write_image_data(&data)
      .context(error::PngEncode { path })?;

    writer.finish().context(error::PngEncode { path })?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn color_type_reduction() {
    #[track_caller]
    fn case(
      dir: &Utf8Path,
      data: &[u8],
      color_type: ColorType,
      bit_depth: BitDepth,
      expected: &[u8],
    ) {
      let image = Image {
        data: data.into(),
        height: 1,
        width: 2,
      };

      let path = dir.join("image.png");

      image.save(&path).unwrap();

      let decoder = Decoder::new(BufReader::new(File::open(&path).unwrap()));
      let mut reader = decoder.read_info().unwrap();
      let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
      let info = reader.next_frame(&mut buffer).unwrap();
      assert_eq!(info.color_type, color_type);
      assert_eq!(info.bit_depth, bit_depth);
      let bytes = &buffer[..info.buffer_size()];
      assert_eq!(bytes, expected);
      assert_eq!(info.height, image.height);
      assert_eq!(info.width, image.width);

      let loaded = Image::load(&path).unwrap();
      assert_eq!(loaded, image);
    }

    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path().into_utf8_path().unwrap();

    case(
      path,
      &[0, 0, 0, 255, 255, 255, 255, 255],
      ColorType::Grayscale,
      BitDepth::One,
      &[0b0100_0000],
    );

    case(
      path,
      &[0, 0, 0, 255, 127, 127, 127, 255],
      ColorType::Grayscale,
      BitDepth::Eight,
      &[0, 127],
    );

    case(
      path,
      &[0, 0, 0, 255, 255, 255, 255, 127],
      ColorType::GrayscaleAlpha,
      BitDepth::Eight,
      &[0, 255, 255, 127],
    );

    case(
      path,
      &[0, 0, 0, 255, 0, 127, 255, 255],
      ColorType::Rgb,
      BitDepth::Eight,
      &[0, 0, 0, 0, 127, 255],
    );

    case(
      path,
      &[0, 0, 0, 255, 0, 127, 255, 127],
      ColorType::Rgba,
      BitDepth::Eight,
      &[0, 0, 0, 255, 0, 127, 255, 127],
    );
  }
}
