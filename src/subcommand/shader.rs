use super::*;

pub(crate) fn run() -> Result {
  let resolution = 1.try_into().unwrap();

  pollster::block_on(Renderer::new(
    None,
    None,
    resolution,
    Size::new(resolution, resolution),
    None,
  ))?
  .render(&Analyzer::new(), &State::default(), None)?;

  print!("{FilterWgsl}");
  Ok(())
}
