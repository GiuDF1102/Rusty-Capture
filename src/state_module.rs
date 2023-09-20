#![allow(dead_code)]
pub mod state_module {
    use std::{error::Error, fmt::{Display, Formatter}};
    use eframe::egui::Context;
    use std::time::{Duration, Instant};
    use eframe::egui::Margin;
    use image::{EncodableLayout, ImageFormat};
    use crate::screenshots_module::screenshot_module::Screenshot;
    use crate::settings_module::settings_module::*;
    use screenshots::Screen;
    use eframe::{egui::{CentralPanel, Layout, Align, TextEdit, Direction, Key, Window, ComboBox, TopBottomPanel, CursorIcon}, App, epaint::{ColorImage, Vec2, Pos2}, egui};
    use std::{cmp, path::PathBuf};
    use eframe::egui::{Color32, Frame, Rect, Slider};
    use eframe::epaint::Stroke;
    use global_hotkey::GlobalHotKeyEvent;
    use global_hotkey::hotkey::Modifiers;
    use rusttype::Scale;
    use crate::hotkey_module::hotkey_module::{ActiveShortcuts, HotkeyManager, KeyType};


    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DrawingMode {
        Paint,
        Highlight,
        Erase,
        Shape,
        Text,
        Pause,
        Crop,
    }

    impl Display for DrawingMode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                DrawingMode::Paint => write!(f, "Paint"),
                DrawingMode::Highlight => write!(f, "Highlight"),
                DrawingMode::Erase => write!(f, "Erase"),
                DrawingMode::Shape => write!(f, "Shape"),
                DrawingMode::Text => write!(f, "Text"),
                DrawingMode::Pause => write!(f, "Pause"),
                DrawingMode::Crop => write!(f, "Crop"),
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Shape {
        Rectangle,
        Circle,
        Arrow,
    }

    pub struct ScreenshotStr {
        pub timer: usize,
        pub screen: usize,
        pub screenshot: Screenshot,
        pub format: ImageFormat,
        pub color_image: ColorImage,
        pub show_image: bool,
        pub error_dialog: bool,
        pub error_message: String,
        pub save_dialog: bool,
        pub drawing_mode: Option<DrawingMode>,
        pub previous_drawing_mode: Option<DrawingMode>,
        pub previous_drawing_mode_error: Option<DrawingMode>,
        pub text_edit_dialog: bool,
        pub text_edit_dialog_position: Pos2,
        pub text: String,
        pub shape: Option<Shape>,
        pub tool_color: [u8; 3],
        pub tool_size: f32,
        pub settings_dialog: bool,
        pub settings: Settings,
        pub instant: Instant,
        pub starting_point: Option<(f32, f32)>,
        pub upper_panel_size: Vec2,
        pub screen_state: u8,
        pub screenshot_taken: bool,
        pub image_converted: bool,
        pub window_pos: Pos2,
        pub window_size: Vec2,
        pub crop_screenshot_tmp: Screenshot,
        pub saved_to_clipboard_dialog: bool,
        pub hotkey_manager: HotkeyManager,
    }

    impl Default for ScreenshotStr {
        fn default() -> Self {
            let mut tmp = HotkeyManager::new().unwrap();
            let startup_settings = read_settings_from_file("settings.json".to_string()).unwrap();
            let key_quick = startup_settings.get_quick_hotkey();
            let key_screenshot = startup_settings.get_new_screenshot_hotkey();
            let key_pen = startup_settings.get_pen_hotkey();
            let key_rubber = startup_settings.get_rubber_hotkey();
            let key_save = startup_settings.get_save_hotkey();

            tmp.register_new_hotkey(Some(Modifiers::CONTROL), key_screenshot.unwrap(), KeyType::NewScreenshot).unwrap();
            tmp.register_new_hotkey(Some(Modifiers::CONTROL), key_quick.unwrap(), KeyType::Quick).unwrap();
            tmp.register_new_hotkey(Some(Modifiers::CONTROL), key_pen.unwrap(), KeyType::Pen).unwrap();
            tmp.register_new_hotkey(Some(Modifiers::CONTROL), key_rubber.unwrap(), KeyType::Rubber).unwrap();
            tmp.register_new_hotkey(Some(Modifiers::CONTROL), key_save.unwrap(), KeyType::Save).unwrap();
            tmp.set_active_shortcuts(ActiveShortcuts::ScreenshotWaiting).unwrap();
            Self {
                timer: 0,
                screen: 0,
                screenshot: Screenshot::new_empty(),
                format: ImageFormat::Png,
                color_image: ColorImage::example(),
                show_image: false,
                error_dialog: false,
                error_message: String::new(),
                save_dialog: false,
                drawing_mode: None,
                previous_drawing_mode: Some(DrawingMode::Pause),
                previous_drawing_mode_error: Some(DrawingMode::Pause),
                text_edit_dialog: false,
                text_edit_dialog_position: Pos2::new(0.0, 0.0),
                text: String::new(),
                shape: Some(Shape::Rectangle),
                tool_color: [0, 0, 0],
                tool_size: 10.0,
                settings_dialog: false,
                settings: Settings::default(),
                instant: Instant::now(),
                starting_point: None,
                upper_panel_size: Vec2::new(0.0, 0.0),
                screen_state: 0,
                screenshot_taken: false,
                image_converted: false,
                window_pos: Pos2::new(0.0, 0.0),
                window_size: Vec2::new(0.0, 0.0),
                crop_screenshot_tmp: Screenshot::new_empty(),
                saved_to_clipboard_dialog: false,
                hotkey_manager: tmp,
            }
        }
    }

