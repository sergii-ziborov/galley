use crate::app::App;

const GOLD: &str = "\x1b[38;5;220m";
const SALMON: &str = "\x1b[38;5;203m";
const CYAN: &str = "\x1b[38;5;51m";
const RED: &str = "\x1b[38;5;196m";
const DIM: &str = "\x1b[38;5;131m";
const HINT: &str = "\x1b[38;5;245m";
const RULE: &str = "\x1b[38;5;160m";
const KEEP: &str = "\x1b[0;48;5;0m";
const HOT: &str = "\x1b[1;4;30;48;5;220m";
const ON: &str = "\x1b[30;48;5;220m";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Back,
    Forward,
    Reload,
    Home,
    Js,
    Live,
    Address,
    Help,
    HelpClose,
    Page,
}

struct Hit {
    x: usize,
    y: usize,
    w: usize,
    spot: Spot,
}

pub struct Screen {
    pub frame: String,
    pub cursor: Option<(usize, usize)>,
    hits: Vec<Hit>,
}

impl Screen {
    pub fn paint(app: &App, cols: usize, rows: usize) -> Self {
        let mut screen = Self {
            frame: String::new(),
            cursor: None,
            hits: Vec::new(),
        };
        screen.frame.push_str("\x1b]0;Galley\x07");
        if cols < 40 || rows < 10 {
            screen.line(0, cols, "too small    F10 quits", RED);
            return screen;
        }
        screen.line(0, cols, "", DIM);
        screen.put(0, 0, cols, "Galley", GOLD);
        let help = "[?]";
        let help_x = cols.saturating_sub(help.chars().count() + 1);
        screen.put(help_x, 0, cols, help, if app.help() { ON } else { GOLD });
        screen.hit(help_x, 0, help.chars().count(), Spot::Help);

        screen.line(1, cols, "", DIM);
        let mut x = 0usize;
        for (label, spot) in [
            ("[<]", Spot::Back),
            ("[>]", Spot::Forward),
            ("[reload]", Spot::Reload),
            ("[home]", Spot::Home),
        ] {
            let hot = app.hot() == Some(spot);
            screen.put(x, 1, cols, label, if hot { HOT } else { GOLD });
            screen.hit(x, 1, label.chars().count(), spot);
            x += label.chars().count() + 1;
        }
        let js = if app.js_on() { "js on" } else { "js off" };
        let live = if app.live_on() { "live on" } else { "live off" };
        let chips = format!("{js}  {live}");
        let chip_x = cols.saturating_sub(chips.chars().count() + 1);
        if chip_x > x {
            let js_ink = if app.hot() == Some(Spot::Js) {
                HOT
            } else if app.js_on() {
                ON
            } else {
                DIM
            };
            screen.put(chip_x, 1, cols, js, js_ink);
            screen.hit(chip_x, 1, js.chars().count(), Spot::Js);
            let live_x = chip_x + js.chars().count() + 2;
            let live_ink = if app.hot() == Some(Spot::Live) {
                HOT
            } else if app.live_on() {
                ON
            } else {
                DIM
            };
            screen.put(live_x, 1, cols, live, live_ink);
            screen.hit(live_x, 1, live.chars().count(), Spot::Live);
        }

        let address = app.address_line(cols.saturating_sub(2));
        let ink = if address == "type an address" { HINT } else { CYAN };
        screen.line(2, cols, &address, ink);
        screen.hit(0, 2, cols.saturating_sub(1), Spot::Address);
        if app.address_on() {
            let caret = app.address_cursor().min(cols.saturating_sub(2));
            screen.cursor = Some((caret + 1, 3));
        }

        screen.line(3, cols, &"-".repeat(cols.saturating_sub(1)), RULE);

        let page_top = 4usize;
        let status = rows - 2;
        let hint_y = rows - 1;
        let room = status.saturating_sub(page_top);
        if app.help() {
            screen.help_sheet(cols, page_top, status);
        } else {
            for (index, line) in app.page_lines().iter().skip(app.scroll()).take(room).enumerate()
            {
                screen.span(0, page_top + index, cols, line);
            }
            for y in (page_top + app.page_lines().len().saturating_sub(app.scroll()))..status {
                screen.line(y, cols, "", DIM);
            }
            let wide = cols.saturating_sub(1);
            for y in page_top..status {
                screen.hit(0, y, wide, Spot::Page);
            }
        }
        screen.line(status, cols, &app.status_line(), DIM);
        screen.line(
            hint_y,
            cols,
            "wheel scrolls    click follows the page    ? keys    F10 quits",
            DIM,
        );
        if !app.help() {
            if let Some(place) = app.sixel_place(page_top) {
                screen.frame.push_str(&place);
            }
        }
        screen
    }

