use notify::{Event, RecursiveMode, Result, Watcher};
use std::{path::PathBuf, sync::mpsc, thread};

pub fn watch(path: PathBuf) -> Result<mpsc::Receiver<()>> {
	let (restart_tx, restart_rx) = mpsc::channel();

	thread::spawn(move || {
		let (event_tx, event_rx) = mpsc::channel::<Result<Event>>();
		let mut watcher = match notify::recommended_watcher(event_tx) {
			Ok(watcher) => watcher,
			Err(_) => return,
		};

		if watcher.watch(&path, RecursiveMode::NonRecursive).is_err() {
			return;
		}

		for event in event_rx {
			if event.is_ok() {
				let _ = restart_tx.send(());
			}
		}
	});

	Ok(restart_rx)
}
