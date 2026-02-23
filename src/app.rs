use crate::scanner::scan_library;
use crate::types::{AppConfig, MediaItem, SortColumn, SortOrder, StatusRank};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use std::fs;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

pub struct MediaStatsApp {
    items: Vec<MediaItem>,
    filtered_indices: Vec<usize>,
    search_query: String,
    config: AppConfig,
    sort_primary: Option<(SortColumn, SortOrder)>,
    sort_secondary: Option<(SortColumn, SortOrder)>,
    
    // Scanning state
    is_scanning: bool,
    scan_receiver: Receiver<Vec<MediaItem>>,
    scan_sender: Sender<Vec<MediaItem>>,
    status_message: String,
}

impl MediaStatsApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Load config
        let config_path = "config.json";
        let mut config = AppConfig::default();
        if let Ok(file) = fs::File::open(config_path) {
            if let Ok(c) = serde_json::from_reader(file) {
                config = c;
            }
        }

        let (tx, rx) = channel();

        let mut app = Self {
            items: Vec::new(),
            filtered_indices: Vec::new(),
            search_query: String::new(),
            config,
            sort_primary: None,
            sort_secondary: None,
            is_scanning: false,
            scan_receiver: rx,
            scan_sender: tx,
            status_message: "Ready to scan.".to_string(),
        };

        // Auto-load last library
        if let Some(path) = app.config.last_library_path.clone() {
            if Path::new(&path).exists() {
                app.start_scan(path);
            }
        }

        app
    }

    fn start_scan(&mut self, path: String) {
        self.is_scanning = true;
        self.status_message = format!("Scanning: {}...", path);
        let tx = self.scan_sender.clone();
        
        thread::spawn(move || {
            let items = scan_library(&path);
            let _ = tx.send(items);
        });
    }

    fn save_config(&self) {
        let config_path = "config.json";
        if let Ok(file) = fs::File::create(config_path) {
            let _ = serde_json::to_writer_pretty(file, &self.config);
        }
    }

    fn apply_filter_and_sort(&mut self) {
        let query = self.search_query.to_lowercase();
        
        // Filter
        let mut indices: Vec<usize> = self.items.iter().enumerate()
            .filter(|(_, item)| {
                if query.is_empty() { return true; }
                item.name.to_lowercase().contains(&query) ||
                item.group.to_lowercase().contains(&query) ||
                item.video_codec.to_lowercase().contains(&query) ||
                item.audio_codec.to_lowercase().contains(&query) ||
                item.source.to_lowercase().contains(&query) ||
                item.resolution.to_lowercase().contains(&query) ||
                item.season.as_deref().unwrap_or("").to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();

        // Sort
        let items = &self.items;
        let config = &self.config;
        let sort_primary = self.sort_primary;
        let sort_secondary = self.sort_secondary;

        if let Some((col, order)) = sort_secondary {
            indices.sort_by(|&a, &b| {
                let item_a = &items[a];
                let item_b = &items[b];
                Self::static_compare(item_a, item_b, col, config, order)
            });
        }

        if let Some((col, order)) = sort_primary {
            indices.sort_by(|&a, &b| {
                let item_a = &items[a];
                let item_b = &items[b];
                Self::static_compare(item_a, item_b, col, config, order)
            });
        }

        self.filtered_indices = indices;
    }

    fn static_compare(a: &MediaItem, b: &MediaItem, col: SortColumn, config: &AppConfig, order: SortOrder) -> std::cmp::Ordering {
        let cmp = match col {
            SortColumn::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortColumn::Season => a.season.as_deref().unwrap_or("").cmp(b.season.as_deref().unwrap_or("")),
            SortColumn::Group => a.group.to_lowercase().cmp(&b.group.to_lowercase()),
            SortColumn::Resolution => a.resolution.cmp(&b.resolution), 
            SortColumn::Source => a.source.to_lowercase().cmp(&b.source.to_lowercase()),
            SortColumn::VideoCodec => a.video_codec.to_lowercase().cmp(&b.video_codec.to_lowercase()),
            SortColumn::AudioCodec => a.audio_codec.to_lowercase().cmp(&b.audio_codec.to_lowercase()),
            SortColumn::AvgSize => a.avg_size_gb.partial_cmp(&b.avg_size_gb).unwrap_or(std::cmp::Ordering::Equal),
            SortColumn::Verified => {
                let status_a = config.media_statuses.get(&a.path).map(|s| s.as_str()).unwrap_or("");
                let status_b = config.media_statuses.get(&b.path).map(|s| s.as_str()).unwrap_or("");
                status_a.cmp(status_b)
            },
            SortColumn::Status => {
                let rank_a = StatusRank::from_item(a);
                let rank_b = StatusRank::from_item(b);
                rank_a.cmp(&rank_b)
            }
        };
        if order == SortOrder::Descending { cmp.reverse() } else { cmp }
    }

    fn process_sort_action(&mut self, col: SortColumn, secondary: bool) {
        if secondary {
            if let Some((current_col, current_order)) = self.sort_secondary {
                if current_col == col {
                    self.sort_secondary = Some((col, if current_order == SortOrder::Ascending { SortOrder::Descending } else { SortOrder::Ascending }));
                } else {
                    self.sort_secondary = Some((col, SortOrder::Ascending));
                }
            } else {
                self.sort_secondary = Some((col, SortOrder::Ascending));
            }
        } else {
            if let Some((current_col, current_order)) = self.sort_primary {
                if current_col == col {
                    self.sort_primary = Some((col, if current_order == SortOrder::Ascending { SortOrder::Descending } else { SortOrder::Ascending }));
                } else {
                    self.sort_primary = Some((col, SortOrder::Ascending));
                }
            } else {
                self.sort_primary = Some((col, SortOrder::Ascending));
            }
        }
        self.apply_filter_and_sort();
    }
}

impl eframe::App for MediaStatsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(new_items) = self.scan_receiver.try_recv() {
            self.items = new_items;
            self.is_scanning = false;
            self.status_message = format!("Scan complete. Found {} items.", self.items.len());
            self.apply_filter_and_sort();
        }

        let mut status_changed = false;
        let mut sort_action = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Select Library Folder").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        let path_str = path.to_string_lossy().to_string();
                        self.config.last_library_path = Some(path_str.clone());
                        self.save_config();
                        self.start_scan(path_str);
                    }
                }

                let mut sort_mode = "Status: Default";
                if let Some((SortColumn::Status, order)) = self.sort_primary {
                    sort_mode = if order == SortOrder::Ascending { "Status: Best -> Worst" } else { "Status: Worst -> Best" };
                }
                
                egui::ComboBox::from_id_salt("status_sort")
                    .selected_text(sort_mode)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(sort_mode == "Status: Default", "Status: Default").clicked() {
                            self.sort_primary = None;
                            self.apply_filter_and_sort();
                        }
                        if ui.selectable_label(sort_mode == "Status: Best -> Worst", "Status: Best -> Worst").clicked() {
                            self.sort_primary = Some((SortColumn::Status, SortOrder::Ascending));
                            self.apply_filter_and_sort();
                        }
                        if ui.selectable_label(sort_mode == "Status: Worst -> Best", "Status: Worst -> Best").clicked() {
                            self.sort_primary = Some((SortColumn::Status, SortOrder::Descending));
                            self.apply_filter_and_sort();
                        }
                    });

                ui.label(&self.status_message);
                if self.is_scanning {
                    ui.spinner();
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("Search:");
                if ui.text_edit_singleline(&mut self.search_query).changed() {
                    self.apply_filter_and_sort();
                }
            });

            ui.add_space(10.0);

            let items = &self.items;
            let filtered = &self.filtered_indices;
            let config = &mut self.config;
            let mut actions = Vec::new();

            TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(300.0).at_least(100.0).clip(true))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(100.0).at_least(50.0))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(100.0).at_least(50.0))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(80.0).at_least(50.0))
                .column(Column::initial(100.0).at_least(50.0))
                .column(Column::initial(80.0).at_least(50.0))
                .header(20.0, |mut header| {
                    let cols = [
                        ("Name", SortColumn::Name),
                        ("Season", SortColumn::Season),
                        ("Group", SortColumn::Group),
                        ("Resolution", SortColumn::Resolution),
                        ("Source", SortColumn::Source),
                        ("Video", SortColumn::VideoCodec),
                        ("Audio", SortColumn::AudioCodec),
                        ("Avg Size", SortColumn::AvgSize),
                        ("Verified", SortColumn::Verified),
                    ];
                    
                    for (name, col_enum) in cols {
                         header.col(|ui| {
                             let response = ui.heading(name);
                             if response.clicked() {
                                 actions.push((col_enum, false));
                             } else if response.clicked_by(egui::PointerButton::Secondary) {
                                 actions.push((col_enum, true));
                             }
                         });
                    }
                })
                .body(|mut body| {
                    for &idx in filtered {
                        if idx >= items.len() { continue; }
                        let item = &items[idx];
                        let status_rank = StatusRank::from_item(item);
                        
                        let bg_color = match status_rank {
                            StatusRank::Airing => egui::Color32::from_rgb(70, 130, 180),
                            StatusRank::Great => egui::Color32::from_rgb(46, 139, 87),
                            StatusRank::Good => egui::Color32::from_rgb(144, 238, 144),
                            StatusRank::Okay => egui::Color32::from_rgb(255, 165, 0),
                            StatusRank::Bad => egui::Color32::from_rgb(205, 92, 92),
                            StatusRank::None => egui::Color32::TRANSPARENT,
                        };
                        
                        let text_color = if status_rank == StatusRank::Good || status_rank == StatusRank::Okay {
                            egui::Color32::BLACK
                        } else {
                            egui::Color32::WHITE
                        };

                        let row_height = 50.0;

                        body.row(row_height, |mut row| {
                            let add_content = |row: &mut egui_extras::TableRow, text: &str| {
                                row.col(|ui| {
                                    if status_rank != StatusRank::None {
                                         let rect = ui.max_rect();
                                         ui.painter().rect_filled(rect, 0.0, bg_color);
                                    }
                                    ui.label(egui::RichText::new(text).size(20.0).color(text_color));
                                });
                            };

                            add_content(&mut row, &item.name);
                            add_content(&mut row, item.season.as_deref().unwrap_or(""));
                            add_content(&mut row, &item.group);
                            add_content(&mut row, &item.resolution);
                            add_content(&mut row, &item.source);
                            add_content(&mut row, &item.video_codec);
                            add_content(&mut row, &item.audio_codec);
                            add_content(&mut row, &format!("{:.2} GB", item.avg_size_gb));
                            
                            row.col(|ui| {
                                if status_rank != StatusRank::None {
                                     let rect = ui.max_rect();
                                     ui.painter().rect_filled(rect, 0.0, bg_color);
                                }
                                
                                // FIX: We clone the status into an owned string so we drop the read-reference immediately
                                let current_status = config.media_statuses.get(&item.path).cloned().unwrap_or_default();

                                let mark = match current_status.as_str() {
                                    "verified" => "☑",
                                    "rejected" => "☒",
                                    _ => "☐",
                                };

                                let response = ui.add(egui::Label::new(egui::RichText::new(mark).size(20.0).color(text_color)).sense(egui::Sense::click()));

                                if response.clicked() {
                                    let new_status = match current_status.as_str() {
                                        "" => Some("verified".to_string()),
                                        "verified" => Some("rejected".to_string()),
                                        _ => None,
                                    };
                                    
                                    if let Some(s) = new_status {
                                        config.media_statuses.insert(item.path.clone(), s);
                                    } else {
                                        config.media_statuses.remove(&item.path);
                                    }
                                    status_changed = true;
                                } else if response.clicked_by(egui::PointerButton::Secondary) {
                                     let new_status = match current_status.as_str() {
                                        "" => Some("rejected".to_string()),
                                        "rejected" => Some("verified".to_string()),
                                        _ => None,
                                    };
                                    
                                    if let Some(s) = new_status {
                                        config.media_statuses.insert(item.path.clone(), s);
                                    } else {
                                        config.media_statuses.remove(&item.path);
                                    }
                                    status_changed = true;
                                }
                            });
                        });
                    }
                });
            
            if !actions.is_empty() {
                sort_action = Some(actions[0]);
            }
        });

        if status_changed {
            self.save_config();
        }

        if let Some((col, secondary)) = sort_action {
            self.process_sort_action(col, secondary);
        }
    }
}