mod types;
mod scanner;
mod app;

use app::MediaStatsApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "Media Stats Viewer",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            let mut style = (*cc.egui_ctx.style()).clone();

            // Fonts
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles = [
                (TextStyle::Heading, FontId::new(28.0, FontFamily::Proportional)),
                (TextStyle::Body, FontId::new(16.0, FontFamily::Proportional)),
                (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
                (TextStyle::Button, FontId::new(16.0, FontFamily::Proportional)),
                (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
            ].into();

            // Spacing
            style.spacing.item_spacing = egui::vec2(8.0, 8.0);
            style.spacing.window_margin = egui::Margin::same(16.0);
            style.spacing.button_padding = egui::vec2(16.0, 10.0);

            // Visuals (Modern Dark Theme)
            let mut visuals = egui::Visuals::dark();

            // Colors
            let bg_color = egui::Color32::from_rgb(30, 30, 46);       // Deep dark blue/grey
            let panel_bg = egui::Color32::from_rgb(36, 36, 56);       // Slightly lighter
            let text_color = egui::Color32::from_rgb(205, 214, 244);  // Soft white
            let accent = egui::Color32::from_rgb(137, 180, 250);      // Soft blue
            let widget_bg = egui::Color32::from_rgb(49, 50, 68);      // Surface color
            let widget_hover = egui::Color32::from_rgb(69, 71, 90);   // Lighter surface

            visuals.panel_fill = bg_color;
            visuals.window_fill = bg_color;
            visuals.override_text_color = Some(text_color);

            visuals.widgets.noninteractive.bg_fill = panel_bg;
            visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, text_color);

            visuals.widgets.inactive.bg_fill = widget_bg;
            visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_color);

            visuals.widgets.hovered.bg_fill = widget_hover;
            visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, text_color);

            visuals.widgets.active.bg_fill = accent;
            visuals.widgets.active.rounding = egui::Rounding::same(6.0);
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, egui::Color32::BLACK);

            visuals.selection.bg_fill = accent;
            visuals.selection.stroke = egui::Stroke::new(1.0, accent);

            visuals.window_rounding = egui::Rounding::same(12.0);
            visuals.menu_rounding = egui::Rounding::same(6.0);

            cc.egui_ctx.set_visuals(visuals);
            cc.egui_ctx.set_style(style);

            Ok(Box::new(MediaStatsApp::new(cc)))
        }),
    )
}
