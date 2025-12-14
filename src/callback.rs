use super::*;

pub(crate) trait Callback: FnMut(&mut State, f32) {
  fn clone_box(&self) -> Box<dyn Callback>;
}

impl<F> Callback for F
where
  F: FnMut(&mut State, f32) + Clone + 'static,
{
  fn clone_box(&self) -> Box<dyn Callback> {
    Box::new(self.clone())
  }
}

impl Clone for Box<dyn Callback> {
  fn clone(&self) -> Self {
    self.clone_box()
  }
}
