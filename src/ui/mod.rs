use std::sync::{
	Arc,
	atomic::{AtomicBool, Ordering},
	mpsc::Receiver,
};
use std::time::{Duration, Instant};

use antbyte::{
	util::vec2::Pos,
	world::{
		World,
		config::WorldConfig,
		frame::{FrameInput, FrameOutput},
	},
};
use eframe::{
	App,
	egui::{self, Color32, Pos2, Rect, Sense, Vec2},
};

const TILE_SIZE: f32 = 10.0;
const TILE_CAP: f32 = 80.0;
const PADDING: f32 = 80.0;
const MIN_SIZE: f32 = 32.0 * TILE_SIZE;

pub fn run_with_watch(world: &World, watch_rx: Option<Receiver<()>>) -> eframe::Result<bool> {
	let WorldConfig { height, width, .. } = *world.config();

	let (height, width) = (height as f32, width as f32);

	let tile_size: f32 = TILE_SIZE / (height.max(width).div_euclid(TILE_CAP) + 1.0);

	let height = (tile_size * height + PADDING).max(MIN_SIZE);
	let width = (tile_size * width).max(MIN_SIZE);

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

struct AntbyteApp {
	world: World,
	tile_size: f32,
	last_frame: Option<FrameOutput>,
	stopped: bool,
	step_mode: bool,
	next_frame_at: Instant,
	pending_keys: String,
	watch_rx: Option<Receiver<()>>,
	restart_requested: Arc<AtomicBool>,
}

impl AntbyteApp {
	pub fn new(
		world: World,
		tile_size: f32,
		watch_rx: Option<Receiver<()>>,
		restart_requested: Arc<AtomicBool>,
	) -> Self {
		Self {
			world,
			tile_size,
			last_frame: None,
			stopped: false,
			step_mode: false,
			next_frame_at: Instant::now(),
			pending_keys: String::new(),
			watch_rx,
			restart_requested,
		}
	}
}

impl App for AntbyteApp {
	fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		let WorldConfig {
			height,
			width,
			keys,
			..
		} = self.world.config().clone();

		if self
			.watch_rx
			.as_ref()
			.is_some_and(|rx| rx.try_recv().is_ok())
		{
			self.restart_requested.store(true, Ordering::Relaxed);
			ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
			return;
		}

		if !self.stopped {
			let now = Instant::now();
			let mut held_keys = String::new();
			ui.input(|input| {
				for key in &input.keys_down {
					if let Some(ch) = key_to_char(*key) {
						push_unique_char(&mut held_keys, ch);
					}
				}

				for event in &input.events {
					if let egui::Event::Key {
						key,
						pressed: true,
						repeat: false,
						..
					} = event && let Some(ch) = key_to_char(*key)
					{
						push_unique_char(&mut self.pending_keys, ch);
					}
				}
			});

			let should_advance = if self.step_mode {
				!self.pending_keys.is_empty()
			} else {
				now >= self.next_frame_at
			};

			if should_advance {
				let mut keys_str = held_keys.clone();
				for ch in self.pending_keys.chars() {
					push_unique_char(&mut keys_str, ch);
				}

				let input = if let Some(keys) = keys.as_ref() {
					antbyte::ui::chars_to_input(&Some(keys.clone()), &keys_str)
				} else {
					0
				};

				if let Some(frame) = self.world.next_frame(&FrameInput { ext_in: input }) {
					let frame_ms = frame.ms;
					self.last_frame = Some(frame);
					self.pending_keys.clear();
					if let Some(frame_ms) = frame_ms {
						self.step_mode = false;
						self.next_frame_at += Duration::from_millis(frame_ms.into());
					} else {
						self.step_mode = true;
					}
				} else {
					self.stopped = true;
				}
			}
		}

		if let Some(frame) = self.last_frame.as_ref() {
			ui.vertical(|ui| {
				let size = Vec2::new(
					width as f32 * self.tile_size,
					height as f32 * self.tile_size,
				);
				let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
				let painter = ui.painter_at(rect);

				for y in 0..height {
					for x in 0..width {
						let value = frame.bg.get(&Pos { x, y }).unwrap_or(&0);
						let color = PALETTE[*value as usize];

						let min = Pos2::new(
							rect.left() + x as f32 * self.tile_size,
							rect.top() + y as f32 * self.tile_size,
						);
						let tile = Rect::from_min_size(min, Vec2::splat(self.tile_size));
						painter.rect_filled(tile, 0.0, color);
					}
				}

				ui.add_space(8.0);
				ui.horizontal(|ui| {
					ui.label(
						egui::RichText::new(self.world.metadata_str())
							.monospace()
							.size(16.0),
					);

					if !self.stopped {
						ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
							if ui
								.add_sized(
									[120.0, 44.0],
									egui::Button::new(egui::RichText::new("STOP").size(22.0)),
								)
								.clicked()
							{
								self.stopped = true;
							}
						});
					}
				});
			});
		}

		if !self.stopped {
			ui.request_repaint_after(if self.step_mode {
				Duration::from_millis(30)
			} else {
				self.next_frame_at.saturating_duration_since(Instant::now())
			});
		}
	}
}

fn key_to_char(key: egui::Key) -> Option<char> {
	let mut chars = key.symbol_or_name().chars();
	match (chars.next(), chars.next()) {
		(Some(ch), None) => Some(ch),
		_ => None,
	}
}

fn push_unique_char(target: &mut String, ch: char) {
	let ch = ch.to_ascii_lowercase();
	if ch.is_ascii() && !target.contains(ch) {
		target.push(ch);
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
