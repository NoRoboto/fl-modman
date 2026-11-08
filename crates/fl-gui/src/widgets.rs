//! Shared widgets: the on-screen numeric picker (no keyboard on handhelds) and the in-app
//! folder browser (no native file dialog).

use std::fs;
use std::path::{Path, PathBuf};

use eframe::egui::{RichText, Ui};

use crate::input::Action;
use crate::nav::Nav;
use crate::ui::{self, ACCENT, BACKGROUND, Row, TITLE};

/// A fixed-width decimal number edited digit by digit: A enters edit mode, Left/Right pick a
/// digit, Up/Down change it, A or B leave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumPicker {
    pub digits: Vec<u8>,
    pub cursor: usize,
    pub editing: bool,
}

impl NumPicker {
    pub fn new(width: usize, value: u32) -> Self {
        let text = format!("{value:0width$}");
        let digits = text.bytes().rev().take(width).rev().map(|b| b - b'0');
        NumPicker {
            digits: digits.collect(),
            cursor: width - 1,
            editing: false,
        }
    }

    pub fn value(&self) -> u32 {
        self.digits.iter().fold(0u32, |v, d| {
            v.saturating_mul(10).saturating_add(u32::from(*d))
        })
    }

    pub fn step(&mut self, i: usize, up: bool) {
        self.digits[i] = (self.digits[i] + if up { 1 } else { 9 }) % 10;
    }

    /// Handles an action while editing; false if it is not the picker's.
    pub fn handle(&mut self, a: Action) -> bool {
        match a {
            Action::Left => self.cursor = self.cursor.saturating_sub(1),
            Action::Right => self.cursor = (self.cursor + 1).min(self.digits.len() - 1),
            Action::Up | Action::Down => self.step(self.cursor, a == Action::Up),
            Action::Accept | Action::Back => self.editing = false,
            _ => return false,
        }
        true
    }

    /// Draws the digits with +/- buttons (touch) and registers one focusable item.
    pub fn ui(&mut self, ui: &mut Ui, nav: &mut Nav) {
        if nav.focus != nav.next_index() {
            self.editing = false;
        }
        if self.editing {
            use Action::*;
            for a in [Up, Down, Left, Right, Accept, Back] {
                while nav.take(a) {
                    self.handle(a);
                }
            }
        }
        let r = ui.horizontal(|ui| {
            for i in 0..self.digits.len() {
                ui.vertical(|ui| {
                    if ui.small_button(" + ").clicked() {
                        self.step(i, true);
                    }
                    let mut text = RichText::new(self.digits[i].to_string()).size(TITLE);
                    if self.editing && i == self.cursor {
                        text = text.color(BACKGROUND).background_color(ACCENT);
                    }
                    ui.label(text);
                    if ui.small_button(" - ").clicked() {
                        self.step(i, false);
                    }
                });
            }
        });
        if nav.item(ui, &r.response) {
            self.editing = true;
        }
    }
}

/// A folder listing: sub-folders, plus archives when `archives` is set. Hidden entries are
/// skipped.
pub struct Browser {
    pub dir: PathBuf,
    pub entries: Vec<(String, bool)>,
    pub archives: bool,
}

pub fn is_archive(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    [".zip", ".7z", ".rar"].iter().any(|e| n.ends_with(e))
}

impl Browser {
    pub fn new(dir: &Path, archives: bool) -> Self {
        let mut b = Browser {
            dir: dir.into(),
            entries: Vec::new(),
            archives,
        };
        b.load();
        b
    }

    fn load(&mut self) {
        let list = fs::read_dir(&self.dir).into_iter().flatten().flatten();
        self.entries = list
            .map(|e| {
                (
                    e.file_name().to_string_lossy().into_owned(),
                    e.path().is_dir(),
                )
            })
            .filter(|(n, d)| !n.starts_with('.') && (*d || self.archives && is_archive(n)))
            .collect();
        self.entries.sort_by_key(|(n, d)| (!d, n.to_lowercase()));
    }

