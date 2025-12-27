use eframe::egui::{self, Slider};

use crate::{pages::SettingsPage};

impl SettingsPage {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.add(Slider::new(&mut self.max_parallelism, 1..=50)
            .text("Max parallel connections")
        );
    }
}

