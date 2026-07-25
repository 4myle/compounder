
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

const GUI_SIZE: egui::Vec2 = egui::Vec2::new(400.0, 370.0);
const ACCENT_COLOR: egui::Color32 = egui::Color32::from_rgb(170, 0, 204);
const DATEFORMAT: &str = "%Y-%m-%d";

use chrono::NaiveDate;
use eframe:: { 
    egui,
    App, 
    Frame
};

mod switch;
mod errorfield;

use switch::Switch;
use errorfield::ErrorField;

fn date_difference (sd: NaiveDate, fd: NaiveDate) -> (u8, u8, u8, u8) {
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

fn days_in_month (year: i32, month: u32) -> u32 {
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

fn is_leap_year (year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Compounder 
{
    id: i64,
    title: String,
    is_open: bool,
    start_date: String,
    final_date: String,
    follow_today: bool,
    years: u8,
    months: u8,
    weeks: u8,
    days: u8,
    start_amount: String,
    final_amount: String,
    cagr: String
}

impl Default for Compounder 
{
    fn default() -> Self {
        let dt = chrono::Local::now().date_naive();
        Self {
            // id: chrono::Local::now().timestamp_subsec_millis(),
            id: chrono::Local::now().timestamp_millis(),
            title: String::from("<New Compounder>"),
            is_open: true,
            start_date: dt.to_string(),
            final_date: dt.checked_add_months(chrono::Months::new(12)).unwrap_or_default().to_string(),
            follow_today: false,
            years: 1,
            months: 0,
            weeks: 0,
            days: 0,
            start_amount: String::from("1000"),
            final_amount: String::from("1100"),
            cagr: String::from("10")
        }
    }
}

impl Compounder 
{
    fn new () -> Self {
        let view: Compounder = Compounder::default();
        view
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

    fn show (&mut self, ui: &mut egui::Ui, open: &mut bool) {
        let start_is_valid = self.valid_start();
        let final_is_valid = self.valid_final();
        let range_is_valid = self.valid_range();
        egui::CentralPanel::default().show(ui, |ui| {
            let style = ui.style_mut();
            style.spacing.item_spacing = egui::Vec2::new(16.0, 8.0);
            style.spacing.text_edit_width = 85.0;
            ui.horizontal(|ui| {
                ui.scope(|ui| { // Let style changes only effect title field.
                    let style = ui.style_mut();
                    style.spacing.text_edit_width = GUI_SIZE.x-60.0;
                    style.override_text_style = Some(egui::TextStyle::Heading);
                    style.visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
                    ui.add(ErrorField::new(&mut self.title, true));
                });
                if ui.button("\u{2717}").clicked() {
                    *open = false;
                }
            });
            ui.separator();
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("START DATE").small().weak());
                    if ui.add(ErrorField::new(&mut self.start_date, start_is_valid && (!final_is_valid || range_is_valid))).highlight().changed() {
                        self.redo_parts();
                    }
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("FINAL DATE").small().weak());
                    ui.checkbox(&mut self.follow_today, "Use today");
                    if self.follow_today {
                        self.final_date = chrono::Local::now().date_naive().to_string();
                        self.redo_parts();
                        ui.add_enabled_ui(false, |ui| {
                            ui.add(ErrorField::new(&mut self.final_date, final_is_valid && (!start_is_valid || range_is_valid))).highlight()
                        });
                    } else if ui.add(ErrorField::new(&mut self.final_date, final_is_valid && (!start_is_valid || range_is_valid))).highlight().changed() {
                        self.redo_parts();
                    }
                });
                ui.add_space(36.0);
                ui.vertical(|ui| {
                    ui.add_space(12.0);
                    if ui.add(egui::Slider::new(&mut self.years,  0..=50).text("years")).changed() {
                        self.redo_final();
                    }
                    if ui.add(egui::Slider::new(&mut self.months, 0..=11).text("months")).changed() {
                        self.redo_final();
                    }
                    if ui.add(egui::Slider::new(&mut self.weeks,  0..=4).text("weeks")).changed() {
                        self.redo_final();
                    }
                    if ui.add(egui::Slider::new(&mut self.days,   0..=6).text("days")).changed() {
                        self.redo_final();
                    }
                    let sd = NaiveDate::parse_from_str(&self.start_date, DATEFORMAT).unwrap_or_default();
                    let fd = NaiveDate::parse_from_str(&self.final_date, DATEFORMAT).unwrap_or_default();
                    ui.label(egui::RichText::new(format!("{} days in total", (fd-sd).num_days())).italics());
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
                            }
                        });
                        ui.label(egui::RichText::new("\n  =  ").strong());
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("CAGR").small().weak());
                            if ui.text_edit_singleline(&mut self.cagr).highlight().changed() {
                                self.redo_amount();
                            }
                        });
                    });
                });
            });
        });
    }

}


#[derive(serde::Deserialize, serde::Serialize, PartialEq, Copy, Clone)]
enum InterfaceMode
{
    Dark,
    Light
}

