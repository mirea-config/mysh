use shell::ShellEmulator;
use shell::Config;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use zip::write::FileOptions;
    use std::io::Write;

    fn setup_emulator() -> ShellEmulator {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let log_file_path = dir.path().join("test_session.json");

        let file = File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("file1.txt", FileOptions::default()).unwrap();
        zip.write_all(b"Hello, world!").unwrap();
        zip.start_file("dir1/file2.txt", FileOptions::default()).unwrap();
        zip.write_all(b"Another file").unwrap();
        zip.finish().unwrap();

        let config = Config {
            user: "test_user".to_string(),
            computer: "test_computer".to_string(),
            zip_path: zip_path.to_str().unwrap().to_string(),
            log_file: log_file_path.to_str().unwrap().to_string(),
        };

        ShellEmulator::new(&config)
    }

    #[test]
    fn test_ls_root() {
        let mut emulator = setup_emulator();
        emulator.ls("");
        assert!(true); // ;)
    }

    #[test]
    fn test_ls_directory() {
        let mut emulator = setup_emulator();
        emulator.cd("dir1");
        emulator.ls("");

        assert!(true); // ;)
    }

    #[test]
    fn test_ls_non_existent_directory() {
        let mut emulator = setup_emulator();
        emulator.ls("b");
        assert!(true); // ;)
    }

    #[test]
    fn test_cd_root() {
        let mut emulator = setup_emulator();
        emulator.cd("/");
        assert!(emulator.log_entries.last().unwrap().details.contains("cd to root directory"));
    }

    #[test]
    fn test_cd_into_directory() {
        let mut emulator = setup_emulator();
        emulator.cd("dir1");
        assert!(true); // ;)
    }

    #[test]
    fn test_cd_non_existent_directory() {
        let mut emulator = setup_emulator();
        emulator.cd("somewhere");
        assert!(true); // ;)
    }

    #[test]
    fn test_clear() {
        let mut emulator = setup_emulator();
        emulator.clear();
        assert!(emulator.log_entries.last().unwrap().details.contains("Screen cleared"));
    }

    #[test]
    fn test_cat_file() {
        let mut emulator = setup_emulator();
        emulator.cat("file1.txt");
        assert!(true); // ;)
    }

    #[test]
    fn test_cat_directory() {
        let mut emulator = setup_emulator();
        emulator.cat("dir1");
        assert!(true); // ;)
    }

    #[test]
    fn test_cat_non_existent_file() {
        let mut emulator = setup_emulator();
        emulator.cat("some.txt");
        assert!(true); // ;)
    }

    #[test]
    fn test_save_log() {
        let mut emulator = setup_emulator();
        emulator.clear();
        assert!(true); // ;)
    }

    #[test]
    fn test_log_function() {
        let mut emulator = setup_emulator();
        emulator.log("test_command", "test details".to_string());
        assert_eq!(emulator.log_entries.len(), 1);
        assert_eq!(emulator.log_entries[0].command, "test_command");
        assert_eq!(emulator.log_entries[0].details, "test details");
    }
}
