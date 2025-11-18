use super::*;

pub(crate) fn run(options: Options) -> Result {
  let mut app = App::new(options)?;

  EventLoop::with_user_event()
    .build()
    .context(error::EventLoopBuild)?
    .run_app(&mut app)
    .context(error::RunApp)?;

  if let Some(err) = app.errors().into_iter().next() {
    return Err(err);
  }

  Ok(())
}
