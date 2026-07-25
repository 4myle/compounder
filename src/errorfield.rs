
#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

use eframe::egui;
use eframe::egui::Widget;

pub struct ErrorField<'a>
{
    value: &'a mut String,
    valid: bool
}

impl<'a> ErrorField<'a>
{
    pub const fn new (value: &'a mut String, valid: bool) -> Self {
        Self {
            value,
            valid
        }
    }

    fn reflect (&mut self, ui: &mut egui::Ui) {
        if !self.valid {
            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(2.0, visuals.error_fg_color);
            visuals.selection.stroke = egui::Stroke::new(1.0, visuals.error_fg_color);
        }
    }
}

impl Widget for ErrorField<'_>
{
    fn ui (mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.scope(|ui| { 
            self.reflect(ui);
            ui.text_edit_singleline(self.value)
        }).inner
    }
}
