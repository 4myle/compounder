
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

use eframe::egui;
use eframe::egui::Widget;

pub struct Switch
{
    state: bool

}

impl Switch
{
    pub const fn new (state: bool) -> Self {
        Self {
            state
        }
    }
}

impl Widget for Switch
{
    fn ui (mut self, ui: &mut egui::Ui) -> egui::Response {
        let size = ui.spacing().interact_size.y * egui::vec2(2.5, 1.0);
        let (area, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
        if response.clicked() {
            self.state = !self.state;
            response.mark_changed();
        }
        if ui.is_rect_visible(area) {
            let easing  = ui.ctx().animate_bool_responsive(response.id, self.state);
            let area    = area.expand(ui.style().interact_selectable(&response, self.state).expansion);
            let radius  = 0.5 * area.height();
            let visuals = ui.style().interact_selectable(&response, self.state);
            ui.painter().rect(area, radius, visuals.bg_fill, visuals.bg_stroke, egui::StrokeKind::Middle); // Paint "slider" beneath.
            let circle  = egui::lerp((area.left() + radius - 2.0)..=(area.right() - radius), easing);
            let center  = egui::pos2(circle, area.center().y);
            ui.painter().circle(center, 0.8 * radius, visuals.weak_bg_fill, visuals.fg_stroke); // Paint "knob" above.
        }
        response
    }
}
// Alternative way without struct, lifetimes and what not.
// pub fn toggle (state: &mut bool) -> impl egui::Widget + '_ {
//     move |ui: &mut egui::Ui| switch(ui, state)
// }