#[derive(serde::Deserialize, serde::Serialize)]
struct Mainview 
{
    windows: Vec<Compounder>,
    ui_mode: InterfaceMode,
    ui_size: f32
}

impl Default for Mainview 
{
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            ui_mode: InterfaceMode::Dark,
            ui_size: 1.1

        }
    }
}

impl Mainview
{
    fn new (context: &eframe::CreationContext<'_>) -> Self {
        // egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut view: Mainview = if let Some(ps) = context.storage { eframe::get_value(ps, eframe::APP_KEY).unwrap_or_default() } else { Mainview::default() };
        if  view.windows.is_empty() {
            view.windows.push(Compounder::new());
        }
        Self::set_fonts(&context.egui_ctx);
        Self::set_style(&context.egui_ctx, view.ui_mode);
        context.egui_ctx.set_zoom_factor(view.ui_size); // Adjust size on initialization.
        view
    }

    fn get_frame (&self) -> egui::Frame {
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
        }
    }
    
    fn set_style (context: &egui::Context, mode: InterfaceMode) {
        let mut visuals: egui::Visuals;
        match mode {
            InterfaceMode::Dark  => {
                context.set_theme(egui::Theme::Dark);
                visuals = egui::Visuals::dark();
                visuals.override_text_color = Option::Some(egui::Color32::from_gray(255));
            },
            InterfaceMode::Light => {
                context.set_theme(egui::Theme::Light);
                visuals = egui::Visuals::light();
                visuals.override_text_color = Option::Some(egui::Color32::from_gray(0));
            }
        }
        visuals.widgets.active.bg_fill = ACCENT_COLOR;
        visuals.widgets.noninteractive.bg_fill = ACCENT_COLOR;
        visuals.selection.bg_fill = ACCENT_COLOR.gamma_multiply(0.6);
        visuals.widgets.hovered.bg_fill = ACCENT_COLOR;
        visuals.selection.stroke = egui::Stroke::new(1.0, ACCENT_COLOR.lerp_to_gamma(egui::Color32::WHITE, 0.5));
        visuals.widgets.hovered.weak_bg_fill = ACCENT_COLOR.gamma_multiply(0.1);
        visuals.slider_trailing_fill = true;
        visuals.window_shadow.offset = [4,4];
        context.set_visuals(visuals);
    }

    fn ui_topbar (&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("TEXT SIZE").small().weak());
                if ui.add(egui::Slider::new(&mut self.ui_size, 0.7..=1.7)).changed() {
                    ui.ctx().set_zoom_factor(self.ui_size);
                }
            });
            ui.add_space(24.0);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("DARK MODE").small().weak());
                if ui.add(Switch::new(InterfaceMode::Dark == self.ui_mode)).clicked() {
                    match self.ui_mode {
                        InterfaceMode::Dark  => { 
                            self.ui_mode = InterfaceMode::Light;
                            Self::set_style(ui.ctx(), InterfaceMode::Light);
                        },
                        InterfaceMode::Light => { 
                            self.ui_mode = InterfaceMode::Dark;
                            Self::set_style(ui.ctx(), InterfaceMode::Dark);
                        }
                    }
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let styles = ui.style_mut();
                styles.spacing.button_padding = egui::Vec2::new(12.0, 8.0);
                if ui.button("Add compunder").clicked() {
                    self.windows.push(Compounder::new());
                }
            });
        });
    }
}

impl App for Mainview 
{
    fn save (&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui (&mut self, ui: &mut egui::Ui, _frame: &mut Frame) {
        egui::Panel::top("Topbar").frame(self.get_frame()).resizable(false).show(ui, |ui| {
            self.ui_topbar(ui);
        });
        // egui::Image::new (egui::include_image!("../assets/Panel-Background.svg")).paint_at(ui, ui.ctx().content_rect());
        egui::CentralPanel::default().frame(self.get_frame()).show(ui, |ui| {
            self.windows.retain_mut(|window| {
                if !window.is_open {
                    return false;
                }
                let mut open = true;
                egui::Window::new(&window.title)
                    .id(egui::Id::new(window.id))
                    // .default_pos(egui::pos2(ui.min_rect().min.x, ui.min_rect().min.y))
                    .title_bar(false)
                    .resizable(false)
                    .fixed_size(GUI_SIZE)
                    .show(ui, |ui| window.show(ui, &mut open));
                window.is_open = open;
                true
            });
        });
    }
    
}

fn main() -> eframe::Result {
    // let factorial = | n | (1..=n).product::<i32>(); // Nice!
    eframe::run_native(
        "Compounder", 
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_icon(eframe::icon_data::from_png_bytes(&include_bytes!("../assets/Compounder.png")[..]).unwrap_or_default())
                .with_inner_size([1024.0, 768.0]),
            ..Default::default()
        },
        Box::new(|context| {
            Ok(Box::new(Mainview::new(context)))
        })
    )
}
