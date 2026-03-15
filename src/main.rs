mod app;
mod cleaner;
mod cli;
mod config;
mod i18n;
mod output;
mod prism;
mod scanner;

fn main() {
    if let Err(err) = app::run() {
        eprintln!("error: {err:?}");
        std::process::exit(1);
    }
}
