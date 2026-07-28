mod app;
use app::AntbyteApp;

use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
	mpsc::Receiver,
};

use antbyte::world::{World, config::WorldConfig};
use eframe::egui;

const TILE_SIZE: f32 = 28.0;
const PADDING: f32 = 100.0;
const MIN_SIZE: f32 = 720.0;
const MAX_SIZE: f32 = 800.0;

pub fn run_with_watch(
	world: &World,
	watch_rx: Option<Receiver<()>>,
	quiet: bool,
) -> eframe::Result<bool> {
	let WorldConfig { height, width, .. } = *world.config();

	let (height, width) = (height as f32, width as f32);
	let max_dim = height.max(width);
	let tile_size: f32 = TILE_SIZE
		.min((MAX_SIZE - PADDING) / max_dim)
		.clamp(4.0, TILE_SIZE);

	let height = (tile_size * height + PADDING).clamp(MIN_SIZE, MAX_SIZE);
	let width = (tile_size * width).clamp(MIN_SIZE, MAX_SIZE);

	let restart_requested = Arc::new(AtomicBool::new(false));
	let app_restart_requested = restart_requested.clone();

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default()
			.with_icon(app_icon())
			.with_inner_size([width, height]),
		..Default::default()
	};

	let title = if let Some(name) = world.name() {
		format!("{name}   -   ANTBYTE")
	} else {
		"ANTBYTE".into()
	};

	eframe::run_native(
		&title,
		options,
		Box::new(move |_| {
			Ok(Box::new(AntbyteApp::new(
				world.clone(),
				quiet,
				tile_size,
				watch_rx,
				app_restart_requested,
			)))
		}),
	)
	.map(|_| restart_requested.load(Ordering::Relaxed))
}

fn app_icon() -> egui::IconData {
	eframe::icon_data::from_png_bytes(include_bytes!("../../assets/antbyte_logo.png")).unwrap()
}
