use {
  super::*,
  blake3::{Hash, Hasher},
  notify::{Event, EventHandler, RecommendedWatcher, RecursiveMode, Watcher},
};

pub(crate) struct Binary {
  state: Arc<Mutex<Option<State>>>,
  #[allow(unused)]
  watcher: RecommendedWatcher,
}

impl Binary {
  pub(crate) fn new(path: &Utf8Path) -> Result<Self> {
    let path = env::current_dir()
      .context(error::CurrentDir)?
      .into_utf8_path()?
      .join(path);

    let name = path.file_name().unwrap().to_owned();

    let parent = path.parent().unwrap().to_owned();

    eprintln!("watching {parent} for {name}");

    let state = Arc::new(Mutex::new(None));

    let handler = Handler {
      path,
      hash: None,
      state: state.clone(),
    };

    let mut watcher = RecommendedWatcher::new(handler, notify::Config::default()).unwrap();

    watcher
      .watch(parent.as_ref(), RecursiveMode::NonRecursive)
      .unwrap();

    Ok(Self { watcher, state })
  }

  pub(crate) fn state(&self) -> Option<State> {
    match self.state.lock() {
      Err(err) => {
        log::error!("binary mutex poisoned: {err}");
        None
      }
      Ok(mut state) => state.take(),
    }
  }
}

struct Handler {
  hash: Option<Hash>,
  path: Utf8PathBuf,
  state: Arc<Mutex<Option<State>>>,
}

impl EventHandler for Handler {
  fn handle_event(&mut self, event: notify::Result<Event>) {
    let event = match event {
      Err(err) => {
        log::error!("watch error: {err}");
        return;
      }
      Ok(event) => event,
    };

    if event.paths.iter().all(|path| *path != self.path) {
      log::info!("ignoring event");
      return;
    }

    if let Some(old) = self.hash {
      let mut hasher = Hasher::new();

      if let Err(err) = hasher.update_mmap_rayon(&self.path) {
        log::error!("failed to hash binary: {err}");
        return;
      }

      let new = hasher.finalize();

      if new == old {
        log::info!("binary did not change");
        return;
      }
    }

    let child = Command::new(&self.path)
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    let mut reader = BufReader::new(child.stdout.unwrap());
    loop {
      let mut line = String::new();
      reader.read_line(&mut line).unwrap();
      eprint!("{line}");
      match serde_json::from_str::<State>(&line) {
        Err(err) => eprintln!("error deserializing state {err}"),
        Ok(state) => *self.state.lock().unwrap() = Some(state),
      }
    }
  }
}
