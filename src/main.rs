#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
// #![windows_subsystem = "windows"] // Causes stdout to disappear.

#![deny(clippy::pedantic)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(unused_must_use)]

// #![deny(clippy::unwrap_used)]
// #![deny(clippy::expect_used)]
// #![deny(clippy::panic)]
// #![deny(unused_must_use)]

const GUI_SIZE: egui::Vec2 = egui::Vec2::new(400.0, 390.0);
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(170, 0, 204);
const DATEFORMAT: &str = "%Y-%m-%d";

use chrono::NaiveDate;
use eframe::egui;
use eframe:: { 
    App, 
    Frame
};

mod switch;
mod errorfield;

use switch::Switch;
use errorfield::ErrorField;

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Copy, Clone)]
enum InterfaceSize 
{
    Small,
    Medium,
    Large
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Copy, Clone)]
enum InterfaceMode
{
    Dark,
    Light
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Compounder 
{
    start_date: String,
    final_date: String,
    years: u8,
    months: u8,
    weeks: u8,
    days: u8,
    start_amount: String,
    final_amount: String,
    cagr: String,
    ui_size: InterfaceSize,
    ui_mode: InterfaceMode
}

impl Compounder 
{
    fn new (context: &eframe::CreationContext<'_>) -> Self {
        let cc: Compounder = if let Some(ps) = context.storage { eframe::get_value(ps, eframe::APP_KEY).unwrap_or_default() } else { Compounder::default() };
        // egui_extras::install_image_loaders(&cc.egui_ctx);
        Self::set_fonts(&context.egui_ctx);
        Self::set_style(&context.egui_ctx, cc.ui_mode);
        cc
    }

    fn resize (&mut self, context: &egui::Context, size: InterfaceSize) {
        if  self.ui_size == size {
            return;
        }
        self.ui_size = size;
        let zf = match size {
            InterfaceSize::Small  => 1.0,
            InterfaceSize::Medium => 1.3,
            InterfaceSize::Large  => 1.7
        };
        // context.set_zoom_factor(zf); // Strange things happen when zoom is set through method.
        context.options_mut(|writer| writer.zoom_factor = zf);
        context.send_viewport_cmd(egui::ViewportCommand::InnerSize(GUI_SIZE)); // Hack to make gui resize.
    }

    fn remode (&mut self, context: &egui::Context, mode: InterfaceMode) {
        if  self.ui_mode == mode {
            return;
        }
        self.ui_mode = mode;
        Self::set_style(context, mode);
    }

    fn get_frame (&mut self) -> egui::Frame {
        let cb = match self.ui_mode {
            InterfaceMode::Dark  => egui::Color32::from_rgb( 20,  15,  15),
            InterfaceMode::Light => egui::Color32::from_rgb(250, 245, 245)
        };
        egui::Frame {
            inner_margin: egui::Margin::same(24),
            fill: cb,
            ..Default::default()
        }
    }

    fn set_fonts (context: &egui::Context) {
        let fontname = "Sans Font";
        let mut font = egui::FontDefinitions::default();
        font.font_data.insert(fontname.to_string(), std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf"))));
        if let Some(p) = font.families.get_mut(&egui::FontFamily::Proportional) {
            p.insert(0, fontname.to_string());
            context.set_fonts(font);
        };
    }
    
    fn set_style (context: &egui::Context, mode: InterfaceMode) {
        let mut vs: egui::Visuals;
        match mode {
            InterfaceMode::Dark  => {
                context.set_theme(egui::Theme::Dark);
                vs = egui::Visuals::dark();
                vs.override_text_color = Option::Some(egui::Color32::from_gray(255));
            },
            InterfaceMode::Light => {
                context.set_theme(egui::Theme::Light);
                vs = egui::Visuals::light();
                vs.override_text_color = Option::Some(egui::Color32::from_gray(0));
            }
        }
        vs.widgets.active.bg_fill = ACCENT_COLOR;
        vs.widgets.noninteractive.bg_fill = ACCENT_COLOR;
        vs.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.6);
        vs.widgets.hovered.bg_fill = ACCENT_COLOR;
        vs.widgets.hovered.weak_bg_fill = ACCENT_COLOR.gamma_multiply(0.1);
        vs.slider_trailing_fill = true;
        context.set_visuals(vs);
    
    }
    
    fn valid_start (&self) -> bool {
        NaiveDate::parse_from_str(&self.start_date, DATEFORMAT).is_ok() && self.start_date.len() == 10 
    }

    fn valid_final (&self) -> bool {
        NaiveDate::parse_from_str(&self.final_date, DATEFORMAT).is_ok() && self.final_date.len() == 10
    }

