#![allow(dead_code)]
pub mod settings_module {
    use std::error::Error;
    use serde::{Serialize, Deserialize};
    use serde_json;
    use global_hotkey::hotkey::Code;
    use std::str::FromStr;
    use thiserror::Error;

    #[derive(Error, Debug)]
    enum SettingsError {
        #[error("Path is not a dir")]
        PathError,
        #[error("Too Short!")]
        LengthError,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Settings {
        pub quick: String,
        pub new_screenshot: String,
        pub save: String,
        pub pen: String,
        pub rubber: String,
        pub path: String,
    }

    impl Default for Settings {
        fn default() -> Settings {
            Settings {
                quick: String::from("Q"),
                new_screenshot: String::from("N"),
                save: String::from("S"),
                pen: String::from("P"),
                rubber: String::from("R"),
                path: String::from("./"),
            }
        }
    }

    impl Settings {
        pub fn get_quick_hotkey(&self) -> Result<Code, Box<dyn Error>> {
            let code_str = format!("Key{}", self.quick.to_uppercase());
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_new_screenshot_hotkey(&self) -> Result<Code, Box<dyn Error>> {
            let code_str = format!("Key{}", self.new_screenshot.to_uppercase());
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_save_hotkey(&self) -> Result<Code, Box<dyn Error>> {
            let code_str = format!("Key{}", self.save.to_uppercase());
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_pen_hotkey(&self) -> Result<Code, Box<dyn Error>> {
            let code_str = format!("Key{}", self.pen.to_uppercase());
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_rubber_hotkey(&self) -> Result<Code, Box<dyn Error>> {
            let code_str = format!("Key{}", self.rubber.to_uppercase());
            Ok(Code::from_str(&code_str)?)
        }

        pub fn get_path(&self) -> String {
            self.path.clone()
        }
    }

    pub fn read_settings_from_file(filename: String) -> Result<Settings, Box<dyn Error>> {
        let file = std::fs::File::open(filename);
        if file.is_ok() {
            let file = file.unwrap();
            let reader = std::io::BufReader::new(file);
            let u = serde_json::from_reader(reader)?;
            Ok(u)
        } else {
            let sett = Settings::default();
            write_settings_to_file("settings.json".to_string(), &sett)?;
            Ok(sett)
        }
    }

    pub fn write_settings_to_file(filename: String, settings: &Settings) -> Result<(), Box<dyn Error>> {
        let file = std::fs::File::create(filename)?;
        let writer = std::io::BufWriter::new(file);

        //check if path is valid
        if !std::path::Path::new(&settings.path).exists() {
            let sett = Settings::default();
            serde_json::to_writer(writer, &sett)?;
            return Err(Box::new(SettingsError::PathError));
        }


        //check if hotkey is at least 1 character long
        if settings.quick.len() < 1 ||
            settings.new_screenshot.len() < 1 ||
            settings.save.len() < 1 ||
            settings.pen.len() < 1 ||
            settings.rubber.len() < 1
        {
            let sett = Settings::default();
            serde_json::to_writer(writer, &sett)?;
            Err(Box::new(SettingsError::LengthError))
        } else {
            serde_json::to_writer(writer, settings)?;
            Ok(())
        }
    }
}