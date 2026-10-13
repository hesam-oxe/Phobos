use std::fs;
use std::path::PathBuf;

pub struct FileIO;

impl FileIO {
    pub fn open_file_dialog() -> Option<(PathBuf, String)> {
        rfd::FileDialog::new()
            .add_filter("All Files", &["*"])
            .add_filter("Rust", &["rs"])
            .add_filter("Python", &["py"])
            .add_filter("C/C++", &["c", "cpp", "h", "hpp"])
            .add_filter("JavaScript", &["js", "ts", "jsx", "tsx"])
            .add_filter("HTML/CSS", &["html", "css"])
            .add_filter("JSON/YAML", &["json", "yaml", "yml"])
            .pick_file()
            .map(|path| {
                let content = fs::read_to_string(&path).unwrap_or_default();
                (path, content)
            })
    }

    pub fn save_file_dialog(content: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("All Files", &["*"])
            .save_file()
            .map(|path| {
                fs::write(&path, content).ok();
                path
            })
    }

    pub fn open_folder_dialog() -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }

    pub fn read_file(path: &str) -> Option<String> {
        fs::read_to_string(path).ok()
    }

    pub fn write_file(path: &str, content: &str) -> bool {
        fs::write(path, content).is_ok()
    }
}
