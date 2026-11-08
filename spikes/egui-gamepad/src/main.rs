//! Spike 0(a): can an egui/eframe window be driven only by a gamepad on the
//! ROG Ally in Steam Gaming Mode (gamescope)? D-pad moves, A toggles, B quits.
use eframe::egui;
use gilrs::{Axis, Button, EventType, Gilrs};
use std::time::{Duration, Instant};

struct Spike {
    gilrs: Option<Gilrs>,
    mods: Vec<(&'static str, bool)>,
    selected: usize,
    last_event: String,
    quit: bool,
    /// Current stick direction (-1 down, 0 neutral, 1 up) and when it last fired.
    stick_dir: i8,
    stick_fired: Instant,
}

impl Spike {
    fn new() -> Self {
        let gilrs = Gilrs::new().ok();
        let pads = gilrs
            .as_ref()
            .map(|g| g.gamepads().map(|(_, p)| p.name().to_string()).collect::<Vec<_>>().join(", "))
            .unwrap_or_else(|| "gilrs init failed".into());
        Self {
            gilrs,
            mods: vec![
                ("Facepack 26/27 + update 4", true),
                ("Spanish commentary (com269)", true),
                ("Mariano Closs (eng slot)", true),
                ("Spotify Camp Nou 2026", true),
                ("Machete cursors", true),
                ("Machete cameras", false),
                ("ReShade Natural Light", true),
            ],
            selected: 0,
            last_event: format!("gamepads: [{pads}]"),
            quit: false,
            stick_dir: 0,
            stick_fired: Instant::now(),
        }
    }

    fn step(&mut self, down: bool) {
        let n = self.mods.len();
        self.selected = if down { (self.selected + 1) % n } else { (self.selected + n - 1) % n };
    }

    fn poll_gamepad(&mut self) {
        const DEADZONE: f32 = 0.5;
        const FIRST_REPEAT: Duration = Duration::from_millis(400);
        const NEXT_REPEAT: Duration = Duration::from_millis(150);
        let Some(gilrs) = self.gilrs.as_mut() else { return };
        let mut events = Vec::new();
        while let Some(ev) = gilrs.next_event() {
            events.push(ev.event);
        }
        for event in events {
            if let EventType::AxisChanged(Axis::LeftStickY, value, _) = event {
                let dir = if value > DEADZONE { 1 } else if value < -DEADZONE { -1 } else { 0 };
                if dir != self.stick_dir {
                    self.stick_dir = dir;
                    if dir != 0 {
                        self.last_event = format!("left stick {}", if dir > 0 { "up" } else { "down" });
                        self.step(dir < 0);
                        self.stick_fired = Instant::now() + FIRST_REPEAT - NEXT_REPEAT;
                    }
                }
            }
            if let EventType::ButtonPressed(button, _) = event {
                self.last_event = format!("pressed {button:?}");
                match button {
                    Button::DPadDown => self.step(true),
                    Button::DPadUp => self.step(false),
                    Button::South => self.mods[self.selected].1 = !self.mods[self.selected].1,
                    Button::East => self.quit = true,
                    _ => {}
                }
            }
        }
        // Auto-repeat while the stick is held.
        if self.stick_dir != 0 && self.stick_fired.elapsed() >= NEXT_REPEAT {
            self.step(self.stick_dir < 0);
            self.stick_fired = Instant::now();
        }
    }
}

impl eframe::App for Spike {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.poll_gamepad();
        if self.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.set_pixels_per_point(1.6);
        {
            ui.heading("fl-modman: gamepad spike");
            ui.label("D-pad or left stick: move   A: toggle   B: quit");
            ui.separator();
            for (i, (name, on)) in self.mods.iter_mut().enumerate() {
                let text = format!("{} {}", if *on { "[x]" } else { "[ ]" }, name);
                let label = egui::RichText::new(text).size(20.0);
                let resp = ui.selectable_label(i == self.selected, label);
                if resp.clicked() {
                    *on = !*on;
                }
            }
            ui.separator();
            ui.label(&self.last_event);
        }
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        ..Default::default()
    };
    eframe::run_native("fl-modman spike", options, Box::new(|_cc| Ok(Box::new(Spike::new()))))
}
