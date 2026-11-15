use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

use crate::app::{open_sample, App};
use crate::term::Key;
use crate::ui::Screen;

const COLS: usize = 96;
const ROWS: usize = 28;

pub fn write_frames(out_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(out_dir)?;
    let work = std::env::temp_dir().join("galley-frame");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work)?;
    let mut home = App::new(work.clone());
    home.set_size(COLS, ROWS);
    save(out_dir, "home", &home)?;
    open_sample(&work)?;
    home.open_target("page.html");
    home.on_key(Key::Esc);
    save(out_dir, "page", &home)?;
    home.on_key(Key::Char('?'));
    save(out_dir, "keys", &home)?;
    let _ = fs::remove_dir_all(&work);
    Ok(())
}

fn save(out_dir: &Path, name: &str, app: &App) -> io::Result<()> {
    let screen = Screen::paint(app, COLS, ROWS);
    let ans = out_dir.join(format!("{name}.ans"));
    let png = out_dir.join(format!("{name}.png"));
    fs::write(&ans, &screen.frame)?;
    paint_png(&ans, &png, COLS, ROWS)
}

fn paint_png(ans: &Path, png: &Path, cols: usize, rows: usize) -> io::Result<()> {
    let script = include_str!("raster.ps1");
    let file = std::env::temp_dir().join("galley-raster.ps1");
    fs::write(&file, script)?;
    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-File",
            &file.to_string_lossy(),
            &ans.to_string_lossy(),
            &png.to_string_lossy(),
            &cols.to_string(),
            &rows.to_string(),
        ])
        .stdin(std::process::Stdio::null())
        .status()?;
    let _ = fs::remove_file(&file);
    if status.success() {
        Ok(())
    } else {
        let _ = writeln!(io::stderr(), "galley: the frame painter failed");
        Err(io::Error::other("png"))
    }
}
