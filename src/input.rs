use super::*;

pub(crate) struct Input {
  format: SoundFormat,
  queue: Arc<Mutex<Vec<f32>>>,
  #[allow(unused)]
  stream: Stream,
}

impl Input {
  pub(crate) fn drain(&self) -> Sound {
    Sound::new(self.format, self.queue.lock().unwrap().drain(..).collect())
  }

  pub(crate) fn new(
    device: cpal::Device,
    supported_stream_config: SupportedStreamConfig,
  ) -> Result<Self> {
    let mut stream_config = supported_stream_config.config();

    stream_config.buffer_size = match supported_stream_config.buffer_size() {
      SupportedBufferSize::Range { min, max } => {
        BufferSize::Fixed(DEFAULT_BUFFER_SIZE.clamp(*min, *max))
      }
      SupportedBufferSize::Unknown => BufferSize::Default,
    };

    let queue = Arc::new(Mutex::new(Vec::new()));

    let stream = device
      .build_input_stream(
        &stream_config,
        {
          let queue = queue.clone();
          move |data: &[f32], _: &cpal::InputCallbackInfo| {
            queue.lock().unwrap().extend(data);
          }
        },
        move |err| {
          eprintln!("input stream error: {err}");
        },
        None,
      )
      .context(error::AudioBuildInputStream)?;

    stream.play().context(error::AudioPlayStream)?;

    log::info!(
      "input stream opened: {}",
      StreamConfigDisplay(&stream_config),
    );

    Ok(Self {
      format: SoundFormat {
        channels: stream_config.channels,
        sample_rate: stream_config.sample_rate.0,
      },
      queue,
      stream,
    })
  }
}
