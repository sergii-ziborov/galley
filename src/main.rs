mod app;
mod draw;
mod engine;
mod inline;
mod model;
mod page;
mod raster;
mod sixel;
mod term;
mod ui;

use std::env;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::ExitCode;

use app::App;
use term::{Polled, Term};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("frame") => {
            let Some(dir) = args.next() else {
                eprintln!("galley: frame needs a directory");
                return ExitCode::from(1);
            };
            return match raster::write_frames(std::path::Path::new(&dir)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(err) => {
                    eprintln!("galley: {err}");
                    return ExitCode::from(1);
                }
            };
        }
        Some(other) if other.starts_with('-') => {
            eprintln!("galley: {other} is not a flag");
            return ExitCode::from(2);
        }
        other => {
            let url = other.map(str::to_string);
            if !Term::tty() {
                eprintln!("galley: run this inside a terminal");
                return ExitCode::from(1);
            }
            return run(url);
        }
    }
}

fn run(url: Option<String>) -> ExitCode {
    let dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut app = App::new(dir);
    if let Some(url) = url {
        app.open_target(&url);
    }
    let mut term = match Term::enter() {
        Ok(term) => term,
        Err(err) => {
            eprintln!("galley: {err}");
            return ExitCode::from(1);
        }
    };
    let mut drawn = String::new();
    let mut drawn_cursor = None;
    loop {
        if app.quit() {
            break;
        }
        let (cols, rows) = Term::size();
        app.set_size(cols, rows);
        if cols == 0 || rows == 0 {
            match term.poll() {
                Ok(Polled::Eof) if !Term::tty() => break,
                Ok(_) => continue,
                Err(_) if Term::tty() => continue,
                Err(_) => break,
            }
        }
        let screen = ui::Screen::paint(&app, cols, rows);
        if screen.frame != drawn || screen.cursor != drawn_cursor {
            match term.present(&screen.frame, screen.cursor) {
                Ok(()) => {
                    drawn = screen.frame.clone();
                    drawn_cursor = screen.cursor;
                }
                Err(err) => {
                    let kind = err.kind();
                    let soft = kind == ErrorKind::Interrupted || kind == ErrorKind::WouldBlock;
                    if !soft && !Term::tty() {
                        break;
                    }
                }
            }
        }
        match term.poll() {
            Ok(Polled::Keys(keys)) => {
                for key in keys {
                    app.on_key(key);
                    if app.quit() {
                        break;
                    }
                }
            }
            Ok(Polled::Eof) if !Term::tty() => break,
            Ok(_) => {}
            Err(_) if Term::tty() => continue,
            Err(_) => break,
        }
    }
    ExitCode::SUCCESS
}
