use eframe::egui;

mod database;
mod request;
mod utils;
mod pages;

use pages::{
    CharactersPage, SettingsPage,
};

pub struct InazugleScraper {
    active_tab: Tab,

    characters_page: CharactersPage,
    settings: SettingsPage,
}

impl InazugleScraper {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        InazugleScraper { 
            active_tab: Tab::Characters, 
            characters_page: CharactersPage::new(),
            settings: SettingsPage::default(),
        }
    }
}

impl eframe::App for InazugleScraper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.active_tab == Tab::Characters {
            self.characters_page.receive_char();
        }

        egui::TopBottomPanel::top("tabs").show(ctx, |ui|{
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Characters, "Characters");
                ui.selectable_value(&mut self.active_tab, Tab::Techniques, "Techniques");
                ui.selectable_value(&mut self.active_tab, Tab::Settings, "Settings");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.active_tab {
                Tab::Characters => self.characters_page.render(&self.settings, ui),
                Tab::Techniques => (),
                Tab::Settings => self.settings.render(ui),
            }
            
        });
        ctx.request_repaint();
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Characters,
    Techniques,
    Settings,
}