    impl ScreenshotStr {
        //front
        pub fn toggle_drawing_mode(&mut self, mode: DrawingMode) {
            if self.drawing_mode == Some(mode) {
                self.drawing_mode = None;
            } else {
                self.drawing_mode = Some(mode);
            }
            self.show_image = true;
        }

        pub fn convert_image(&mut self) -> () {
            let image = self.screenshot.get_image().unwrap();
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let col_im: ColorImage = ColorImage::from_rgba_unmultiplied(
                size,
                image_buffer.as_bytes(),
            );

            self.color_image = col_im;
            self.image_converted = true;
        }

        pub fn calculate_texture_coordinates(&self, cursor_pos: Pos2, available: Vec2, total_window: Vec2, return_always: bool) -> Option<Pos2> {
            let w = self.screenshot.get_width().unwrap() as f32;
            let h = self.screenshot.get_height().unwrap() as f32;
            //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
            let w_window = available.x;
            let h_window = available.y;
            let aspect_ratio = w / h;
            let height = h_window.min(w_window / aspect_ratio);
            let width = height * aspect_ratio;
            let h_scale = height / h;
            let w_scale = width / w;
            let image_pos_x = (total_window.x - width) / 2.0;
            let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y + (h_window - height) / 2.0;
            let image_cursor_pos = Pos2 {
                x: (cursor_pos.x - image_pos_x) / w_scale,
                y: (cursor_pos.y - image_pos_y) / h_scale,
            };
            if image_cursor_pos.x > w || image_cursor_pos.y > h || image_cursor_pos.y < 0.0 || image_cursor_pos.x < 0.0 {
                if !return_always {
                    None
                } else {
                    Some(image_cursor_pos)
                }
            } else {
                Some(image_cursor_pos)
            }
        }

        pub fn calculate_rect_image(&self, available: Vec2, total_window: Vec2) -> (f32, f32, f32, f32, f32, f32) {
            let w = self.screenshot.get_width().unwrap() as f32;
            let h = self.screenshot.get_height().unwrap() as f32;
            //x is 640 out of 640, y is 356 out of 400 (-22*2 equal to borders+top and bottom)
            let w_window = available.x;
            let h_window = available.y;
            let height = h_window.min(w_window * h / w);
            let width = height * w / h;
            let h_scale = height / h;
            let w_scale = width / w;
            let image_pos_x = (total_window.x - width) / 2.0;
            let image_pos_y = self.upper_panel_size.y + Margin::same(1.0).sum().y + (h_window - height) / 2.0;
            return (image_pos_x, image_pos_y, width, height, w_scale, h_scale);
        }