    fn valid_range (&self) -> bool {
        let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT);
        let fd = NaiveDate::parse_from_str(&self.final_date, DATEFORMAT);
        sd.is_ok() && fd.is_ok() && sd.unwrap_or_default() <= fd.unwrap_or_default()
    }

    fn redo_parts (&mut self) {
        let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT);
        let fd = NaiveDate::parse_from_str(&self.final_date, DATEFORMAT);
        if  sd.is_err() || fd.is_err() {
            return;
        }
        let sd = sd.unwrap_or_default();
        let fd = fd.unwrap_or_default();
        if fd < sd {
            return;
        }
        let (y,m,w,d) = date_difference(sd, fd);
        // println!("years = {},\n months = {},\n weeks = {},\n days = {}\n", y,m,w,d);
        self.years  = y;
        self.months = m;
        self.weeks  = w;
        self.days   = d;
        self.redo_cagr();
    }

    fn redo_final (&mut self) {
        let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT);
        if  sd.is_err() {
            return;
        }
        let fd = sd.ok()
            .and_then(|r| r.checked_add_months(chrono::Months::new(12 * u32::from(self.years) + u32::from(self.months)))
            .and_then(|r| r.checked_add_days(chrono::Days::new(7 * u64::from(self.weeks) + u64::from(self.days))))
        ).unwrap_or_default();
        self.final_date = fd.format(DATEFORMAT).to_string();
        self.redo_cagr();
    }

    fn redo_cagr (&mut self) {
        //TODO: refactor string to dates into own metod?
        let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT);
        let fd = NaiveDate::parse_from_str(&self.final_date, DATEFORMAT);
        if  sd.is_err() || fd.is_err() {
            return;
        }
        let sd = sd.unwrap_or_default();
        let fd = fd.unwrap_or_default();
        if fd < sd {
            return;
        }
        let nd = (fd-sd).num_days();
        if  nd == 0 {
            return;
        }
        let sv = self.start_amount.trim().parse::<f64>();
        let fv = self.final_amount.trim().parse::<f64>();
        if sv.is_err() || fv.is_err() {
            return;
        }
        let sv = sv.unwrap_or_default();
        let fv = fv.unwrap_or_default();
        let cc = ((fv/sv).powf(1.0 / (f64::from(i32::try_from(nd).unwrap_or(0)) / 365.25)) - 1.0) * 100.0;
        let dp = match cc {
            0.0..100.0 => 1,
            _ => 0
        };
        self.cagr = format!("{cc:.dp$}");
    }

    fn redo_amount (&mut self) {
        let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT);
        let fd = NaiveDate::parse_from_str(&self.final_date, DATEFORMAT);
        if  sd.is_err() || fd.is_err() {
            return;
        }
        let sd = sd.unwrap_or_default();
        let fd = fd.unwrap_or_default();
        if fd < sd {
            return;
        }
        let nd = (fd-sd).num_days();
        if  nd == 0 {
            return;
        }
        let sv = self.start_amount.trim().parse::<f64>();
        let cc = self.cagr.trim().parse::<f64>();
        if sv.is_err() || cc.is_err() {
            return;
        }
        let sv = sv.unwrap_or_default();
        let cc = cc.unwrap_or_default() / 100.0;
        let fv = sv * (1.0 + cc).powf(f64::from(i32::try_from(nd).unwrap_or(0)) / 365.25);
        self.final_amount = fv.round().to_string();
    }

}

impl Default for Compounder 
{
    fn default() -> Self {
        let dt = chrono::Local::now().date_naive();
        Self {
            start_date: dt.to_string(),
            final_date: dt.checked_add_months(chrono::Months::new(12)).unwrap_or_default().to_string(),
            years: 1,
            months: 0,
            weeks: 0,
            days: 0,
            start_amount: String::from("1000"),
            final_amount: String::from("1100"),
            cagr: String::from("10"),
            ui_size: InterfaceSize::Small,
            ui_mode: InterfaceMode::Dark
        }
    }
}