    pub fn spot_at(&self, x: usize, y: usize) -> Option<Spot> {
        self.hits
            .iter()
            .rev()
            .find(|hit| hit.y == y && x >= hit.x && x < hit.x + hit.w)
            .map(|hit| hit.spot)
    }

    fn help_sheet(&mut self, cols: usize, top: usize, bottom: usize) {
        let mut lines = vec!["Galley keys".to_string(), String::new()];
        for (key, what) in [
            ("Enter", "open the address"),
            ("Ctrl+L", "focus the address"),
            ("Esc", "leave the address, or close this"),
            ("F5", "reload"),
            ("Backspace", "back, when the address is closed"),
            ("[<] [>]", "back and forward"),
            ("[reload]", "reload the page"),
            ("[home]", "this screen"),
            ("js  live", "scripts and the network"),
            ("Wheel", "scroll"),
            ("Click", "a link, a button, or the page"),
            ("F10", "quit"),
            ("Ctrl+C", "quit on the second press"),
        ] {
            lines.push(format!("{key:<12}{what}"));
        }
        lines.push(String::new());
        lines.push("Click this sheet, or [?], to close it.".into());
        for (index, line) in lines.iter().enumerate() {
            let y = top + index;
            if y >= bottom {
                break;
            }
            let ink = if index == 0 { GOLD } else { SALMON };
            self.line(y, cols, line, ink);
        }
        for y in (top + lines.len())..bottom {
            self.line(y, cols, "", DIM);
        }
        self.hit(0, top, cols.saturating_sub(1), Spot::HelpClose);
    }

    fn hit(&mut self, x: usize, y: usize, w: usize, spot: Spot) {
        if w > 0 {
            self.hits.push(Hit { x, y, w, spot });
        }
    }

    fn line(&mut self, y: usize, cols: usize, text: &str, ink: &str) {
        self.frame.push_str(&format!("\x1b[{};1H{ink}", y + 1));
        let mut count = 0usize;
        for ch in text.chars() {
            if ch == '\u{1b}' || ch == '\n' || ch == '\r' {
                continue;
            }
            if count + 1 >= cols {
                break;
            }
            self.frame.push(ch);
            count += 1;
        }
        while count + 1 < cols {
            self.frame.push(' ');
            count += 1;
        }
        self.frame.push_str(KEEP);
    }

    fn put(&mut self, x: usize, y: usize, cols: usize, text: &str, ink: &str) {
        let room = cols.saturating_sub(1).saturating_sub(x);
        if room == 0 {
            return;
        }
        let shown: String = text.chars().take(room).collect();
        self.frame
            .push_str(&format!("\x1b[{};{}H{ink}{shown}{KEEP}", y + 1, x + 1));
    }

    fn span(&mut self, x: usize, y: usize, cols: usize, text: &str) {
        self.frame.push_str(&format!("\x1b[{};{}H", y + 1, x + 1));
        if !text.starts_with('\u{1b}') {
            self.frame.push_str(SALMON);
        }
        let mut count = 0usize;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '\u{1b}' {
                self.frame.push(ch);
                for next in chars.by_ref() {
                    self.frame.push(next);
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
            if ch == '\n' || ch == '\r' {
                continue;
            }
            if count + 1 >= cols {
                break;
            }
            self.frame.push(ch);
            count += 1;
        }
        self.frame.push_str(KEEP);
        while count + 1 < cols {
            self.frame.push(' ');
            count += 1;
        }
    }
}

pub fn sixel_at(page_top: usize, col: usize, row: usize, sixel: &str) -> String {
    if sixel.is_empty() {
        return String::new();
    }
    format!(
        "\x1b[{};{}H{sixel}",
        page_top + row + 1,
        col + 1
    )
}
