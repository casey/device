use super::*;

static KEY: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(crate) struct MediaHandle {
  key: u64,
  media: Rc<Media>,
}

impl MediaHandle {
  pub(crate) fn key(&self) -> u64 {
    self.key
  }

  pub(crate) fn media(&self) -> &Media {
    &self.media
  }
}

impl From<Media> for MediaHandle {
  fn from(media: Media) -> Self {
    Self {
      key: KEY.fetch_add(1, atomic::Ordering::Relaxed),
      media: Rc::new(media),
    }
  }
}
