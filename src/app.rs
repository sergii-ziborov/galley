use std::path::{Path, PathBuf};

use crate::engine::{Engine, WebHit, KIND_BUTTON, KIND_FIELD, KIND_LINK};
use crate::page;
use crate::term::Key;
use crate::ui::{sixel_at, Spot};

pub struct App {
    dir: PathBuf,
    cols: usize,
    rows: usize,
    engine: Engine,
    lines: Vec<String>,
    hits: Vec<WebHit>,
    sixel: String,
    sixel_col: usize,
    sixel_row: usize,
    url: String,
    address: String,
    address_at: usize,
    address_on: bool,
    js: bool,
    live: bool,
    remote: bool,
    back: Vec<String>,
    forward: Vec<String>,
    scroll: usize,
    note: String,
    bytes: u64,
    help: bool,
    field: bool,
    hot: Option<Spot>,
    quit_armed: bool,
    quit: bool,
}

impl App {
    pub fn new(dir: PathBuf) -> Self {
        let mut app = Self {
            dir,
            cols: 96,
            rows: 28,
            engine: Engine::default(),
            lines: Vec::new(),
            hits: Vec::new(),
            sixel: String::new(),
            sixel_col: 0,
            sixel_row: 0,
            url: String::new(),
            address: String::new(),
            address_at: 0,
            address_on: false,
            js: true,
            live: true,
            remote: false,
            back: Vec::new(),
            forward: Vec::new(),
            scroll: 0,
            note: "ready".into(),
            bytes: 0,
            help: false,
            field: false,
            hot: None,
            quit_armed: false,
            quit: false,
        };
        app.show_home();
        app
    }

    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn help(&self) -> bool {
        self.help
    }

    pub fn js_on(&self) -> bool {
        self.js
    }

    pub fn live_on(&self) -> bool {
        self.live
    }

    pub fn address_on(&self) -> bool {
        self.address_on
    }

    pub fn address_cursor(&self) -> usize {
        self.address_at.min(self.address.chars().count())
    }

    pub fn hot(&self) -> Option<Spot> {
        self.hot
    }

