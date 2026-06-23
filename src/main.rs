use antbyte::util::print_error;
use anyhow::Result;
mod ui;

fn main() {
	run().unwrap_or_else(|e| {
		print_error(e);
		std::process::exit(1);
	});
}

fn run() -> Result<()> {
	if let Some((world, _args)) = antbyte::cli::create_world()? {
		ui::run(&world)?;
	}

	Ok(())
}
