use super::*;

pub(crate) trait Callback: FnMut(&mut State, f32) {
  fn clone_box(&self) -> Box<dyn Callback>;
}

impl<T> Callback for T
where
  T: FnMut(&mut State, f32) + Clone + 'static,
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
