use antbyte::util::print_error;
use anyhow::Result;

mod ui;
mod watch;

fn main() {
	run().unwrap_or_else(|e| {
		print_error(e);
		std::process::exit(1);
	});
}

fn run() -> Result<()> {
	loop {
		let Some((world, args)) = antbyte::cli::create_world()? else {
			return Ok(());
		};

		let watch_rx = watch::watch(args.path)?;
		if !ui::run_with_watch(&world, Some(watch_rx))? {
			break;
		}
	}

	Ok(())
}
