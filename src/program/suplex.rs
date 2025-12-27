use super::*;

pub(crate) fn callback(config: &Config) -> Result<Box<dyn Callback>> {
  let path = &config.find_image(r"nichijou-principal-german-suplex-deer")?;

  let reader = BufReader::new(File::open(path).context(error::FilesystemIo { path })?);
  let decoder = GifDecoder::new(reader).context(error::ImageDecode { path })?;

  let mut media = Vec::new();

  for frame in decoder.into_frames() {
    let frame = frame.context(error::ImageDecode { path })?;
    let buffer = frame.into_buffer();
    let width = buffer.width();
    let height = buffer.height();

    let image = ImageData {
      alpha_type: peniko::ImageAlphaType::Alpha,
      data: buffer.into_vec().into(),
      format: peniko::ImageFormat::Rgba8,
      height,
      width,
    };

    media.push(Media::new().image(image).handle());
  }

  let mut index = 0;

  Ok(Box::new(move |state: &mut State, tick: Tick| {
    if state.filters.is_empty() {
      state.filters.push(Filter {
        blend_mode: BlendMode::Source,
        media: Some(media[0].clone()),
        ..default()
      });
    }

    let Some(position) = tick.position else {
      return;
    };

    let tempo = tick.tempo.unwrap();

    if position < bbq(19, 1, 1) {
      if tick.advanced() && position.is_phrase() {
        state.filters[0].media = Some(media[index % media.len()].clone());
        index += 1;
      }
    } else if position < bbq(23, 1, 1) {
    } else if position < bbq(24, 1, 1) {
      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      let frame = index + (tempo.bars(tick.time).fract() * (media.len() - index) as f64) as usize;
      state.filters[0].media = Some(media[frame.min(media.len() - 1)].clone());
    } else {
      #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
      let frame = (tempo.bars(tick.time).fract() * media.len() as f64) as usize;
      state.filters[0].media = Some(media[frame.min(media.len() - 1)].clone());
    }
  }))
}