impl App for Compounder 
{
    fn save (&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update (&mut self, context: &egui::Context, _frame: &mut Frame) {
        let start_is_valid = self.valid_start();
        let final_is_valid = self.valid_final();
        let range_is_valid = self.valid_range();
        egui::CentralPanel::default().frame(self.get_frame()).show(context, |ui| {
            let styles = ui.style_mut();
            styles.spacing.item_spacing = egui::Vec2::new(16.0, 8.0);
            styles.spacing.text_edit_width = 75.0;
            // egui::Image::new (egui::include_image!("../assets/Panel-Background.svg")).paint_at(ui, ui.ctx().screen_rect());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("START DATE").small().weak());
                    if ui.add(ErrorField::new(&mut self.start_date, start_is_valid && (!final_is_valid || range_is_valid))).changed() {
                        self.redo_parts();
                    };
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("FINAL DATE").small().weak());
                    if ui.add(ErrorField::new(&mut self.final_date, final_is_valid && (!start_is_valid || range_is_valid))).changed() {
                        self.redo_parts();
                    };
                });
                ui.add_space(36.0);
                ui.vertical(|ui| {
                    ui.add_space(12.0);
                    if ui.add(egui::Slider::new(&mut self.years,  0..=50).text("years")).changed() {
                        self.redo_final();
                    };
                    if ui.add(egui::Slider::new(&mut self.months, 0..=11).text("months")).changed() {
                        self.redo_final();
                    };
                    if ui.add(egui::Slider::new(&mut self.weeks,  0..=4).text("weeks")).changed() {
                        self.redo_final();
                    };
                    if ui.add(egui::Slider::new(&mut self.days,   0..=6).text("days")).changed() {
                        self.redo_final();
                    };
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("START AMOUNT").small().weak());
                    if ui.text_edit_singleline(&mut self.start_amount).highlight().changed() {
                        self.redo_cagr();
                    }
                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("FINAL AMOUNT").small().weak());
                            if ui.text_edit_singleline(&mut self.final_amount).highlight().changed() {
                                self.redo_cagr();
                            };
                        });
                        ui.label(egui::RichText::new("\n  =  ").strong());
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("CAGR").small().weak());
                            if ui.text_edit_singleline(&mut self.cagr).highlight().changed() {
                                self.redo_amount();
                            };
                        });
                    });
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("DARK MODE").small().weak());
                    if ui.add(Switch::new(InterfaceMode::Dark == self.ui_mode)).clicked() {
                        match self.ui_mode {
                            InterfaceMode::Dark  => self.remode(ui.ctx(), InterfaceMode::Light),
                            InterfaceMode::Light => self.remode(ui.ctx(), InterfaceMode::Dark)
                        }
                    };
                });
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("TEXT SIZE").small().weak());
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.ui_size == InterfaceSize::Small,  "small" ).highlight().clicked() {
                            self.resize(ui.ctx(), InterfaceSize::Small);
                        };
                        if ui.selectable_label(self.ui_size == InterfaceSize::Medium, "medium").highlight().clicked() {
                            self.resize(ui.ctx(), InterfaceSize::Medium);
                        };
                        if ui.selectable_label(self.ui_size == InterfaceSize::Large,  "large" ).highlight().clicked() {
                            self.resize(ui.ctx(), InterfaceSize::Large);
                        };
                    });
                });
            });
        });
    }
}

fn date_difference(sd: NaiveDate, fd: NaiveDate) -> (u8, u8, u8, u8) {
    // Solution suggested by ChatGPT (added number of weeks and adjusted remaining days accordingly).
    use chrono::Datelike;
    let mut yn = fd.year() - sd.year();
    let mut mn = i32::try_from(fd.month()).unwrap_or(0) - i32::try_from(sd.month()).unwrap_or(0);
    let mut dn = i32::try_from(fd.day()).unwrap_or(0) - i32::try_from(sd.day()).unwrap_or(0);

    if dn < 0 {
        mn -= 1;
        let mp = if fd.month() == 1 { 12 } else { fd.month() - 1 };
        let pn = days_in_month(fd.year(), mp);
        dn += i32::try_from(pn).unwrap_or(0);
        if  dn < 0 { // Rare cases like january 31st to march 1st on leap years.
            dn = 1;
        }
    }
    if mn < 0 {
        yn -= 1;
        mn += 12;
    }
    (
        u8::try_from(yn).unwrap_or(0), 
        u8::try_from(mn).unwrap_or(0), 
        u8::try_from(dn / 7).unwrap_or(0), 
        u8::try_from(dn % 7).unwrap_or(0)
    )
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}


fn main() -> eframe::Result {
    // let factorial = | n | (1..=n).product::<i32>(); // Nice!
    // println!("{}", factorial(5));
    eframe::run_native(
        "Compounder", 
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_resizable(false)
                .with_maximize_button(false)
                .with_inner_size(GUI_SIZE)
                .with_icon(eframe::icon_data::from_png_bytes(&include_bytes!("../assets/Compounder.png")[..]).unwrap_or_default()),
            ..Default::default()
        },
        Box::new(|context| {
            Ok(Box::new(Compounder::new(context)))
        })
    )
}
