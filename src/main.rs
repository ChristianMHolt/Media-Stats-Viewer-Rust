mod types;
mod scanner;
mod app;

use app::MediaStatsApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init(); 

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Media Stats Viewer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            
            // --- VISUAL OVERHAUL START ---
            let mut style = (*cc.egui_ctx.style()).clone();
            
            // 1. Larger, more readable text (matching your CustomTkinter sizing)
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles = [
                (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
                (TextStyle::Body, FontId::new(18.0, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(18.0, FontFamily::Monospace)),
                (TextStyle::Button, FontId::new(18.0, FontFamily::Proportional)),
                (TextStyle::Small, FontId::new(14.0, FontFamily::Proportional)),
            ].into();

            // 2. Better spacing so everything isn't crammed together
            style.spacing.item_spacing = egui::vec2(10.0, 10.0);
            style.spacing.button_padding = egui::vec2(12.0, 8.0);
            
            // 3. Force dark mode and apply rounded corners to all widgets
            let mut visuals = egui::Visuals::dark();
            let rounding = egui::Rounding::same(8.0);
            visuals.widgets.noninteractive.rounding = rounding;
            visuals.widgets.inactive.rounding = rounding;
            visuals.widgets.hovered.rounding = rounding;
            visuals.widgets.active.rounding = rounding;
            visuals.widgets.open.rounding = rounding;
            visuals.window_rounding = rounding;
            
            // 4. CustomTkinter Blue Accent Color
            visuals.selection.bg_fill = egui::Color32::from_rgb(31, 106, 165); 

            // Apply the new styles
            cc.egui_ctx.set_visuals(visuals);
            cc.egui_ctx.set_style(style);
            // --- VISUAL OVERHAUL END ---

            Ok(Box::new(MediaStatsApp::new(cc)))
        }),
    )
}