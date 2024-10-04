use shell::ShellEmulator;
use shell::load_config;

fn main() {
    let config = load_config("src/config/config.toml");
    let mut emulator = ShellEmulator::new(&config);
    emulator.run();
}
