#![allow(dead_code)]
pub mod screenshot_module {
    use std::borrow::Cow;
    use std::error::Error;
    use std::path::PathBuf;
    use std::{cmp, thread};
    use std::time::Duration;
    use arboard::{Clipboard, ImageData};
    use chrono::Local;
    use image::{DynamicImage, GenericImage, GenericImageView, ImageFormat, Rgba, RgbaImage};
    use imageproc::drawing::{draw_line_segment_mut, draw_hollow_rect_mut, draw_text_mut, draw_hollow_circle_mut, draw_polygon_mut};
    use imageproc::point::Point;
    use imageproc::rect::Rect;
    use screenshots::Screen;
    use thiserror::Error;
    use rusttype::{Scale, Font};

    #[derive(Error, Debug)]
    enum ScreenShotError {
        #[error("sizes are not compatible")]
        ResizeSize,
        #[error("Path is not a dir")]
        PathError,
        #[error("extension error")]
        ExtensionError,
    }

    #[derive(Clone)]
    pub struct Screenshot {
        screenshot: DynamicImage,
        original_image: DynamicImage,
        intermediate_image: DynamicImage,
    }

    impl Screenshot {
        pub fn new_empty() -> Screenshot {
            Screenshot {
                screenshot: DynamicImage::new_rgba8(0, 0),
                original_image: DynamicImage::new_rgba8(0, 0),
                intermediate_image: DynamicImage::new_rgba8(0, 0),
            }
        }

        pub fn new(screen: Screen) -> Result<Screenshot, Box<dyn Error>> {
            let image_captured = screen.capture()?;
            let width = image_captured.width();
            let height = image_captured.height();
            let image_rgba = image_captured.rgba().to_owned();
            let rgba_image = RgbaImage::from_raw(width, height, image_rgba).unwrap();
            let image_obj = DynamicImage::from(rgba_image);
            let original_obj = image_obj.clone();
            let intermediate_obj = image_obj.clone();
            Ok(
                Screenshot {
                    screenshot: image_obj,
                    original_image: original_obj,
                    intermediate_image: intermediate_obj,
                }
            )
        }

