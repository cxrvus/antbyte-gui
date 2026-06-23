use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
	mpsc::Receiver,
};
use std::time::Duration;

use antbyte::{
	util::vec2::Pos,
	world::{World, config::WorldConfig, frame::FrameOutput},
};
use eframe::{
	App,
	egui::{self, Color32, Pos2, Rect, Sense, Vec2},
};

const TILE_PX: f32 = 10.0;
const PADDING: f32 = 60.0;

pub fn run_with_watch(world: &World, watch_rx: Option<Receiver<()>>) -> eframe::Result<bool> {
	let WorldConfig { height, width, .. } = *world.config();
	let height = TILE_PX * height as f32 + PADDING;
	let width = TILE_PX * width as f32;
	let restart_requested = Arc::new(AtomicBool::new(false));
	let app_restart_requested = restart_requested.clone();

	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size([width, height]),
		..Default::default()
	};

	eframe::run_native(
		"ANTBYTE",
		options,
		Box::new(move |_| {
			Ok(Box::new(AntbyteApp::new(
				world.clone(),
				watch_rx,
				app_restart_requested,
			)))
		}),
	)
	.map(|_| restart_requested.load(Ordering::Relaxed))
}

struct AntbyteApp {
	world: World,
	last_frame: Option<FrameOutput>,
	watch_rx: Option<Receiver<()>>,
	restart_requested: Arc<AtomicBool>,
}

impl AntbyteApp {
	pub fn new(
		world: World,
		watch_rx: Option<Receiver<()>>,
		restart_requested: Arc<AtomicBool>,
	) -> Self {
		Self {
			world,
			last_frame: None,
			watch_rx,
			restart_requested,
		}
	}
}

impl App for AntbyteApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let WorldConfig { height, width, .. } = *self.world.config();

		if self
			.watch_rx
			.as_ref()
			.is_some_and(|rx| rx.try_recv().is_ok())
		{
			self.restart_requested.store(true, Ordering::Relaxed);
			ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
			return;
		}

		let frame = self.world.next_frame_auto();

		if let Some(frame) = frame {
			self.last_frame = Some(frame);
		}

		if let Some(frame) = self.last_frame.as_ref() {
			ui.vertical(|ui| {
				let size = Vec2::new(width as f32 * TILE_PX, height as f32 * TILE_PX);
				let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
				let painter = ui.painter_at(rect);

				for y in 0..height {
					for x in 0..width {
						let value = frame.bg.get(&Pos { x, y }).unwrap_or(&0);
						let color = PALETTE[*value as usize];

						let min = Pos2::new(
							rect.left() + x as f32 * TILE_PX,
							rect.top() + y as f32 * TILE_PX,
						);
						let tile = Rect::from_min_size(min, Vec2::splat(TILE_PX));
						painter.rect_filled(tile, 0.0, color);
					}
				}

				let metadata = self.world.metadata_str();
				ui.label(egui::RichText::new(metadata).monospace().size(16.0));
			});

			ui.request_repaint_after(Duration::from_millis(frame.ms.unwrap_or(20).into()));
		}
	}
}

const PALETTE: [Color32; 16] = [
	Color32::from_rgb(0x00, 0x00, 0x00), // 0: black
	Color32::from_rgb(0xAA, 0x00, 0x00), // 1: red
	Color32::from_rgb(0x00, 0xAA, 0x00), // 2: green
	Color32::from_rgb(0xAA, 0xAA, 0x00), // 3: yellow
	Color32::from_rgb(0x00, 0x00, 0xAA), // 4: blue
	Color32::from_rgb(0xAA, 0x00, 0xAA), // 5: magenta
	Color32::from_rgb(0x00, 0xAA, 0xAA), // 6: cyan
	Color32::from_rgb(0xAA, 0xAA, 0xAA), // 7: white
	Color32::from_rgb(0x55, 0x55, 0x55), // 8: bright black
	Color32::from_rgb(0xFF, 0x55, 0x55), // 9: bright red
	Color32::from_rgb(0x55, 0xFF, 0x55), // 10: bright green
	Color32::from_rgb(0xFF, 0xFF, 0x55), // 11: bright yellow
	Color32::from_rgb(0x55, 0x55, 0xFF), // 12: bright blue
	Color32::from_rgb(0xFF, 0x55, 0xFF), // 13: bright magenta
	Color32::from_rgb(0x55, 0xFF, 0xFF), // 14: bright cyan
	Color32::from_rgb(0xFF, 0xFF, 0xFF), // 15: bright white
];
