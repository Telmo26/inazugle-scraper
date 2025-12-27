use eframe::egui::{self, ProgressBar};
use egui_extras::{Column, TableBuilder};

use crate::{pages::{CharactersPage, SettingsPage}, utils::{ELEMENT_LIST, GAME_LIST, POSITION_LIST, Progress}};

impl CharactersPage {
    pub fn render(&mut self, settings: &SettingsPage, ui: &mut egui::Ui) {
        let max_parallelism = settings.max_parallelism;

        ui.heading("Character Comparator");

        ui.separator();

        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.request.name);

            if ui.button("Send Request").clicked() {
                let request = self.request.clone();
                let mut db = self.character_cache.clone();
                let sender = self.sender.clone();

                let progress = Progress::new();
                self.progress = Some(progress.clone());

                self.runtime.spawn(async move {
                    let characters = request.send(&mut db, max_parallelism, progress).await;

                    let characters = characters
                        .into_iter()
                        .filter(|char| char.stats.is_some())
                        .collect();
                    let _ = sender.send(characters);
                });
            }

            if let Some(progress) = &self.progress {
                if !progress.pages_done() {
                    let (fetched, total) = progress.pages();
                    let frac = fetched as f32 / total as f32;
                    ui.add(ProgressBar::new(frac).text("Fetching pages..."))
                } else if !progress.characters_done() {
                    let (characters, total) = progress.characters();
                    let frac = characters as f32 / total as f32;
                    ui.add(ProgressBar::new(frac).text("Fetching characters..."))
                } else {
                    ui.add(ProgressBar::new(1f32).text("Characters fetched"))
                }
            } else {
                ui.add(ProgressBar::new(0f32))
            }
        });

        ui.horizontal(|ui| {
            ui.menu_button("Elements", |ui| {
                for element in &ELEMENT_LIST {
                    let checked = self.request.has_element(&element);

                    if ui.selectable_label(checked, element.db_str()).clicked() {
                        self.request.toggle_element(element);
                    }
                }
            });

            ui.menu_button("Position", |ui| {
                for position in &POSITION_LIST {
                    let checked = self.request.has_position(position);

                    if ui.selectable_label(checked, position.to_str()).clicked() {
                        self.request.toggle_position(position);
                    }
                }
            });

            ui.menu_button("Game", |ui| {
                for game in &GAME_LIST {
                    let checked = self.request.has_game(game);

                    if ui.selectable_label(checked, game.to_str()).clicked() {
                        self.request.toggle_game(game);
                    }
                }
            });
        });

        ui.separator();

        TableBuilder::new(ui)
            .striped(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto()) // ID
            .column(Column::auto()) // Name
            .column(Column::auto()) // Kick
            .column(Column::auto()) // Control
            .column(Column::auto()) // Technique
            .column(Column::auto()) // Pressure
            .column(Column::auto()) // Physical
            .column(Column::auto()) // Agility
            .column(Column::auto()) // Intelligence
            .column(Column::auto()) // Link to the self
            .header(20.0, |mut header| {
                header.col(|ui| {
                    sortable_header(ui, "ID", SortColumn::ID, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Name", SortColumn::Name, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Kick", SortColumn::Kick, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Control", SortColumn::Control, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Technique", SortColumn::Technique, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Pressure", SortColumn::Pressure, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Physical", SortColumn::Physical, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Agility", SortColumn::Agility, self);
                });
                header.col(|ui| {
                    sortable_header(ui, "Intelligence", SortColumn::Intelligence, self);
                });
                header.col(|ui| {
                    ui.add_enabled(false, egui::Button::new("Link"));
                });
            })
            .body(|mut body| {
                for character in &self.characters {
                    let stats = character.stats.as_ref().unwrap();
                    body.row(18.0, |mut row| {
                        row.col(|ui| {
                            ui.label(character.number.to_string());
                        });
                        row.col(|ui| {
                            ui.label(&character.name);
                        });
                        row.col(|ui| {
                            ui.label(stats.kick.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.control.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.technique.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.pressure.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.physical.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.agility.to_string());
                        });
                        row.col(|ui| {
                            ui.label(stats.intelligence.to_string());
                        });
                        row.col(|ui| {
                            ui.hyperlink_to("Inazugle", &character.page_url);
                        });
                    });
                }
            });
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortColumn {
    ID,
    Name,
    Kick,
    Control,
    Technique,
    Pressure,
    Physical,
    Agility,
    Intelligence,
}

pub fn sortable_header(ui: &mut egui::Ui, label: &str, column: SortColumn, state: &mut CharactersPage) {
    let mut text = label.to_string();

    if state.sort_column == column {
        text.push_str(if state.sort_ascending { " ^" } else { " v" });
    }

    if ui.button(text).clicked() {
        if state.sort_column == column {
            state.sort_ascending = !state.sort_ascending;
        } else {
            state.sort_column = column;
            state.sort_ascending = true;
        }
        state.sort_characters();
    }
}