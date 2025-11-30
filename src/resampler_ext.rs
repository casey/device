use rubato::{Resampler, Sample};

pub(crate) trait ResamplerExt<T> {
  fn process_all<V: AsRef<[T]>>(
    &mut self,
    wave_in: &[V],
    active_channels_mask: Option<&[bool]>,
    resample_ratio: f64,
  ) -> rubato::ResampleResult<Vec<Vec<T>>>;
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
impl<T: Sample, R: Resampler<T>> ResamplerExt<T> for R {
  fn process_all<V: AsRef<[T]>>(
    &mut self,
    wave_in: &[V],
    active_channels_mask: Option<&[bool]>,
    resample_ratio: f64,
  ) -> rubato::ResampleResult<Vec<Vec<T>>> {
    if wave_in.is_empty() {
      return Ok(Vec::new());
    }

    let new_length = (wave_in[0].as_ref().len() as f64 * resample_ratio) as usize;

    let mut output = vec![Vec::<T>::with_capacity(new_length); wave_in.len()];

    let mut input_buffer = (0..wave_in.len())
      .map(|channel| wave_in[channel].as_ref())
      .collect::<Vec<&[T]>>();

    let mut output_buffer = self.output_buffer_allocate(true);

    let delay = self.output_delay();

    let mut total = 0;

    let mut append = |output_channels: &mut [Vec<T>], output_buffer: &[Vec<T>], produced: usize| {
      // skip samples up to delay
      let skip = delay.saturating_sub(total).min(produced);
      for (output, buffer) in output_channels.iter_mut().zip(output_buffer) {
        output.extend_from_slice(&buffer[skip..produced]);
      }
      total += produced;
    };

    while input_buffer[0].len() >= self.input_frames_next() {
      let (consumed, produced) =
        self.process_into_buffer(&input_buffer, &mut output_buffer, active_channels_mask)?;

      for channel in &mut input_buffer {
        *channel = &channel[consumed..];
      }

      append(&mut output, &output_buffer, produced);
    }

    if !input_buffer[0].is_empty() {
      let (_consumed, produced) = self.process_partial_into_buffer(
        Some(&input_buffer),
        &mut output_buffer,
        active_channels_mask,
      )?;

      append(&mut output, &output_buffer, produced);
    }

    while output[0].len() < new_length {
      let (_consumed, produced) =
        self.process_partial_into_buffer(None::<&[V]>, &mut output_buffer, active_channels_mask)?;

      append(&mut output, &output_buffer, produced);

      if produced == 0 {
        break;
      }
    }

    for channel in &mut output {
      channel.truncate(new_length);
    }

    Ok(output)
  }
}
