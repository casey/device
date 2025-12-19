use super::*;

pub(crate) struct Track {
  pub(crate) audio: Arc<Wave>,
  pub(crate) tempo: Tempo,
}
