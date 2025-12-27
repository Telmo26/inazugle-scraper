#![windows_subsystem = "windows"]

use eframe::egui::{self, ProgressBar};
use egui_extras::{Column, TableBuilder};

use tokio::{runtime::Runtime, sync::mpsc};

mod request;
mod utils;
mod database;

use utils::{Character, SortColumn, Progress};
use database::Database;
use request::Request;

use crate::utils::{ELEMENT_LIST, GAME_LIST, POSITION_LIST};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let native_options = eframe::NativeOptions::default();

    let _ = eframe::run_native(
        "Inazugle Scraper", 
        native_options, 
        Box::new(|cc| 
            Ok(Box::new(InazugleScraper::new(cc)))
        )
    );

    return Ok(())
}

struct InazugleScraper {
    runtime: Runtime,
    character_cache: Database,
    request: Request,

    characters: Vec<Character>,
    sender: mpsc::UnboundedSender<Vec<Character>>,
    receiver: mpsc::UnboundedReceiver<Vec<Character>>,
    progress: Option<Progress>,

    sort_column: SortColumn,
    sort_ascending: bool,
}

impl InazugleScraper {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let character_cache= Database::connect("character_cache.sqlite");   
        
        let (sender, receiver) = mpsc::unbounded_channel();

        InazugleScraper { 
            runtime, 
            character_cache,
            request: Request::new(), 

            characters: Vec::new(),
            sender,
            receiver,
            progress: None,

            sort_column: SortColumn::ID,
            sort_ascending: true,
        }
    }
}

impl eframe::App for InazugleScraper {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(characters) = self.receiver.try_recv() {
            // We update the stored characters
            self.characters = characters;
            self.sort_characters();
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
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
                        let characters = request.send(&mut db, progress).await;

                        let characters = characters.into_iter().filter(|char | {
                            char.stats.is_some()
                        })
                        .collect();
                        let _ = sender.send(characters);
                    });
                }

                if let Some(progress) = &self.progress {
                    if !progress.pages_done() {
                        let (page, total) = progress.pages();
                        let frac = page as f32 / total as f32;
                        ui.add(
                            ProgressBar::new(frac)
                                .text("Fetching pages...")
                        )
                    } else if !progress.characters_done() {
                        let (characters, total) = progress.characters();
                        let frac = characters as f32 / total as f32;
                        ui.add(
                            ProgressBar::new(frac)
                                .text("Fetching characters...")
                        )
                    } else {
                        ui.add(
                            ProgressBar::new(1f32)
                                .text("Characters fetched")
                        )
                    }
                } else {
                    ui.add(
                        ProgressBar::new(0f32)
                    )
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
                .column(Column::auto()) // Link to the page

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
                        ui.add_enabled(
                            false,
                            egui::Button::new("Link"),
                        );
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
        });
        ctx.request_repaint();
   }
}

impl InazugleScraper {
    fn sort_characters(&mut self) {
        let column = self.sort_column;

        self.characters.sort_by(|a, b| {
            let a_stats = a.stats.as_ref().unwrap();
            let b_stats = b.stats.as_ref().unwrap();

            match column {
                SortColumn::ID => a.number.cmp(&b.number),
                SortColumn::Name => a.name.cmp(&b.name),
                SortColumn::Kick => a_stats.kick.cmp(&b_stats.kick),
                SortColumn::Control => a_stats.control.cmp(&b_stats.control),
                SortColumn::Technique => a_stats.technique.cmp(&b_stats.technique),
                SortColumn::Pressure => a_stats.pressure.cmp(&b_stats.pressure),
                SortColumn::Physical => a_stats.physical.cmp(&b_stats.physical),
                SortColumn::Agility => a_stats.agility.cmp(&b_stats.agility),
                SortColumn::Intelligence => a_stats.intelligence.cmp(&b_stats.intelligence)
            }
        });

        if !self.sort_ascending {
            self.characters.reverse();
        }
    }
}

fn sortable_header(ui: &mut egui::Ui, label: &str, column: SortColumn, app: &mut InazugleScraper) {
    let mut text = label.to_string();

    if app.sort_column == column {
        text.push_str(if app.sort_ascending { " ^" } else { " v" });
    }

    if ui.button(text).clicked() {
        if app.sort_column == column {
            app.sort_ascending = !app.sort_ascending;
        } else {
            app.sort_column = column;
            app.sort_ascending = true;
        }
        app.sort_characters();
    }
}