        pub fn draw_paint(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8; 4]) -> bool {
            ctx.input(|is| -> bool {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if texture_coordinates.is_some() {
                        let texture_coordinates = texture_coordinates.unwrap();
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
                        if is.pointer.any_down() {
                            if self.starting_point.is_none() {
                                self.starting_point = Some((x, y));
                            } else {
                                self.screenshot.draw_line(
                                    self.starting_point.unwrap(),
                                    (x, y),
                                    color,
                                    size,
                                );
                                self.starting_point = Some((x, y));
                                self.conversion();
                            }
                        } else {
                            self.starting_point = None;
                        }
                        return true;
                    } else {
                        self.starting_point = None;
                    }
                }
                return false;
            })
        }

        pub fn draw_highlight(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8; 3]) -> bool {
            ctx.input(|is| -> bool {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if texture_coordinates.is_some() {
                        let texture_coordinates = texture_coordinates.unwrap();
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
                        if is.pointer.any_down() {
                            if self.starting_point.is_none() {
                                self.starting_point = Some((x, y));
                            } else {
                                self.screenshot.highlight_line(
                                    self.starting_point.unwrap(),
                                    (x, y),
                                    size,
                                    color,
                                );
                                let mut dx = 1.0;
                                if self.starting_point.unwrap().0 > x { dx = -1.0 }
                                self.starting_point = Some((x + dx, y));
                                self.conversion();
                            }
                        } else {
                            self.starting_point = None;
                        }
                        return true;
                    } else {
                        self.starting_point = None;
                    }
                } else {
                    self.starting_point = None;
                }
                return false;
            })
        }

        pub fn erase(&mut self, ctx: &Context, available: Vec2, size: f32) -> bool {
            ctx.input(|ui| -> bool{
                let pos = ui.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if texture_coordinates.is_some() {
                        let texture_coordinates = texture_coordinates.unwrap();
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;

                        if ui.pointer.any_down() {
                            self.screenshot.erase_point(x, y, size);
                            self.conversion();
                        }
                        return true;
                    }
                }
                self.starting_point = None;
                return false;
            })
        }

        pub fn draw_rectangle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8; 4]) -> Option<((f32, f32), (f32, f32))> {
            return ctx.input(|is| {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if let Some(texture_coordinates) = texture_coordinates {
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
                        if is.pointer.any_down() {
                            if self.starting_point.is_none() {
                                self.starting_point = Some((x, y));
                            } else {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.rectangle(start, end, size, color);
                                self.conversion();
                                return None;
                            }
                        } else {
                            if self.starting_point.is_some() {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.rectangle(start, end, size, color);
                                self.conversion();
                                let tmp = self.starting_point.take().unwrap();
                                self.screenshot.save_intermediate_image().unwrap();
                                return Some((tmp, (x, y)));
                            }
                            return None;
                        }
                    } else {
                        self.starting_point = None;
                        self.screenshot.rollback_changes();
                        self.conversion();
                    }
                } else {
                    self.starting_point = None;
                    self.screenshot.rollback_changes();
                    self.conversion();
                }
                return None;
            });
        }

        pub fn draw_circle(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8; 4]) {
            ctx.input(|is| {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if let Some(texture_coordinates) = texture_coordinates {
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
                        if is.pointer.any_down() {
                            if self.starting_point.is_none() {
                                self.starting_point = Some((x, y));
                            } else {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.circle(start, end, size, color);
                                self.conversion();
                            }
                        } else {
                            if self.starting_point.is_some() {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.circle(start, end, size, color);
                                self.conversion();
                            }
                            self.starting_point = None;
                            self.screenshot.save_intermediate_image().unwrap();
                        }
                    } else {
                        self.starting_point = None;
                        self.screenshot.rollback_changes();
                        self.conversion();
                    }
                }else{
                    self.starting_point = None;
                    self.screenshot.rollback_changes();
                    self.conversion();
                }
            });
        }

        pub fn draw_arrow(&mut self, ctx: &Context, available: Vec2, size: f32, color: [u8; 4]) -> Option<((f32, f32), (f32, f32))> {
            return ctx.input(|is| {
                let pos = is.pointer.interact_pos();
                if let Some(pos) = pos {
                    let texture_coordinates = self.calculate_texture_coordinates(pos, available, ctx.used_size(), false);
                    if let Some(texture_coordinates) = texture_coordinates {
                        let x = texture_coordinates.x;
                        let y = texture_coordinates.y;
                        if is.pointer.any_down() {
                            if self.starting_point.is_none() {
                                self.starting_point = Some((x, y));
                            } else {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.arrow(start, end, size, color);
                                self.conversion();
                                return None;
                            }
                        } else {
                            if self.starting_point.is_some() {
                                let start = (
                                    self.starting_point.unwrap().0,
                                    self.starting_point.unwrap().1,
                                );
                                let end = (x, y);
                                self.screenshot.arrow(start, end, size, color);
                                self.conversion();
                                let tmp = self.starting_point.take().unwrap();
                                self.screenshot.save_intermediate_image().unwrap();
                                return Some((tmp, (x, y)));
                            }
                            return None;
                        }
                    } else {
                        self.starting_point = None;
                        self.screenshot.rollback_changes();
                        self.conversion();
                    }
                }else{
                    self.starting_point = None;
                    self.screenshot.rollback_changes();
                    self.conversion();
                }
                return None;
            });
        }

        fn conversion(&mut self) {
            if Instant::now() > self.instant {
                self.convert_image();
                self.instant += Duration::from_millis(5);
            }
        }

        pub fn check_minimization(&mut self, frame: &mut eframe::Frame) {
            if self.screenshot_taken {
                match self.screen_state {
                    0 => {
                        frame.set_window_pos(Pos2::new(-200.0, -200.0));
                        frame.set_window_size(Vec2::new(0.0, 0.0));
                        if frame.info().window_info.position.unwrap().x == -200.0 && frame.info().window_info.position.unwrap().y == -200.0 {
                            self.screen_state = 1;
                        }
                    }
                    1 => {
                        let duration = Duration::from_secs(self.timer as u64);
                        self.screenshot = take_screenshot(duration, self.screen);
                        self.crop_screenshot_tmp=self.screenshot.clone();
                        self.starting_point=None;
                        self.convert_image();
                        self.show_image = true;
                        if self.image_converted {
                            self.screen_state = 2;
                        }
                    }
                    2 => {
                        frame.set_window_pos(self.window_pos);
                        frame.set_window_size(self.window_size);
                        let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::ScreenshotDone);
                        self.manage_errors(result);
                        self.screen_state = 0;
                        self.screenshot_taken = false;
                    }
                    _ => {}
                }
            }
        }

        pub fn manage_errors<E>(&mut self, result: Result<E, Box<dyn Error>>) -> Option<E> {
            match result {
                Ok(value) => Some(value),
                Err(e) => {
                    self.previous_drawing_mode_error = self.drawing_mode;
                    self.drawing_mode = None;
                    self.error_message = e.to_string();
                    self.error_dialog = true;
                    None
                }
            }
        }
    }

    impl App for ScreenshotStr {
        fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
            //shortcuts
            if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                //KEY_QUICK
                if self.hotkey_manager.get_key(KeyType::Quick).is_some() && self.hotkey_manager.get_key(KeyType::Quick).unwrap() == event.id {
                    println!("Screenshot taken");
                    let startup_settings = read_settings_from_file("settings.json".to_string());
                    let startup_settings = self.manage_errors(startup_settings);
                    if startup_settings.is_some() {
                        let ss = take_screenshot(Duration::from_secs(0), 0);
                        let result = ss.save_image(&PathBuf::from(startup_settings.unwrap().path), ImageFormat::Png);
                        self.manage_errors(result);
                    }
                }
                //KEY_SCREENSHOT
                if self.hotkey_manager.get_key(KeyType::NewScreenshot).is_some() && self.hotkey_manager.get_key(KeyType::NewScreenshot).unwrap() == event.id {
                    self.window_size = frame.info().window_info.size;
                    self.window_pos = frame.info().window_info.position.unwrap();
                    self.screenshot_taken = true;
                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                    self.manage_errors(result);
                }
                //KEY_SAVE
                if self.hotkey_manager.get_key(KeyType::Save).is_some() && self.hotkey_manager.get_key(KeyType::Save).unwrap() == event.id {
                    if !self.saved_to_clipboard_dialog && !self.settings_dialog && !self.save_dialog {
                        if self.drawing_mode == Some(DrawingMode::Crop) || self.drawing_mode == Some(DrawingMode::Shape){
                            self.screenshot.rollback_changes();
                            self.conversion();
                        }
                        self.previous_drawing_mode = self.drawing_mode;
                        self.drawing_mode = None;
                    }
                    self.starting_point=None;
                    self.saved_to_clipboard_dialog = false;
                    self.text_edit_dialog = false;
                    self.settings_dialog = false;
                    self.save_dialog = true;
                }
                //KEY_PEN
                if self.hotkey_manager.get_key(KeyType::Pen).is_some() && self.hotkey_manager.get_key(KeyType::Pen).unwrap() == event.id {
                    if self.drawing_mode == Some(DrawingMode::Crop){
                        self.screenshot.rollback_changes();
                        self.conversion();
                    }
                    self.starting_point=None;
                    self.drawing_mode = Some(DrawingMode::Paint);
                }
                //KEY_RUBBER
                if self.hotkey_manager.get_key(KeyType::Rubber).is_some() && self.hotkey_manager.get_key(KeyType::Rubber).unwrap() == event.id {
                    if self.drawing_mode == Some(DrawingMode::Crop){
                        self.screenshot.rollback_changes();
                        self.conversion();
                    }
                    self.starting_point=None;
                    self.drawing_mode = Some(DrawingMode::Erase);
                }
            }

            //DIALOGS

            // SAVE_DIALOG
            if self.save_dialog {
                Window::new("Save Screenshot")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_enabled(!self.error_dialog);

                        //close button
                        ui.horizontal(|ui| {
                            ui.label("Save as?");
                            if ui.button("PNG").clicked() {
                                self.format = ImageFormat::Png;
                                //error handling
                                let result = self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.save_dialog = false;
                                self.drawing_mode = self.previous_drawing_mode;
                            }
                            if ui.button("JPG").clicked() {
                                self.format = ImageFormat::Jpeg;
                                let result = self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.save_dialog = false;
                                self.drawing_mode = self.previous_drawing_mode;
                            }
                            if ui.button("GIF").clicked() {
                                self.format = ImageFormat::Gif;
                                let result = self.screenshot.save_image(&PathBuf::from(&self.settings.path), self.format);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.save_dialog = false;
                                self.drawing_mode = self.previous_drawing_mode;
                            }
                        });

                        //close
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.drawing_mode = self.previous_drawing_mode;
                                self.save_dialog = false;
                            }
                        });
                    });
            }


            if self.saved_to_clipboard_dialog {
                Window::new("Save Screenshot")
                    .collapsible(false)
                    .title_bar(false)
                    .resizable(false)
                    .movable(false)
                    .show(ctx, |ui| {
                        ui.set_enabled(!self.error_dialog);

                        if ui.label("Saved to clipboard!").clicked_elsewhere() {
                            self.saved_to_clipboard_dialog = false;
                        };
                        if ui.button("Ok").clicked() {
                            self.saved_to_clipboard_dialog = false;
                            self.drawing_mode = self.previous_drawing_mode;
                        }
                    });
            }

            // SETTING_DIALOG
            if self.settings_dialog {
                Window::new("Settings")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.set_enabled(!self.error_dialog);

                        ui.label("Change Hotkeys");
                        ui.horizontal(|ui| {
                            ui.label("Quick Screenshot");
                            ui.label("CTRL + ");
                            ui.add(TextEdit::singleline(&mut self.settings.quick)
                                .char_limit(1)
                                .desired_width(ui.available_width() / 4.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("New Screenshot");
                            ui.label("CTRL + ");
                            ui.add(TextEdit::singleline(&mut self.settings.new_screenshot)
                                .char_limit(1)
                                .desired_width(ui.available_width() / 4.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Save Screenshot");
                            ui.label("CTRL + ");
                            ui.add(TextEdit::singleline(&mut self.settings.save)
                                .char_limit(1)
                                .desired_width(ui.available_width() / 4.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Pen tool");
                            ui.label("CTRL + ");
                            ui.add(TextEdit::singleline(&mut self.settings.pen)
                                .char_limit(1)
                                .desired_width(ui.available_width() / 4.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Rubber tool");
                            ui.label("CTRL + ");
                            ui.add(TextEdit::singleline(&mut self.settings.rubber)
                                .char_limit(1)
                                .desired_width(ui.available_width() / 4.0));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Path");
                            //turn pathbuf into string
                            ui.add(TextEdit::singleline(&mut self.settings.path));
                        });
                        //close
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.drawing_mode = self.previous_drawing_mode;
                                self.settings_dialog = false;
                            }
                            if ui.button("Save").clicked() {
                                let result = write_settings_to_file("settings.json".to_string(), &self.settings);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                let startup_settings = read_settings_from_file("settings.json".to_string());
                                let result = self.manage_errors(startup_settings);
                                if result.is_none() {
                                    return;
                                }
                                let startup_settings = result.unwrap();
                                //KEY_NEW_SCREENSHOT
                                let key_new_screenshot = startup_settings.get_new_screenshot_hotkey();
                                let result = self.manage_errors(key_new_screenshot);
                                if result.is_none() {
                                    return;
                                }
                                let key_new_screenshot = result.unwrap();
                                let result = self.hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), key_new_screenshot, KeyType::NewScreenshot);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                //KEY_PEN
                                let key_pen = startup_settings.get_pen_hotkey();
                                let result = self.manage_errors(key_pen);
                                if result.is_none() {
                                    return;
                                }
                                let key_pen = result.unwrap();
                                let result = self.hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), key_pen, KeyType::Pen);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                //KEY_RUBBER
                                let key_rubber = startup_settings.get_rubber_hotkey();
                                let result = self.manage_errors(key_rubber);
                                if result.is_none() {
                                    return;
                                }
                                let key_rubber = result.unwrap();
                                let result = self.hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), key_rubber, KeyType::Rubber);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                //KEY_QUICK
                                let key_quick = startup_settings.get_quick_hotkey();
                                let result = self.manage_errors(key_quick);
                                if result.is_none() {
                                    return;
                                }
                                let key_quick = result.unwrap();
                                let result = self.hotkey_manager.register_new_hotkey(Some(Modifiers::CONTROL), key_quick, KeyType::Quick);
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.drawing_mode = self.previous_drawing_mode;
                                self.settings_dialog = false;
                            }
                        });
                    });
            }

            // error dialog
            if self.error_dialog {
                Window::new("Error")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!("Error: {}", self.error_message));
                        ui.horizontal(|ui| {
                            if ui.button("Ok").clicked() {
                                self.drawing_mode = self.previous_drawing_mode_error;
                                self.error_dialog = false;
                            }
                        });
                    });
            }
            self.check_minimization(frame);

            // header of the app
            TopBottomPanel::top("header").frame(
                Frame {
                    inner_margin: Margin::same(1.0),
                    outer_margin: Margin::same(0.0),
                    fill: ctx.style().visuals.panel_fill,
                    ..Default::default()
                }).show(ctx, |ui| {
                ui.set_enabled(!self.error_dialog);

                let timer = self.timer;
                let screen = self.screen;

                let timer_str = format!("{} Seconds", timer);
                let screen_str = format!("Screen {}", screen);
                self.upper_panel_size = ui.available_size();

                ui.horizontal(|ui| {
                    if ui.button("New Screenshot")
                        .on_hover_text(format!("CTRL + {}", self.settings.new_screenshot))
                        .clicked() {
                        self.window_size = frame.info().window_info.size;
                        self.window_pos = frame.info().window_info.position.unwrap();
                        self.screenshot_taken = true;
                        let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                        self.manage_errors(result);
                    }

                    ui.separator();

                    // combo box timer for the screenshot
                    ComboBox::from_label("Timer")
                        .selected_text(timer_str)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut 0, 0, "No Timer").clicked() {
                                self.timer = 0;
                            }
                            if ui.selectable_value(&mut 3, 3, "3 Seconds").clicked() {
                                self.timer = 3;
                            }
                            if ui.selectable_value(&mut 5, 5, "5 Seconds").clicked() {
                                self.timer = 5;
                            }
                            if ui.selectable_value(&mut 10, 10, "10 Seconds").clicked() {
                                self.timer = 10;
                            }
                        });

                    // combo box screen for the screenshot
                    ComboBox::from_label("Screen")
                        .selected_text(screen_str)
                        .show_ui(ui, |ui| {
                            let screens = get_screens();
                            for (index, screen) in screens.iter().enumerate() {
                                if ui.selectable_value(&mut self.screen, index, &format!("Screen {} ({}x{})", index, screen.display_info.height, screen.display_info.width)).clicked() {
                                    self.screen = index;
                                }
                            }
                        });

                    ui.separator();

                    // save button
                    if ui.button("\u{1F4BE}")
                        .on_hover_text(format!("CTRL + {}", self.settings.save))
                        .clicked() {
                        if !self.saved_to_clipboard_dialog && !self.settings_dialog && !self.save_dialog {
                            self.previous_drawing_mode = self.drawing_mode;
                            self.drawing_mode = None;
                        }
                        self.settings_dialog = false;
                        self.saved_to_clipboard_dialog = false;
                        self.text_edit_dialog = false;
                        self.save_dialog = true;
                    }

                    // save to clipboard button
                    if ui.button("\u{1F4CB}").clicked() {
                        let flag = !self.saved_to_clipboard_dialog && !self.settings_dialog && !self.save_dialog;
                        self.settings_dialog = false;
                        self.save_dialog = false;
                        self.text_edit_dialog = false;
                        let result = self.screenshot.save_to_clipboard();
                        if self.manage_errors(result).is_none() {
                            return;
                        }
                        if flag {
                            self.previous_drawing_mode = self.drawing_mode;
                            self.drawing_mode = None;
                        }
                        self.saved_to_clipboard_dialog = true;
                    }
                    // settings button in the top right corner
                    ui.with_layout(Layout::top_down(Align::RIGHT), |ui| {
                        if ui.button("\u{2699}").clicked() {
                            let flag = !self.saved_to_clipboard_dialog && !self.settings_dialog && !self.save_dialog;
                            self.saved_to_clipboard_dialog = false;
                            self.save_dialog = false;
                            self.text_edit_dialog = false;
                            let result = read_settings_from_file("settings.json".to_string());
                            let result = self.manage_errors(result);
                            if result.is_none() {
                                return;
                            }
                            self.settings = result.unwrap();
                            if flag {
                                self.previous_drawing_mode = self.drawing_mode;
                                self.drawing_mode = None;
                            }
                            self.settings_dialog = true;
                        }
                    });
                });
            });

            // footer of the app
            TopBottomPanel::bottom("footer")
                .frame(
                    Frame {
                        inner_margin: Margin::same(1.0),
                        outer_margin: Margin::same(0.0),
                        fill: ctx.style().visuals.panel_fill,
                        ..Default::default()
                    }
                )
                .resizable(false)
                .show(ctx, |ui| {
                    ui.set_enabled(!self.error_dialog && !self.settings_dialog && !self.save_dialog);

                    if self.show_image {
                        ui.horizontal(|ui| {
                            // rotate left
                            if ui.button("\u{27F3}").clicked() {
                                self.drawing_mode = None;
                                self.text_edit_dialog = false;
                                let result = self.screenshot.rotate_sx_90();
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.convert_image();
                                self.show_image = true;
                            }

                            // rotate right
                            if ui.button("\u{27F2}").clicked() {
                                self.drawing_mode = None;
                                self.text_edit_dialog = false;
                                let result = self.screenshot.rotate_dx_90();
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.convert_image();
                                self.show_image = true;
                            }

                            // crop
                            if ui.button("\u{2702}").clicked() {
                                self.text_edit_dialog = false;
                                let result = self.screenshot.save_intermediate_image();
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.toggle_drawing_mode(DrawingMode::Crop);
                                self.crop_screenshot_tmp = self.screenshot.clone();
                            }

                            // draw
                            if ui.button("\u{270F}")
                                .on_hover_text(format!("CTRL + {}", self.settings.pen))
                                .clicked() {
                                self.text_edit_dialog = false;
                                self.toggle_drawing_mode(DrawingMode::Paint);
                            }

                            // highlight
                            if ui.button("\u{1F526}").clicked() {
                                self.text_edit_dialog = false;
                                self.toggle_drawing_mode(DrawingMode::Highlight);
                            }

                            // erase
                            if ui.button("\u{1F4D8}")
                                .on_hover_text(format!("CTRL + {}", self.settings.rubber))
                                .clicked() {
                                self.text_edit_dialog = false;
                                self.toggle_drawing_mode(DrawingMode::Erase);
                            }

                            // shapes
                            if ui.button("\u{2B1F}").clicked() {
                                self.text_edit_dialog = false;
                                let result = self.screenshot.save_intermediate_image();
                                if self.manage_errors(result).is_none() {
                                    return;
                                }
                                self.toggle_drawing_mode(DrawingMode::Shape);
                            }

                            // text
                            if ui.button("\u{1F1F9}").clicked() {
                                self.text_edit_dialog = false;
                                self.toggle_drawing_mode(DrawingMode::Text);
                            }

                            // selected tool
                            if self.drawing_mode.is_some() {
                                ui.label(self.drawing_mode.unwrap().to_string());
                            }


                            //GIVEN A DRAWING_MODE SHOWS DIFFERENT THINGS ON THE RIGHT BOTTOM SIDE
                            if self.drawing_mode.is_some() {
                                // Color Picker, Size Picker for Brush, Highlight, Erase, Shapes, Text
                                ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                                    //SIZE FOR ALL
                                    if self.drawing_mode != Some(DrawingMode::Crop) && self.drawing_mode != Some(DrawingMode::Erase) && self.drawing_mode != None {
                                        //with color picker
                                        let picker = ui.color_edit_button_srgb(&mut self.tool_color).clone();
                                        match self.drawing_mode {
                                            Some(DrawingMode::Paint) => {
                                                ui.add(Slider::new(&mut self.tool_size, 3.0..=25.0));
                                                if picker.clicked() {
                                                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                                                    self.manage_errors(result);
                                                    self.previous_drawing_mode = Some(DrawingMode::Paint);
                                                    self.drawing_mode = Some(DrawingMode::Pause);
                                                }
                                            }
                                            Some(DrawingMode::Highlight) => {
                                                ui.add(Slider::new(&mut self.tool_size, 3.0..=25.0));
                                                if picker.clicked() {
                                                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                                                    self.manage_errors(result);
                                                    self.previous_drawing_mode = Some(DrawingMode::Highlight);
                                                    self.drawing_mode = Some(DrawingMode::Pause);
                                                }
                                            }
                                            Some(DrawingMode::Shape) => {
                                                ui.add(Slider::new(&mut self.tool_size, 3.0..=25.0));
                                                if ui.button("\u{25AD}").clicked() { self.shape = Some(Shape::Rectangle); }
                                                if ui.button("\u{2B55}").clicked() { self.shape = Some(Shape::Circle); }
                                                if ui.button("\u{2197}").clicked() { self.shape = Some(Shape::Arrow); }
                                                if picker.clicked() {
                                                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                                                    self.manage_errors(result);
                                                    self.previous_drawing_mode = Some(DrawingMode::Shape);
                                                    self.drawing_mode = Some(DrawingMode::Pause);
                                                }
                                            }
                                            Some(DrawingMode::Text) => {
                                                ui.add(Slider::new(&mut self.tool_size, 3.0..=25.0));
                                                self.drawing_mode = Some(DrawingMode::Text);
                                                if picker.clicked() {
                                                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::Pause);
                                                    self.manage_errors(result);
                                                    self.previous_drawing_mode = Some(DrawingMode::Text);
                                                    self.drawing_mode = Some(DrawingMode::Pause);
                                                }
                                            }
                                            Some(DrawingMode::Pause) => {
                                                if picker.clicked_elsewhere() || ctx.input(|is| is.key_pressed(Key::Escape))
                                                {
                                                    self.drawing_mode = self.previous_drawing_mode;
                                                    let result = self.hotkey_manager.set_active_shortcuts(ActiveShortcuts::ScreenshotDone);
                                                    self.manage_errors(result);
                                                }
                                            }
                                            _ => {}
                                        }
                                    } else {
                                        //without color picker (Crop,Erase,None)
                                        match self.drawing_mode {
                                            Some(DrawingMode::Erase) => {
                                                ui.add(Slider::new(&mut self.tool_size, 3.0..=25.0));
                                            }
                                            _ => {}
                                        }
                                    }
                                });
                            }
                        });
                    }
                });


            //GIVEN A DRAWING MODE EXECUTES THE FUNCTION
            CentralPanel::default()
                .frame(Frame::none())
                .show(ctx, |ui| {
                    ui.set_enabled(!self.error_dialog);

                    ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                        let values_window = self.calculate_rect_image(ui.available_size(), ctx.used_size());

                        // TEXT EDIT DIALOG
                        if self.text_edit_dialog {
                            //text edit window without titlebar
                            Window::new("TextEdit")
                                .current_pos(self.text_edit_dialog_position)
                                .title_bar(false)
                                .collapsible(false)
                                .resizable(false)
                                .movable(true)
                                .drag_bounds(Rect::from_min_size(Pos2::new(values_window.0, values_window.1), Vec2::new(values_window.2, values_window.3)))
                                .frame(
                                    Frame {
                                        fill: Color32::from_rgba_unmultiplied(0, 0, 0, 50),
                                        stroke: Stroke::new(1.0, Color32::WHITE),
                                        ..Default::default()
                                    })
                                .show(ctx, |ui_window| {
                                    let w = ui_window.add(
                                        TextEdit::multiline(&mut self.text)
                                            .font(egui::FontId::proportional(self.tool_size))
                                            .text_color(Color32::from_rgb(self.tool_color[0], self.tool_color[1], self.tool_color[2]))
                                            .frame(false)
                                    );
                                    self.text_edit_dialog_position = w.rect.left_top(); //for moving it from a position to another
                                    let enter_pressed = ctx.input(|is| is.key_pressed(Key::Enter));
                                    let shift_pressed = ctx.input(|is| is.modifiers.shift);
                                    let exit_pressed = ctx.input(|is| is.key_pressed(Key::Escape));
                                    if enter_pressed && shift_pressed {
                                        //add new line
                                        self.text = format!("{}\n", self.text);
                                    } else if enter_pressed {
                                        //print the line
                                        self.text_edit_dialog = false;
                                        let textbox_pos = self.calculate_texture_coordinates(w.rect.left_top(), ui.available_size(), ctx.used_size(), true).unwrap();
                                        let x = self.tool_size / values_window.4;
                                        let y = self.tool_size / values_window.5;
                                        self.screenshot.draw_text(&self.text, textbox_pos.x.max(0.0), textbox_pos.y.max(0.0), self.tool_color, Scale { x, y });
                                        self.text = "".to_string();
                                        self.convert_image();
                                    } else if exit_pressed {
                                        //exit from the line
                                        self.text_edit_dialog = false;
                                        self.text = "".to_string();
                                    }
                                });
                        }

                        // FUNCTIONS FOR ALL THE DRAWING MODES CENTRAL PANEL
                        if self.show_image {
                            let available = ui.available_size();
                            let mut my_image = MyImage::new();
                            my_image.ui_resize(ui, self.color_image.clone());
                            // drawing
                            match self.drawing_mode {
                                Some(DrawingMode::Paint) => {
                                    match self.draw_paint(ctx, available, self.tool_size, [self.tool_color[0], self.tool_color[1], self.tool_color[2], 255]) {
                                        true => {
                                            ctx.set_cursor_icon(CursorIcon::Crosshair);
                                        }
                                        false => {
                                            ctx.set_cursor_icon(CursorIcon::Default);
                                        }
                                    }
                                }
                                Some(DrawingMode::Highlight) => {
                                    match self.draw_highlight(ctx, available, self.tool_size, self.tool_color) {
                                        true => {
                                            ctx.set_cursor_icon(CursorIcon::VerticalText);
                                        }
                                        false => {
                                            ctx.set_cursor_icon(CursorIcon::Default);
                                        }
                                    }
                                }
                                Some(DrawingMode::Erase) => {
                                    match self.erase(ctx, available, self.tool_size) {
                                        true => {
                                            ctx.set_cursor_icon(CursorIcon::NotAllowed);
                                        }
                                        false => {
                                            ctx.set_cursor_icon(CursorIcon::Default);
                                        }
                                    }
                                }
                                Some(DrawingMode::Shape) => {
                                    match self.shape {
                                        Some(Shape::Rectangle) => {
                                            self.draw_rectangle(ctx, available, self.tool_size, [self.tool_color[0], self.tool_color[1], self.tool_color[2], 255]);
                                        }
                                        Some(Shape::Circle) => {
                                            self.draw_circle(ctx, available, self.tool_size, [self.tool_color[0], self.tool_color[1], self.tool_color[2], 255]);
                                        }
                                        Some(Shape::Arrow) => {
                                            self.draw_arrow(ctx, available, self.tool_size, [self.tool_color[0], self.tool_color[1], self.tool_color[2], 255]);
                                        }
                                        _ => {}
                                    }
                                }
                                Some(DrawingMode::Crop) => {
                                    let coordinates = self.draw_rectangle(ctx, available, 2.0, [255, 255, 255, 255]);
                                    if coordinates.is_some() {
                                        let coordinates = coordinates.unwrap();
                                        let height = (coordinates.0.1 - coordinates.1.1).abs() as i32;
                                        let width = (coordinates.0.0 - coordinates.1.0).abs() as i32;
                                        let min_x = cmp::min(coordinates.0.0 as u32, coordinates.1.0 as u32);
                                        let min_y = cmp::min(coordinates.0.1 as u32, coordinates.1.1 as u32);
                                        let result = self.screenshot.resize_image(min_x + 2, min_y + 2, height - 4, width - 4);
                                        if self.manage_errors(result).is_none() {
                                            self.screenshot = self.crop_screenshot_tmp.clone();
                                        }
                                        self.convert_image();
                                    }
                                }
                                Some(DrawingMode::Text) => {
                                    ctx.input(|is| {
                                        if is.pointer.any_down() && !self.text_edit_dialog && self.calculate_texture_coordinates(is.pointer.interact_pos().unwrap(), available, ctx.used_size(), false).is_some() {
                                            self.text_edit_dialog_position = is.pointer.interact_pos().unwrap();
                                            self.text_edit_dialog = true;
                                        }
                                    });
                                }
                                _ => {}
                            }
                        }
                    });
                });
        }
    }


    struct MyImage {
        texture: Option<egui::TextureHandle>,
    }

    impl MyImage {
        pub fn ui_resize(&mut self, ui: &mut egui::Ui, image: ColorImage) {
            let texture: &egui::TextureHandle = self.texture.get_or_insert_with(|| {
                // Load the texture only once.
                ui.ctx().load_texture(
                    "my-image",
                    image.clone(),
                    Default::default(),
                )
            });

            let available = ui.available_size();
            let w = image.width() as f32;
            let h = image.height() as f32;
            let w_window = available.x;
            let h_window = available.y;
            // gives the min between the height of the window and the height of the image scaled to the width of the window
            let height = h_window.min(w_window * h / w);
            let width = height * w / h;
            let fixed_dimensions = Vec2 { x: width, y: height };
            // Show the image:
            ui.image(texture, fixed_dimensions);
        }

        pub fn new() -> Self {
            Self { texture: None }
        }
    }

    pub fn get_screens() -> Vec<Screen> {
        let screens = Screen::all().unwrap();
        screens
    }

    pub fn take_screenshot(timer: Duration, screen: usize) -> Screenshot {
        let screens = Screen::all().unwrap();
        let screen = screens[screen].clone();
        //screenshot after delay
        let ss1 = Screenshot::screenshot_after_delay(timer, screen).unwrap();
        //save image to clipboard
        ss1
    }
}