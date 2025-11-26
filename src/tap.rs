use super::*;

// todo:
// - return None from emitter so it can be dropped or pruned
// - figure out which notes each key should be
// - remove dc component from brown noise
// - pause input?
// - request input/output with
//   - f32 samples
//   - 48khz
//   - stereo
//   - 128 clamped buffer size
//
// - not getting output
//   - store Option<Synthesizer> when output is a synth
//   - instead of adding to mixer, add to synth
//   - can only play output when output is synthesizer, no playing over input or songs
//   - adding to synth makes the most sense, it already holds a voice
//   - can instead hold a vec voices
//   - voices/synth/patches don't actually have sample rates, so can mix into anything
//   - alternative is some kind of mixer for streams
//   - mixer for streams would let me do more complex dj stuff
//     - add song
//     - remove song
//     - play sound
//   - probably more complicated though
//
// - convert all sources to 48khz f32
// - use 48khz sample rate output and input where possible
// - handle my own tappable mixer
// - remove individual sample saving implementations
// - however, input samples must make it to the analyzer but not the output
// - convert synth into stereo
// - am i actually minimizing the input buffer size?

// todo:
// - should we produce Item = [f32; 2] or a newtype
//   since we are enforcing stereo?
// - can we get a more accurate playback position instead of assuming
//   that everything which has been drained has been played?

// - we really have two paradigms, and i'm not sure if they can be combined
// - input, which is driven by the rate of production from the microphone
// - output, which is driven by the rate of consumption by the speakers
//
// - instead of trying to unify them in a single mixer type,
//   let's instead unify them in the analyzer
//
// - or we could create an iterator over the input, but it returns 0 when there aren't samples
//
// - Stereo and Mono traits
// - don't use iter<item=f32> too error prone
//
// - add frame count and time to capture
//
// - clean up voice and emitter traits

#[derive(Clone)]
pub(crate) struct Tap(Arc<Mutex<Inner>>);

struct Inner {
  active: Vec<Box<dyn Source + Send>>,
  channels: u16,
  pending: Vec<Box<dyn Source + Send>>,
  sample: u64,
  sample_rate: u32,
  samples: Vec<f32>,
}

impl Tap {
  pub(crate) fn add<T: Source + Send + 'static>(&self, source: T) {
    let mut inner = self.0.lock().unwrap();
    let channels = inner.channels;
    let sample_rate = inner.sample_rate;
    inner
      .pending
      .push(Box::new(rodio::source::UniformSourceIterator::new(
        source,
        channels,
        sample_rate,
      )));
  }

  pub(crate) fn drain(&mut self) -> Sound {
    let mut inner = self.0.lock().unwrap();
    Sound {
      channels: inner.channels,
      sample_rate: inner.sample_rate,
      samples: mem::take(&mut inner.samples),
    }
  }

  pub(crate) fn is_empty(&self) -> bool {
    let inner = self.0.lock().unwrap();
    inner.active.is_empty() && inner.pending.is_empty()
  }

  pub(crate) fn new(channels: u16, sample_rate: u32) -> Self {
    Self(Arc::new(Mutex::new(Inner {
      active: Vec::new(),
      channels,
      pending: Vec::new(),
      sample: 0,
      sample_rate,
      samples: Vec::new(),
    })))
  }
}

impl Source for Tap {
  fn channels(&self) -> u16 {
    self.0.lock().unwrap().channels
  }

  fn current_span_len(&self) -> Option<usize> {
    None
  }

  fn sample_rate(&self) -> u32 {
    self.0.lock().unwrap().sample_rate
  }

  fn total_duration(&self) -> Option<std::time::Duration> {
    None
  }
}

impl Iterator for Tap {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    self.0.lock().unwrap().next()
  }
}

impl Iterator for Inner {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if self.sample.is_multiple_of(self.channels.into()) {
      self.active.append(&mut self.pending);
    }

    let mut sum = 0.0;

    self
      .active
      .retain_mut(|source| source.next().inspect(|sample| sum += sample).is_some());

    self.samples.push(sum);

    self.sample += 1;

    Some(sum)
  }
}
