use serde::{Serialize, Deserialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Write};
use zip::read::ZipArchive;
use std::fs;

const LS: &'static str = "ls";
const CD: &'static str = "cd";
const EXIT: &'static str = "exit";
const CLEAR: &'static str = "clear";
const CAT: &'static str = "cat";
const PWD: &'static str = "pwd";

#[derive(Serialize)]
#[derive(Clone)]
pub struct LogEntry {
    pub user: String,
    pub command: String,
    pub details: String,
}

#[derive(Serialize)]
pub struct Session {
    pub user: String,
    pub log: Vec<LogEntry>,
}

pub struct ShellEmulator {
    pub user: String,
    pub computer: String,
    pub current_dir: String,
    pub archive: ZipArchive<File>,
    pub log_file: String,
    pub log_entries: Vec<LogEntry>,
}

fn parent(path: &str) -> &str {
    match path {
        "" | "/" => path,
        _ => {
            let mut copy = path;

            if copy.ends_with("/") {
                copy = copy.strip_suffix("/").unwrap();
            }

            let index = match copy.rfind('/') {
                Some(i) => i,
                None => 0
            };

            match index {
                0 => "",
                _ => &path[..index+1]
            }
        }
    }
}

impl ShellEmulator {
    pub fn new(config: &Config) -> Self {
        let file = File::open(&config.zip_path).expect("failed to open zip");
        let archive = ZipArchive::new(file).expect("failed to read zip");

        ShellEmulator {
            user: config.user.clone(),
            computer: config.computer.clone(),
            current_dir: String::new(),
            archive,
            log_file: config.log_file.clone(),
            log_entries: vec![],
        }
    }

    pub fn run(&mut self) {
        loop {
            print!("{}@{}:/{}$ ", self.user, self.computer, self.current_dir);
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            let command: Vec<&str> = input.split_whitespace().collect();
            if command.is_empty() {
                continue;
            }

            match command[0] {
                LS => {
                    if command.len() > 1 {
                        self.ls(command[1]);
                    } else {
                        self.ls("");
                    }
                },
                CAT => {
                    if command.len() > 1 {
                        self.cat(command[1]);
                    } else {
                        continue;
                    }
                }
                CD => {
                    if command.len() > 1 {
                        self.cd(command[1]);
                    } else {
                        self.cd("");
                    }
                }
                CLEAR => self.clear(),
                EXIT => {
                    self.log(EXIT, "Shell exited".to_string());
                    self.save_log();
                    break;
                }
                PWD => self.pwd(),
                _ => println!("mysh: {}: неизвестная команда", command[0]),
            }
        }
    }

    pub fn ls(&mut self, dir: &str) { 
        match dir {
            ".?." => {
                println!("CONGRATULATIONS!!! YOU FOUND EASTER EGG");
                for name in self.archive.file_names() {
                    print!("{name} ");
                }
                print!("\n");

                self.log("easter egg", String::from("easter egg found"));
            }
            "" => {
                let mut files: HashSet<&str> = HashSet::new();

                if self.current_dir == "" {
                    for name in self.archive.file_names() {
                        let i = match name.find('/') {
                            Some(f) => f,
                            None => name.len()
                        };

                        let filename = &name[..i];

                        if filename != "" && !files.contains(filename) {
                            print!("{filename} ");
                            files.insert(filename);
                        }
                    }
                    print!("\n");
                } else {
                    let names: Vec<&str> = self.archive
                        .file_names()
                        .map(|f| {
                        match f.strip_prefix(self.current_dir.as_str()) {
                            Some(s) => s,
                            None => ""
                        }
                        })
                        .filter(|s| !s.is_empty() && s != &"/")
                        .collect();

                    let count = names.len();

                    if count != 0 {
                        for name in names {
                            let i = match name.find("/") {
                                Some(index) => index,
                                None => name.len()
                            };

                            let file = &name[..i];

                            if !files.contains(file) {
                                print!("{file} ");
                                files.insert(file);
                            }
                        }
                        print!("\n");
                    }
                }

                let dir = self.current_dir.clone();
                self.log(LS, format!("ls of /{dir}"));
            }
            "." => self.ls(self.current_dir.clone().as_str()),
            ".." => {
                if self.current_dir != "" {
                } else {
                    self.ls("");
                }
            }
            _ => {
                let fullpath = self.current_dir.clone() + dir;

                let names: Vec<&str> = self.archive
                    .file_names()
                    .map(|f| {
                    match f.strip_prefix(&(fullpath.as_str().to_owned() + "/")) {
                        Some(s) => s,
                        None => ""
                    }
                    })
                    .filter(|s| !s.is_empty() && s != &"/")
                    .collect();

                let count = names.len();

                let mut found: HashSet<&str> = HashSet::new();
                if count != 0 {
                    for name in names {
                        let i = name.find("/");
                        let filename = match i {
                            Some(index) => &name[..index],
                            None => name
                        };

                        if !found.contains(filename) {
                            print!("{filename} ");
                            found.insert(filename);
                        }
                    }
                    print!("\n");
                }

                self.log(LS, format!("ls of /{fullpath}"));
            }
        }

        self.log("ls", format!("ls of /{}", dir));
    }

