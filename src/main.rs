mod screenshots_module;
mod hotkey_module;
mod settings_module;
mod state_module;

use eframe::{NativeOptions, egui, IconData};
use crate::state_module::state_module::ScreenshotStr;

fn build_gui() -> () {
    let icon = image::open("./resources/icon.png").expect("Failed to open icon path").to_rgba8();
    let (icon_width, icon_height) = icon.dimensions();

    //FONT CONF
    // Start with the default fonts (we will be adding to them rather than replacing them).
    let mut fonts = egui::FontDefinitions::default();
    //Install font (maybe supporting non-latin characters).
    fonts.font_data.insert(
        "Arial".to_owned(),
        egui::FontData::from_static(include_bytes!("../resources/fonts/ARIALN.TTF")),
    );
    // Put my font first (highest priority) for proportional text:
    fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Arial".to_owned());



    //APP CONF
    let options = NativeOptions {
        initial_window_size: Some(egui::vec2(650.0, 410.0)),
        min_window_size: Some(egui::vec2(650.0, 410.0)),
        icon_data: Some(IconData{
            rgba: icon.into_raw(),
            width: icon_width,
            height: icon_height,
        }),

        ..Default::default()
    };


    println!("Starting app");
    eframe::run_native(
        "Rusty Capture",
        options,
        Box::new(|_cc| {
            _cc.egui_ctx.set_fonts(fonts);
            Box::<ScreenshotStr>::new(ScreenshotStr::default())
        }),
    ).unwrap();
    println!("closing eframe");
}


fn main() {
    //HOTKEYS
    build_gui();
}