    pub fn page_lines(&self) -> &[String] {
        &self.lines
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn set_size(&mut self, cols: usize, rows: usize) {
        self.cols = cols.max(1);
        self.rows = rows.max(1);
    }

    pub fn address_line(&self, width: usize) -> String {
        let text = if self.address.is_empty() && !self.address_on {
            if self.url.is_empty() {
                "type an address".to_string()
            } else {
                self.url.clone()
            }
        } else {
            self.address.clone()
        };
        text.chars().take(width).collect()
    }

    pub fn status_line(&self) -> String {
        let size = if self.bytes == 0 {
            String::new()
        } else if self.bytes >= 1024 {
            format!("  {}kb", self.bytes / 1024)
        } else {
            format!("  {}b", self.bytes)
        };
        format!("{}{size}", self.note)
    }

    pub fn sixel_place(&self, page_top: usize) -> Option<String> {
        if self.sixel.is_empty() || self.scroll > 0 {
            return None;
        }
        Some(sixel_at(page_top, self.sixel_col, self.sixel_row, &self.sixel))
    }

    pub fn open_target(&mut self, target: &str) {
        self.go(target, true);
    }

    pub fn on_key(&mut self, key: Key) {
        match key {
            Key::CtrlC => {
                if self.quit_armed {
                    self.quit = true;
                } else {
                    self.quit_armed = true;
                    self.note = "ctrl+c again quits".into();
                }
                return;
            }
            Key::F(10) => {
                self.quit = true;
                return;
            }
            _ => self.quit_armed = false,
        }
        if self.help {
            self.on_help_key(key);
            return;
        }
        if self.address_on {
            self.on_address_key(key);
            return;
        }
        match key {
            Key::Char('?') => self.help = true,
            Key::Char('\u{000c}') => self.focus_address(),
            Key::Char(ch) if self.field && self.remote => {
                let _ = self.engine.insert(&ch.to_string());
                self.repaint(false);
            }
            Key::Click { x, y } | Key::Release { x, y } => {
                if x > 0 && y > 0 {
                    self.press(x - 1, y - 1);
                }
            }
            Key::Move { x, y } => {
                if x > 0 && y > 0 {
                    let screen = crate::ui::Screen::paint(self, self.cols, self.rows);
                    self.hot = screen.spot_at(x - 1, y - 1);
                }
            }
            Key::Wheel(delta) => self.scroll_by(delta),
            Key::F(5) => self.reload(),
            Key::Backspace => self.go_back(),
            Key::Esc => {
                self.address_on = false;
                self.help = false;
            }
            Key::Char(ch) => {
                self.focus_address();
                self.address.clear();
                self.address_at = 0;
                self.insert_char(ch);
            }
            _ => {
                if self.remote {
                    self.page_key(key);
                }
            }
        }
    }

    fn on_help_key(&mut self, key: Key) {
        match key {
            Key::Esc | Key::Char('?') | Key::F(1) => self.help = false,
            Key::Click { x, y } | Key::Release { x, y } if x > 0 && y > 0 => {
                self.press(x - 1, y - 1);
            }
            _ => {}
        }
    }

    fn on_address_key(&mut self, key: Key) {
        match key {
            Key::Esc => self.address_on = false,
            Key::Enter => {
                let target = self.address.clone();
                self.address_on = false;
                self.go(&target, true);
            }
            Key::Backspace => self.backspace_address(),
            Key::Left => self.address_at = self.address_at.saturating_sub(1),
            Key::Right => {
                if self.address_at < self.address.chars().count() {
                    self.address_at += 1;
                }
            }
            Key::Home => self.address_at = 0,
            Key::End => self.address_at = self.address.chars().count(),
            Key::Char('\u{000c}') => {}
            Key::Char(ch) => self.insert_char(ch),
            Key::Click { x, y } | Key::Release { x, y } if x > 0 && y > 0 => {
                self.press(x - 1, y - 1);
            }
            _ => {}
        }
    }

    fn insert_char(&mut self, ch: char) {
        if ch.is_control() {
            return;
        }
        let at = self.address_at.min(self.address.chars().count());
        let byte = self
            .address
            .char_indices()
            .nth(at)
            .map(|(index, _)| index)
            .unwrap_or(self.address.len());
        self.address.insert(byte, ch);
        self.address_at = at + 1;
    }

    fn backspace_address(&mut self) {
        if self.address_at == 0 {
            return;
        }
        let at = self.address_at - 1;
        if let Some((index, ch)) = self.address.char_indices().nth(at) {
            self.address.drain(index..index + ch.len_utf8());
            self.address_at = at;
        }
    }

    fn press(&mut self, x: usize, y: usize) {
        let screen = crate::ui::Screen::paint(self, self.cols, self.rows);
        let Some(spot) = screen.spot_at(x, y) else {
            return;
        };
        match spot {
            Spot::Back => self.go_back(),
            Spot::Forward => self.go_forward(),
            Spot::Reload => self.reload(),
            Spot::Home => self.go_home(),
            Spot::Js => {
                self.js = !self.js;
                self.engine.set_policy(self.js, self.live);
                self.note = if self.js { "js on".into() } else { "js off".into() };
                if self.remote {
                    self.reload();
                }
            }
            Spot::Live => {
                self.live = !self.live;
                self.engine.set_policy(self.js, self.live);
                self.note = if self.live {
                    "live on".into()
                } else {
                    "live off".into()
                };
                if self.remote {
                    self.reload();
                }
            }
            Spot::Address => self.focus_address(),
            Spot::Help => self.help = !self.help,
            Spot::HelpClose => self.help = false,
            Spot::Page => {
                self.help = false;
                self.address_on = false;
                self.field = false;
                if self.remote {
                    self.click_page(x, y);
                } else {
                    self.click_local(x, y);
                }
            }
        }
    }

    fn click_local(&mut self, x: usize, y: usize) {
        let page_top = 4usize;
        if y < page_top {
            return;
        }
        let row = y - page_top + self.scroll;
        let Some(line) = self.lines.get(row).cloned() else {
            return;
        };
        let Some(url) = url_at_column(&line, x) else {
            return;
        };
        self.go(&url, true);
    }

    fn click_page(&mut self, x: usize, y: usize) {
        let page_top = 4usize;
        if y < page_top {
            return;
        }
        let row = y - page_top + self.scroll;
        if let Some(hit) = self.hits.iter().find(|hit| {
            hit.y > 0
                && row >= hit.y - 1
                && row < hit.y - 1 + hit.h.max(1)
                && x >= hit.x
                && x < hit.x + hit.w
        }) {
            let hit = hit.clone();
            if hit.kind == KIND_LINK {
                let href = hit.href.trim();
                if href.starts_with("http://") || href.starts_with("https://") {
                    self.go(href, true);
                    return;
                }
            }
            if hit.kind == KIND_FIELD {
                let _ = self.engine.focus_at(hit.px, hit.py);
                self.address_on = false;
                self.field = true;
                self.note = "field".into();
                return;
            }
            let label = if hit.label.is_empty() && hit.kind == KIND_BUTTON {
                "button"
            } else {
                hit.label.as_str()
            };
            self.click_at(hit.px, hit.py, label);
            return;
        }
        let Some((px, py)) = self.css_at(x, y) else {
            return;
        };
        self.click_at(px, py, "");
    }

    fn css_at(&self, cell_x: usize, cell_y: usize) -> Option<(i32, i32)> {
        let page_top = 4usize;
        if cell_y < page_top {
            return None;
        }
        let row = cell_y - page_top + self.scroll;
        let x = (cell_x as i32).saturating_mul(8).saturating_add(4);
        let y = (row as i32).saturating_mul(16).saturating_add(8);
        Some((x, y))
    }

    fn click_at(&mut self, px: i32, py: i32, label: &str) {
        if !self.live {
            self.note = "live off".into();
            return;
        }
        let before = self.url.clone();
        self.engine.begin_page();
        let result = if label.is_empty() {
            self.engine.click(px, py)
        } else {
            self.engine.activate(px, py, label)
        };
        if let Err(err) = result {
            self.note = clip_note(&err);
            self.engine.end_page();
            return;
        }
        if label.is_empty() {
            let _ = self.engine.watch_widget();
        }
        self.repaint(true);
        self.engine.end_page();
        if let Ok(href) = self.engine.href() {
            if !href.is_empty() && href != before {
                self.remember(before);
                self.url = href.clone();
                self.address = href;
                self.address_at = self.address.chars().count();
            }
        }
        self.note = "ready".into();
    }

    fn page_key(&mut self, key: Key) {
        let (name, code) = match key {
            Key::Enter => ("Enter", 13),
            Key::Backspace => ("Backspace", 8),
            Key::Left => ("ArrowLeft", 37),
            Key::Right => ("ArrowRight", 39),
            Key::Up => ("ArrowUp", 38),
            Key::Down => ("ArrowDown", 40),
            Key::Tab => ("Tab", 9),
            Key::Char(ch) => {
                let _ = self.engine.insert(&ch.to_string());
                self.repaint(false);
                return;
            }
            _ => return,
        };
        if let Err(err) = self.engine.key(name, code) {
            self.note = clip_note(&err);
            return;
        }
        if name == "Enter" {
            self.repaint(true);
            if let Ok(href) = self.engine.href() {
                if !href.is_empty() && href != self.url {
                    let before = self.url.clone();
                    self.remember(before);
                    self.url = href.clone();
                    self.address = href;
                }
            }
        }
    }

    fn scroll_by(&mut self, delta: i8) {
        if self.remote {
            match self.engine.scroll_view(i32::from(delta), self.cols, self.page_rows()) {
                Ok(view) => self.take_view(view),
                Err(err) => self.note = clip_note(&err),
            }
            return;
        }
        let room = self.rows.saturating_sub(6);
        let max = self.lines.len().saturating_sub(room);
        if delta > 0 {
            self.scroll = (self.scroll + usize::from(delta as u8)).min(max);
        } else {
            self.scroll = self.scroll.saturating_sub(usize::from((-delta) as u8));
        }
    }

    fn page_rows(&self) -> usize {
        self.rows.saturating_sub(5).max(8)
    }

    fn go(&mut self, target: &str, push: bool) {
        let target = target.trim();
        if target.is_empty() || target == "home" {
            self.go_home();
            return;
        }
        if push {
            if !self.url.is_empty() {
                self.remember(self.url.clone());
            }
            self.forward.clear();
        }
        if let Some(url) = page::web_url(target, &self.dir) {
            self.open_remote(&url);
        } else {
            self.open_file(target);
        }
    }

    fn open_remote(&mut self, url: &str) {
        self.note = "loading".into();
        self.engine.set_policy(self.js, self.live);
        self.engine.set_look(false, "en-US");
        match self.engine.show(url, self.cols, self.page_rows()) {
            Ok(view) => {
                self.remote = true;
                self.url = if view.url.is_empty() {
                    url.to_string()
                } else {
                    view.url.clone()
                };
                self.address = self.url.clone();
                self.address_at = self.address.chars().count();
                self.scroll = 0;
                self.take_view(view);
                self.take_bytes();
                self.note = "ready".into();
            }
            Err(err) => {
                self.remote = false;
                self.note = clip_note(&err);
                self.lines = vec![self.note.clone()];
            }
        }
    }

    fn open_file(&mut self, target: &str) {
        self.engine.close();
        self.remote = false;
        self.sixel.clear();
        self.hits.clear();
        match page::load_doc(target, &self.dir) {
            Ok(doc) => {
                self.bytes = doc.bytes;
                self.lines = doc.lines;
                self.url = target.to_string();
                self.address = self.url.clone();
                self.address_at = self.address.chars().count();
                self.scroll = 0;
                self.note = "ready".into();
            }
            Err(err) => {
                self.lines = vec![err.clone()];
                self.note = clip_note(&err);
                self.bytes = 0;
            }
        }
    }

    fn go_home(&mut self) {
        if !self.url.is_empty() {
            self.remember(self.url.clone());
        }
        self.forward.clear();
        self.engine.close();
        self.show_home();
    }

    fn focus_address(&mut self) {
        self.address_on = true;
        self.field = false;
        self.help = false;
        if self.address.is_empty() {
            self.address = self.url.clone();
            self.address_at = self.address.chars().count();
        }
    }

    fn show_home(&mut self) {
        self.remote = false;
        self.url.clear();
        self.address.clear();
        self.address_at = 0;
        self.address_on = false;
        self.field = false;
        self.scroll = 0;
        self.bytes = 0;
        self.sixel.clear();
        self.hits.clear();
        self.note = "ready".into();
        self.lines = vec![
            "A console browser.".into(),
            String::new(),
            "Type an address and press Enter.".into(),
            "A file opens as a page. An http address is drawn from Edge.".into(),
            String::new(),
            "[<] and [>] step through the pages you opened.".into(),
            "[reload] draws the page again. [home] returns here.".into(),
            "js runs scripts. live keeps the network on.".into(),
            String::new(),
            "The wheel scrolls. A click follows the page.".into(),
            "? lists the keys. F10 quits. Ctrl+C quits on the second press.".into(),
        ];
    }

    fn go_back(&mut self) {
        let Some(url) = self.back.pop() else {
            self.note = "no previous page".into();
            return;
        };
        if !self.url.is_empty() {
            self.forward.push(self.url.clone());
        }
        self.go(&url, false);
    }

    fn go_forward(&mut self) {
        let Some(url) = self.forward.pop() else {
            self.note = "no next page".into();
            return;
        };
        if !self.url.is_empty() {
            self.back.push(self.url.clone());
        }
        self.go(&url, false);
    }

    fn reload(&mut self) {
        if self.url.is_empty() {
            self.note = "no page".into();
            return;
        }
        let url = self.url.clone();
        self.go(&url, false);
    }

    fn remember(&mut self, url: String) {
        if self.back.last() != Some(&url) {
            self.back.push(url);
        }
    }

    fn repaint(&mut self, capture: bool) {
        self.engine.begin_page();
        match self.engine.refresh(self.cols, self.page_rows(), capture) {
            Ok(view) => self.take_view(view),
            Err(err) => self.note = clip_note(&err),
        }
        self.engine.end_page();
        self.take_bytes();
    }

    fn take_view(&mut self, view: crate::engine::WebView) {
        self.lines = view.lines.into_iter().skip(1).collect();
        self.hits = view.hits;
        self.sixel = view.widget_sixel;
        self.sixel_col = view.widget_col;
        self.sixel_row = view.widget_row.saturating_sub(1);
        if !view.url.is_empty() {
            self.url = view.url;
        }
        self.scroll = 0;
    }

    fn take_bytes(&mut self) {
        if let Ok((page, net)) = self.engine.traffic() {
            self.bytes = net.max(page);
        }
    }
}

fn clip_note(err: &str) -> String {
    err.chars().take(80).collect()
}

/// A local page prints a link as `<https://...>`. A click on that span opens it.
fn url_at_column(line: &str, column: usize) -> Option<String> {
    let mut plain = String::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        plain.push(ch);
    }
    let chars: Vec<char> = plain.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        if chars[index] == '<' {
            let start = index;
            if let Some(end) = chars[index + 1..].iter().position(|ch| *ch == '>') {
                let inside: String = chars[index + 1..index + 1 + end].iter().collect();
                let url = if inside.starts_with("http://") || inside.starts_with("https://") {
                    Some(inside)
                } else {
                    None
                };
                index = index + 1 + end;
                if column >= start && column <= index {
                    return url;
                }
            }
        }
        index += 1;
    }
    None
}

