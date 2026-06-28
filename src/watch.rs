use notify::{
	Event, RecursiveMode, Result, Watcher,
	event::{EventKind, ModifyKind},
};
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

		for ev in event_rx.into_iter().flatten() {
			if let EventKind::Modify(ModifyKind::Data(_)) = ev.kind {
				let _ = restart_tx.send(());
			}
		}
	});

	Ok(restart_rx)
}
