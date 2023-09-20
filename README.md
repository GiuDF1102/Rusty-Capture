# Rusty Capture
Rusty Capture is a Cross-Platform screen-grabbing and post-processing utiliy. This application allows you to capture your screen, post-process the captured image, and save it in various formats. Written in Rust, this utility is designed to work on Windows, macOS, and Linux operating systems.

# Features
 1. ### Platform Support
    Rusty Capture supports multiple desktop operating systems, ensuring compatibility across various platforms.

 2. ### User Interface (UI)
    The application comes with an intuitive and user-friendly interface that makes it easy to navigate and access all of its features.

 3. ### Selection Options
    You can easily grab a custom area on your screen by using a click-and-drag motion. Further adjustments to the selected area are possible with subsequent interactions.

 4. ### Hotkey Support
    To speed up your screen-grabbing process, Rusty Capture supports customizable hotkeys. You can set up your preferred shortcut keys for quick access.

 5. ### Output Format
    The utility supports multiple output formats, including .png, .jpg, .gif. Additionally, it allows you to copy the screen grab directly to your clipboard.

 6. ### Annotation Tools
    Rusty Capture comes with built-in annotation tools such as the possibility to draw shapes, add text and easily highlight or redact specific parts of the captured image, but also draw with a pencil tool. You are also able to rotate the captured image.

 7. ### Delay Timer
    A delay timer function is available, which allows you to set up a screen grab after a specified delay. This is useful for capturing time-sensitive content.

 8. ### Save Options
    You can customize the default save location for your screen grabs. The utility also supports automatic saving with predefined naming conventions.

 9. ### Multi-monitor Support
    Rusty Capture can handle multiple monitors independently, allowing you to grab screens from any connected displays.

# Libraries

1.  eframe = "0.22.0" for the GUI
2.  arboard = "3.2.0" for saving the image to clipboard
3. chrono = "0.4.26" time library (required for the screenshot name)
4. global-hotkey = "0.2.3" hotkeys library 
5. image = "0.24.6" DynamicImage library, it's the main structure for saving and manipulating the image inside this program
6. screenshots = "0.7.0" Screenshot library
7. thiserror = "1.0.43" Error library, used inside the modules for creating custom-made errors
8. serde = { version = "1.0", features = ["derive"] } serializing library, used for writing settings
9. serde_json = "1.0"
10. imageproc = "0.23.0" Extension of the image library, used for creating lines, polygons etc inside the DynamicImage
11. rusttype = "0.9.3" Font Library