        pub fn save_image(&self, path: &PathBuf, format: ImageFormat) -> Result<(), Box<dyn Error>> {
            if path.is_dir() == false {
                return Err(Box::new(ScreenShotError::PathError));
            }
            let mut file_name = "screenshot-".to_string();
            file_name.push_str(Local::now().format("%d-%m-%Y-%H-%M-%S_%3f").to_string().as_str());
            let mut path_with_file_name = path.join(PathBuf::from(file_name));
            match format {
                ImageFormat::Png => {
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("png"));
                }
                ImageFormat::Gif => {
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("gif"));
                }
                ImageFormat::Jpeg => {
                    path_with_file_name = path_with_file_name.with_extension(PathBuf::from("jpg"));
                }
                _ => {
                    return Err(Box::new(ScreenShotError::ExtensionError));
                }
            }
            self.screenshot.save_with_format(path_with_file_name, format)?;
            return Ok(());
        }

        pub fn save_to_clipboard(&self) -> Result<(), Box<dyn Error>> {
            let mut clipboard = Clipboard::new()?;
            clipboard.set_image(ImageData {
                width: self.screenshot.width() as usize,
                height: self.screenshot.height() as usize,
                bytes: Cow::from(self.screenshot.as_bytes()),
            })?;
            Ok(())
        }

        pub fn get_image(&self) -> Result<DynamicImage, Box<dyn Error>> {
            Ok(self.screenshot.clone())
        }

        pub fn resize_image(&mut self, x: u32, y: u32, height: i32, width: i32) -> Result<(), Box<dyn Error>> {
            if height < 0 || width < 0 || self.screenshot.width() < (x + width as u32) || self.screenshot.height() < (y + height as u32) {
                return Err(Box::new(ScreenShotError::ResizeSize));
            }
            self.screenshot = self.screenshot.crop(x, y, width as u32, height as u32);
            self.intermediate_image = self.screenshot.clone();
            self.original_image = self.original_image.crop(x, y, width as u32, height as u32);
            Ok(())
        }
        pub fn get_width(&self) -> Result<u32, Box<dyn Error>> {
            return Ok(self.screenshot.width());
        }

        pub fn get_height(&self) -> Result<u32, Box<dyn Error>> {
            return Ok(self.screenshot.height());
        }

        pub fn rotate_sx_90(&mut self) -> Result<(), Box<dyn Error>> {
            self.screenshot = self.screenshot.rotate90();
            self.original_image = self.original_image.rotate90();
            Ok(())
        }
        pub fn rotate_dx_90(&mut self) -> Result<(), Box<dyn Error>> {
            self.screenshot = self.screenshot.rotate270();
            self.original_image = self.original_image.rotate270();
            Ok(())
        }

        pub fn screenshot_after_delay(duration: Duration, screen: Screen) -> Result<Screenshot, Box<dyn Error>> {
            thread::sleep(duration);
            Screenshot::new(screen)
        }

        pub fn save_intermediate_image(&mut self) -> Result<(), Box<dyn Error>> {
            self.intermediate_image = self.screenshot.clone();
            Ok(())
        }

        pub fn rollback_changes(&mut self){
            self.screenshot=self.intermediate_image.clone();
        }

        pub fn blend_colors(background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
            let alpha = foreground[3] as f32 / 255.0;
            let inv_alpha = 1.0 - alpha;
            let r = (foreground[0] as f32 * alpha + background[0] as f32 * inv_alpha) as u8;
            let g = (foreground[1] as f32 * alpha + background[1] as f32 * inv_alpha) as u8;
            let b = (foreground[2] as f32 * alpha + background[2] as f32 * inv_alpha) as u8;
            let a = background[3];
            Rgba([r, g, b, a])
        }

        /// Draws a single point as a full circle centered in (x0,y0) with radius r of color c
        /// as (x,y) s.t. (x-x0)^2 + (y-y0)^2 <= r^2
        pub fn draw_point(&mut self, x: f32, y: f32, r: f32, color: [u8; 4]) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let x = x as i32;
            let y = y as i32;
            let r = r as i32;
            if x > 0 && x < width && y > 0 && y < height {
                for i in (x - r)..=(x + r) {
                    for j in (y - r)..=(y + r) {
                        // Full circles
                        if ((x - i) * (x - i) + (y - j) * (y - j)) <= r {
                            if i >= 0 && i < width && j >= 0 && j < height {
                                let color_pixel = Rgba(color);
                                self.screenshot.put_pixel(i as u32, j as u32, color_pixel);
                            }
                        }
                    }
                }
            }
        }

        /// Draws a line between a given starting point and an ending point with specific width and color
        /// as a stripe of unit width lines parallel and aligned to the original line
        pub fn draw_line(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), color: [u8; 4], size: f32) {
            let color_pixel = Rgba::from(color);
            // calculate the direction vector of the line
            let dx = ending_point.0 - starting_point.0;
            let dy = ending_point.1 - starting_point.1;
            let length = (dx * dx + dy * dy).sqrt();
            // calculate the normalized perpendicular vector to the line
            let nx = dy / length;
            let ny = -dx / length;
            // calculate the step size for the brush strokes
            let factor = 10.0;
            let step_size = 1.0 / factor;
            let thickness = (factor / 2.0) * (size + 0.5);
            for i in -(thickness/2.0) as i32..(thickness/2.0) as i32 {
                // calculate the offset along the perpendicular vector
                let offset = (i as f32 - size) * step_size;
                // calculate the starting and ending points for each brush stroke
                let start_x = starting_point.0 + nx * offset;
                let start_y = starting_point.1 + ny * offset;
                let end_x = ending_point.0 + nx * offset;
                let end_y = ending_point.1 + ny * offset;
                // draw the brush stroke
                draw_line_segment_mut(&mut self.screenshot, (start_x, start_y), (end_x, end_y), color_pixel);
            }
        }


        /// Highlights the region between the starting point and the ending point of the given size
        /// with four times less transparency than the maximum (alpha channel = 64/255)
        /// as a stripe of parallel lines with the same abscissa drawn on an overlay later merged
        /// with the original image calling the blend_colors function
        pub fn highlight_line(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), size: f32, color: [u8; 3]) {
            let transparency = 64;
            let highlight_color = Rgba([color[0], color[1], color[2], transparency]);
            // create a temporary overlay image with the same size as the screenshot
            let mut overlay_image = RgbaImage::new(self.screenshot.width(), self.screenshot.height());
            // draw the highlighted stripes on the overlay image
            draw_line_segment_mut(&mut overlay_image, starting_point, ending_point, highlight_color);
            for i in 1..=(size as i32) {
                let offset = i as f32;
                draw_line_segment_mut(&mut overlay_image,
                                      (starting_point.0, starting_point.1 - offset),
                                      (ending_point.0, ending_point.1 - offset),
                                      highlight_color,
                );
            }
            // combine the overlay image with the screenshot using alpha blending
            let (width, height) = self.screenshot.dimensions();
            for y in 0..height {
                for x in 0..width {
                    let screenshot_pixel = self.screenshot.get_pixel(x, y);
                    let overlay_pixel = overlay_image.get_pixel(x, y);
                    // calculate the blended pixel color manually
                    let blended_pixel = Self::blend_colors(screenshot_pixel, *overlay_pixel);
                    self.screenshot.put_pixel(x, y, blended_pixel);
                }
            }
        }

        /// Erases whatever modification or annotation made to the image within a circular region
        /// centered in (x0,y0) with a radius equal to r in accordance with (x-x0)^2 + (y-y0)^2 <= r^2
        /// restoring the corresponding portion of the original image by retrieving the pixels within
        /// the area and pasting them on the current image
        pub fn erase_point(&mut self, x: f32, y: f32, r: f32) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let (x, y) = (x as i32, y as i32);
            let r = r as i32;

            if x > 0 && x < width && y > 0 && y < height {
                for dx in -r..r {
                    for dy in -r..r {
                        let src_x = x + dx;
                        let src_y = y + dy;
                        if src_x >= 0 && src_x < width && src_y >= 0 && src_y < height {
                            let src_pixel = self.original_image.get_pixel(src_x as u32, src_y as u32);
                            if (dx * dx + dy * dy) <= r * r {
                                self.screenshot.put_pixel(src_x as u32, src_y as u32, src_pixel);
                            }
                        }
                    }
                }
            }
        }

        /// Draws a rectangle which diagonal is the line drawn from the starting point to the ending point
        /// with a border of specified size and given color obtained by drawing a number of concentric rectangles equal to size
        pub fn rectangle(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), size: f32, color: [u8; 4]) {
            let width = self.screenshot.width() as i32;
            let height = self.screenshot.height() as i32;
            let start = (starting_point.0 as i32, starting_point.1 as i32);
            let end = (ending_point.0 as i32, ending_point.1 as i32);
            let color_rgba = Rgba(color);
            let half_size = (size / 2.0) as i32;
            self.screenshot = self.intermediate_image.clone();
            // loop for concentric rectangles to emulate border thickness
            for dx in -half_size..=half_size {
                // calculate vertical increment dy based on horizontal increment dx
                // based on which plane quadrant the rectangle is drawn: the first (+,+) and the third (-,-) one need a concordant increment dy, but the second (+,-) and the fourth (-,+) one need discordant increment -dy (implemented with XOR)
                let mut dy = dx;
                if (start.0 > end.0) ^ (start.1 > end.1) { dy = -dx }
                // calculate the starting point, the ending point and the dimensions of the inner smaller rectangle
                let start = (start.0 - dx, start.1 - dy);
                let end = (end.0 + dx, end.1 + dy);
                let b = end.0 - start.0;
                let h = end.1 - start.1;

                if start.0 > 0 && start.0 < width && start.1 > 0 && start.1 < height && b.abs() > 0 && h.abs() > 0 {
                    let x0 = cmp::min(start.0, end.0);
                    let y0 = cmp::min(start.1, end.1);
                    let _x1 = cmp::max(start.0, end.0);
                    let _y1 = cmp::max(start.1, end.1);

                    let rect = Rect::at(x0, y0).of_size(b.abs() as u32, h.abs() as u32);
                    draw_hollow_rect_mut(&mut self.screenshot, rect, color_rgba);
                }
            }
        }

        /// Draws a rectangle from the center to the ending point with a border of specified size and
        /// given color obtained by drawing a number of concentric circles equal to size
        pub fn circle(&mut self, center: (f32, f32), ending_point: (f32, f32), size: f32, color: [u8; 4]) {
            let times = 2;
            let (width, height) = (self.screenshot.width() as i32 - times, self.screenshot.height() as i32 - times);
            let (mut x0, mut y0) = (center.0 as i32, center.1 as i32);
            let mut radius = f32::sqrt((ending_point.0 - center.0).powf(2.0) + (ending_point.1 - center.1).powf(2.0)) as i32 - times;
            let color_rgba = Rgba(color);
            let half_size = (size / 2.0) as i32;
            // save the image before any modification
            self.screenshot = self.intermediate_image.clone();
            // loop for concentric circles to emulate border thickness
            for i in 0..times {
                for dr in -half_size..=half_size {
                    let r = radius + dr;
                    if (x0, y0) > (0, 0) && (x0, y0) < (width, height) {
                        draw_hollow_circle_mut(&mut self.screenshot, (x0, y0), r, color_rgba);
                    }
                }
                if i == 0 {
                    x0 = x0 + 1;
                }
                else if i == 1 {
                    y0 = y0 + 1;
                }
                radius = radius + 1;
            }
        }

        /// Draws an arrow pointing from the starting point to the ending point with a specific size
        /// that modifies both the size of the head and the thickness of the body using the given color
        pub fn arrow(&mut self, starting_point: (f32, f32), ending_point: (f32, f32), size: f32, color: [u8; 4]) {
            // save the image before any modification
            self.screenshot = self.intermediate_image.clone();
            // arrow body
            self.draw_line(starting_point, ending_point, color, size);
            // arrow head
            let color_pixel = Rgba::from(color);
            // calculate the direction vector of the line
            let (dx, dy) = (ending_point.0 - starting_point.0, ending_point.1 - starting_point.1);
            let length = (dx * dx + dy * dy).sqrt();
            // calculate the normalized perpendicular vector to the line
            let (nx, ny) = (dy / length, -dx / length);
            // calculate the points for the arrow head triangle
            let arrow_length = size * 3.0;
            let arrow_width = size * 2.0;
            let arrow_head = (ending_point.0, ending_point.1);
            let arrow_tip = Point::new((ending_point.0 + arrow_length * dx / length) as i32, (ending_point.1 + arrow_length * dy / length) as i32);
            let arrow_left = Point::new((arrow_head.0 - arrow_width * nx) as i32, (arrow_head.1 - arrow_width * ny) as i32);
            let arrow_right = Point::new((arrow_head.0 + arrow_width * nx) as i32, (arrow_head.1 + arrow_width * ny) as i32);
            // draw the triangle-shaped head connecting the 3 non-collinear points calling the polygon function which fills up the bounded region
            if arrow_tip != arrow_left && arrow_tip != arrow_right {
                let points = vec![arrow_tip, arrow_left, arrow_right];
                draw_polygon_mut(&mut self.screenshot, points.as_slice(), color_pixel);
            }
        }

        pub fn draw_text(&mut self, text: &String, x: f32, y: f32, color: [u8; 3], scale: Scale) {
            // Load a font.
            let mut dy = 0;
            let lines = text.split("\n");
            for line in lines {
                let font = Vec::from(include_bytes!("../resources/fonts/ARIALN.TTF") as &[u8]);
                let font = Font::try_from_vec(font).unwrap();
                let color_rgba: [u8; 4] = [color[0], color[1], color[2], 255];
                draw_text_mut(&mut self.screenshot,
                              Rgba::from(color_rgba),
                              x as i32,
                              y as i32 + dy,
                              scale,
                              &font,
                              line,
                );
                dy += scale.y as i32;
            }
        }
    }
}