    pub fn cd(&mut self, dir: &str) {
        match dir {
            "" | "/" => {
                self.current_dir = String::new();
                self.log(CD, String::from("cd to root directory"));
            }
            ".." => {
                if self.current_dir == "" {
                    self.log(CD, String::from("cd to root directory"));
                    return;
                }

                self.current_dir = String::from(parent(&self.current_dir));
            }
            _ => {
                match dir.chars().nth(0) {
                    Some('.') | Some('/') => {
                        let msg = "mysh: cd: path starts with . or /";
                        println!("{msg}");
                        self.log(CD, String::from(msg));
                        return;
                    },
                    _ => {}
                }

                let mut fullpath: String = self.current_dir.clone() + dir;

                if !fullpath.ends_with("/") {
                    fullpath += "/";
                }

                let mut msg = String::new();

                match self.archive.by_name(&fullpath) {
                    Ok(f) => {
                        if f.is_dir() {
                            self.current_dir = String::from(fullpath.clone());
                            msg = format!("cd to /{fullpath}");
                        } else {
                            msg = format!("mysh: cd: {fullpath} is not a directory");
                            println!("{msg}");
                        }
                    }
                    Err(e) => {
                        let msg = format!("mysh: cd: {e}");
                        println!("{msg}");
                    }
                }

                self.log(CD, msg);
            }
        }
    }

    pub fn clear(&mut self) {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
        self.log(CLEAR, "clear shell".to_string());
    }

    pub fn cat(&mut self, dir: &str) {
        match dir.chars().nth(0) {
            Some('/') => {
                println!("mysh: cat: path starts with . or /");
                return
            }
            _ => {}
        };

        let fullpath = self.current_dir.clone() + dir;

        let mut file = match self.archive.by_name(&fullpath) {
            Ok(t) => t,
            Err(e) => {
                println!("mysh: cat: {e}");
                return
            }
        };

        if file.is_dir() {
            let msg = format!("mysh: {fullpath} is a directory");
            println!("{msg}");
        } else {
            let mut body = String::new();
            match file.read_to_string(&mut body) {
                Ok(_) => {}
                Err(e) => {
                    let msg = format!("mysh: cat: {e}");
                    println!("{msg}");
                }
            };

            println!("{body}");
        }
    }

    pub fn log(&mut self, command: &str, details: String) {
        let entry = LogEntry {
            user: self.user.clone(),
            command: command.to_string(),
            details,
        };
        self.log_entries.push(entry);
    }

    pub fn save_log(&self) {
        let session = Session {
            user: self.user.clone(),
            log: self.log_entries.clone(),
        };

        let log_file = File::create(&self.log_file).unwrap();
        serde_json::to_writer_pretty(log_file, &session).expect("Failed to write log file");
    }

    pub fn pwd(&self) {
        let wd = self.current_dir.as_str();

        println!("/{wd}");
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub user: String,
    pub computer: String,
    pub zip_path: String,
    pub log_file: String,
}

pub fn load_config(path: &str) -> Config {
    let contents = fs::read_to_string(path).expect("failed to read config file");
    toml::from_str(&contents).expect("failed to parse config")
}