    /// Goes to the parent folder; false at the root.
    pub fn up(&mut self) -> bool {
        let moved = self.dir.pop();
        self.load();
        moved
    }

    /// Draws the listing and its action bar (Cancel when `cancel`, Up one folder, `use_label`):
    /// Some(None) on Cancel, Some(the archive picked or the current folder on `use_label`).
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        nav: &mut Nav,
        cancel: bool,
        use_label: &str,
    ) -> Option<Option<PathBuf>> {
        if nav.take(Action::Back) && self.up() {
            nav.reset();
        }
        let mut open = None;
        ui::body(ui, |ui| {
            ui::card(ui, &self.dir.display().to_string(), |ui| {
                for (i, (name, dir)) in self.entries.iter().enumerate() {
                    let text = if *dir {
                        format!("▶ {name}/")
                    } else {
                        name.clone()
                    };
                    if Row::new(text, "").nav(ui, nav) {
                        open = Some(i);
                    }
                }
                if self.entries.is_empty() {
                    ui.weak("No folders or archives here.");
                }
            })
        });
        let labels = ["Cancel", "Up one folder", use_label];
        let labels = &labels[usize::from(!cancel)..];
        match ui::actions(ui, nav, labels).map(|i| i + usize::from(!cancel)) {
            Some(0) => return Some(None),
            Some(1) if self.up() => nav.reset(),
            Some(1) => {}
            Some(_) => return Some(Some(self.dir.clone())),
            None => {}
        }
        let i = open?;
        if !self.entries[i].1 {
            return Some(Some(self.dir.join(&self.entries[i].0)));
        }
        self.dir.push(&self.entries[i].0);
        self.load();
        nav.reset();
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Action::*;

    #[test]
    fn picker_edits_digits_with_wraparound() {
        let mut p = NumPicker::new(5, 103);
        assert_eq!(p.digits, [0, 0, 1, 0, 3]);
        assert_eq!((p.value(), p.cursor), (103, 4));
        for a in [Up, Left, Down, Left, Left, Left, Left, Left, Up] {
            assert!(p.handle(a));
        }
        assert_eq!(p.value(), 10_194);
        p.editing = true;
        for a in [Right; 9] {
            p.handle(a);
        }
        assert_eq!(p.cursor, 4);
        assert!(p.handle(Back) && !p.editing);
        assert!(!p.handle(Toggle));
        assert_eq!(NumPicker::new(3, 98_765).value(), 765);
    }

    #[test]
    fn the_default_fonts_draw_every_symbol_the_screens_use() {
        // No circled letters (Ⓐ) nor ▲▼ in the bundled fonts: the hint bar writes [A] and ⏶⏷.
        let ctx = eframe::egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |_| {});
        out.textures_delta.clear();
        let font = eframe::egui::FontId::proportional(20.0);
        let missing: String = "☰⚠ℹ↕◀▶⏶⏷…·"
            .chars()
            .filter(|c| !ctx.fonts_mut(|f| f.has_glyph(&font, *c)))
            .collect();
        assert_eq!(missing, "");
    }

    #[test]
    fn browser_lists_folders_first_then_archives() {
        let dir = std::env::temp_dir().join(format!("flmm-gui-browse-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for d in ["b", "A", ".hidden"] {
            fs::create_dir_all(dir.join(d)).unwrap();
        }
        for f in ["z.ZIP", "m.7z", "notes.txt"] {
            fs::write(dir.join(f), "").unwrap();
        }
        let names = |b: &Browser| b.entries.iter().map(|e| e.0.clone()).collect::<Vec<_>>();
        assert_eq!(
            names(&Browser::new(&dir, true)),
            ["A", "b", "m.7z", "z.ZIP"]
        );
        let mut b = Browser::new(&dir, false);
        assert_eq!(names(&b), ["A", "b"]);
        assert!(b.up() && b.dir == dir.parent().unwrap());
        fs::remove_dir_all(&dir).unwrap();
    }
}