pub fn open_sample(dir: &Path) -> std::io::Result<PathBuf> {
    let page = dir.join("page.html");
    let mark = dir.join("mark.png");
    if !mark.exists() {
        write_mark(&mark)?;
    }
    if !page.exists() {
        std::fs::write(
            &page,
            r#"<!doctype html><html><head><title>Galley</title></head>
<body>
<h1>Galley</h1>
<p>The page is drawn in the terminal. A click follows a link.</p>
<p><a href="https://example.com">example.com</a></p>
<img src="mark.png" alt="mark">
<ul><li>Back and forward</li><li>Reload</li><li>Scripts and the network</li></ul>
</body></html>
"#,
        )?;
    }
    Ok(page)
}

fn write_mark(path: &Path) -> std::io::Result<()> {
    let width = 96u32;
    let height = 48u32;
    let mut image = image::RgbImage::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        let gold = x > 18 && x < 78 && y > 8 && y < 40;
        *pixel = if gold {
            image::Rgb([212, 175, 55])
        } else {
            image::Rgb([24, 18, 16])
        };
        if gold && ((x > 40 && x < 54 && y > 14 && y < 34) || (y > 22 && y < 28 && x > 28 && x < 68))
        {
            *pixel = image::Rgb([20, 16, 14]);
        }
    }
    image
        .save(path)
        .map_err(|err| std::io::Error::other(err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::{Screen, Spot};

    #[test]
    fn the_home_screen_is_galley() {
        let app = App::new(PathBuf::from("."));
        let screen = Screen::paint(&app, 96, 28);
        assert!(screen.frame.contains("Galley"), "{}", screen.frame);
        assert!(screen.frame.contains("[reload]"));
        assert!(screen.frame.contains("[home]"));
        assert!(screen.frame.contains("js on"));
        assert!(screen.frame.contains("A console browser."));
        assert!(screen.frame.contains("type an address"));
        assert_eq!(screen.spot_at(10, 10), Some(Spot::Page));
        assert_eq!(screen.spot_at(1, 1), Some(Spot::Back));
    }

    #[test]
    fn a_local_link_opens_from_its_span() {
        let line = "example.com <https://example.com>";
        assert_eq!(
            url_at_column(line, 20).as_deref(),
            Some("https://example.com")
        );
        assert!(url_at_column(line, 2).is_none());
    }

    #[test]
    fn a_local_page_keeps_its_title_and_picture() {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("galley-page-{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        let page = open_sample(&dir).unwrap();
        let mut app = App::new(dir.clone());
        app.open_target(page.to_str().unwrap());
        let joined = app.page_lines().join("\n");
        assert!(joined.contains("Galley"), "{joined}");
        assert!(joined.contains("example.com"), "{joined}");
        assert!(
            joined.chars().any(|ch| ch == '\u{2580}'
                || ('\u{2580}'..='\u{259F}').contains(&ch)
                || ('\u{1FB00}'..='\u{1FB3B}').contains(&ch)),
            "{joined}"
        );
        let screen = Screen::paint(&app, 96, 28);
        assert!(screen.frame.contains("[reload]"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn the_second_ctrl_c_quits() {
        let mut app = App::new(PathBuf::from("."));
        app.on_key(Key::CtrlC);
        assert!(!app.quit());
        app.on_key(Key::CtrlC);
        assert!(app.quit());
    }
}
