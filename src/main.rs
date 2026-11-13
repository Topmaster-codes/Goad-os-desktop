// SPDX-License-Identifier: GPL-3.0-or-later
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use amane::{
    Align, Animation, App, Audio, Battery, Center, Color, Column, Easing, End, Full, Horizontal,
    Layer, LayerWindow, Margin, Padding, Parent, Rectangle, Row, Scroll, Service, Size,
    SpaceBetween, SpaceEvenly, Stack, Start, Text, Vertical, Widget, Workspaces, Zone,
    request_frame,
};

#[derive(Clone, Copy)]
struct Palette {
    accent: Color,
    ink: Color,
    muted: Color,
    panel: Color,
    panel_soft: Color,
    edge: Color,
}

const BLUE_PALETTE: Palette = Palette {
    accent: Color::rgba(30, 122, 152, 255),
    ink: Color::rgba(220, 241, 245, 255),
    muted: Color::rgba(132, 177, 188, 255),
    panel: Color::rgba(10, 40, 50, 224),
    panel_soft: Color::rgba(16, 60, 72, 205),
    edge: Color::rgba(30, 122, 152, 150),
};

const RED_PALETTE: Palette = Palette {
    accent: Color::rgba(196, 35, 69, 255),
    ink: Color::rgba(245, 225, 228, 255),
    muted: Color::rgba(217, 144, 156, 255),
    panel: Color::rgba(38, 11, 15, 224),
    panel_soft: Color::rgba(72, 21, 32, 205),
    edge: Color::rgba(196, 35, 69, 150),
};

fn palette() -> Palette {
    static PALETTE: OnceLock<Palette> = OnceLock::new();
    *PALETTE.get_or_init(|| {
        let state_home = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
            });
        let theme_file = state_home.map(|home| home.join("amane/theme"));
        if theme_file
            .and_then(|path| fs::read_to_string(path).ok())
            .is_some_and(|theme| theme.trim() == "ragebait-red")
        {
            RED_PALETTE
        } else {
            BLUE_PALETTE
        }
    })
}

fn accent(alpha: u8) -> Color {
    let color = palette().accent;
    Color::rgba(color.red(), color.green(), color.blue(), alpha)
}
const BAR_HEIGHT: f32 = 30.0;
const TOP_MARGIN: i32 = 3;
const PANEL_HEIGHT: f32 = 28.0;

// Amane's services redraw the bar when a service changes. This small service
// keeps the local clock fresh without continuously redrawing the whole shell.
struct Clock {
    date: String,
    time: String,
}

impl Service for Clock {
    fn new() -> Self {
        let (date, time) = read_clock();
        Self { date, time }
    }

    fn interval() -> Duration {
        Duration::from_secs(15)
    }

    fn update(&mut self) -> bool {
        let (date, time) = read_clock();
        if date == self.date && time == self.time {
            return false;
        }
        self.date = date;
        self.time = time;
        true
    }
}

fn read_clock() -> (String, String) {
    Command::new("date")
        .arg("+%a %d %b|%I:%M %p")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .and_then(|value| {
            let (date, time) = value.split_once('|')?;
            Some((date.to_owned(), time.to_owned()))
        })
        .unwrap_or_else(|| (String::from("--- -- ---"), String::from("--:-- --")))
}

// Small native resource monitor. CPU is calculated from deltas in /proc/stat;
// RAM uses MemTotal and MemAvailable from /proc/meminfo. No extra crate needed.
struct ResourceStats {
    cpu_percent: u8,
    ram_percent: u8,
    previous_cpu: Option<(u64, u64)>,
}

impl Service for ResourceStats {
    fn new() -> Self {
        Self {
            cpu_percent: 0,
            ram_percent: read_ram_percent().unwrap_or(0),
            previous_cpu: read_cpu_counters(),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(2)
    }

    fn update(&mut self) -> bool {
        let mut changed = false;

        if let Some((total, idle)) = read_cpu_counters() {
            if let Some((previous_total, previous_idle)) = self.previous_cpu {
                let total_delta = total.saturating_sub(previous_total);
                let idle_delta = idle.saturating_sub(previous_idle);
                if total_delta > 0 {
                    let busy = total_delta.saturating_sub(idle_delta);
                    let next = ((busy.saturating_mul(100) / total_delta).min(100)) as u8;
                    if next != self.cpu_percent {
                        self.cpu_percent = next;
                        changed = true;
                    }
                }
            }
            self.previous_cpu = Some((total, idle));
        }

        if let Some(next) = read_ram_percent()
            && next != self.ram_percent
        {
            self.ram_percent = next;
            changed = true;
        }

        changed
    }
}

fn read_cpu_counters() -> Option<(u64, u64)> {
    let contents = fs::read_to_string("/proc/stat").ok()?;
    let mut fields = contents.lines().next()?.split_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }
    let counters: Vec<u64> = fields
        .take(8)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if counters.len() < 4 {
        return None;
    }
    let total = counters.iter().copied().sum();
    let idle = counters[3] + counters.get(4).copied().unwrap_or(0);
    Some((total, idle))
}

fn read_ram_percent() -> Option<u8> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = None;
    let mut available_kb = None;
    for line in contents.lines() {
        let mut fields = line.split_whitespace();
        let Some(key) = fields.next() else {
            continue;
        };
        match key {
            "MemTotal:" => total_kb = fields.next()?.parse::<u64>().ok(),
            "MemAvailable:" => available_kb = fields.next()?.parse::<u64>().ok(),
            _ => {}
        }
    }
    let total = total_kb?;
    let available = available_kb?;
    if total == 0 {
        return None;
    }
    let used = total.saturating_sub(available);
    Some(((used.saturating_mul(100) / total).min(100)) as u8)
}

// Keep the bar visible while the pointer is over the bar or notification
// dropdown. Fullscreen clients suppress the top-edge trigger; otherwise reveal
// from the 3px strip and hide when a client overlaps the bar area.
struct BarVisibility {
    visible: bool,
}

impl Service for BarVisibility {
    fn new() -> Self {
        Self {
            visible: should_show_bar(),
        }
    }

    fn interval() -> Duration {
        Duration::from_millis(220)
    }

    fn update(&mut self) -> bool {
        let next = should_show_bar();
        if next == self.visible {
            return false;
        }
        self.visible = next;
        true
    }
}

fn should_show_bar() -> bool {
    if notification_dropdown_pins_bar() || notification_surface_hovered() {
        return true;
    }

    const SCRIPT: &str = r#"
set -u
workspace="$(hyprctl -j activeworkspace 2>/dev/null | jq -r '.id // empty' 2>/dev/null || true)"
cursor_y="$(hyprctl -j cursorpos 2>/dev/null | jq -r '.y // 999999' 2>/dev/null || true)"
# If Hyprland or jq is temporarily unavailable, fail open (show the bar).
if [ -z "$workspace" ]; then
    echo show
    exit 0
fi

clients="$(hyprctl -j clients 2>/dev/null || true)"

# A fullscreen client on the active workspace suppresses the edge trigger.
# This prevents games from opening the bar when the pointer touches the top.
if printf '%s\n' "$clients" | jq -e --argjson ws "$workspace" '
    any(.[];
        (.mapped // false) == true
        and (.hidden // false) == false
        and .workspace.id == $ws
        and (
            .fullscreen == true
            or .fullscreen == 2
            or .fullscreen == 3
        )
    )' >/dev/null 2>&1; then
    echo hide
    exit 0
fi

# Menus are Wayland layer surfaces and do not reliably appear in the Hyprland
# client list. Keep the bar available for the full lifetime of any Rofi menu.
if command -v pgrep >/dev/null 2>&1 \
    && { pgrep -u "$(id -u)" -x rofi >/dev/null 2>&1 \
        || pgrep -u "$(id -u)" -x rofi-wayland >/dev/null 2>&1; }; then
    echo show
    exit 0
fi

case "$cursor_y" in
    ''|*[!0-9-]*) cursor_y=999999 ;;
esac

# Reveal the bar when the pointer reaches the 3px strip at the top edge.
if [ "$cursor_y" -le 3 ]; then
    echo show
    exit 0
fi

if printf '%s\n' "$clients" | jq -e --argjson ws "$workspace" '
    any(.[];
        (.mapped // false) == true
        and (.hidden // false) == false
        and .workspace.id == $ws
        and (.at[1] // 999999) < 33
        and ((.at[1] // 0) + (.size[1] // 0)) > 3
    )' >/dev/null 2>&1; then
    echo hide
else
    echo show
fi
"#;

    let Ok(output) = Command::new("sh").arg("-c").arg(SCRIPT).output() else {
        return true;
    };
    let decision = String::from_utf8_lossy(&output.stdout);
    !decision.trim().eq("hide")
}

fn label(value: impl Into<String>, size: f32, color: Color) -> Text {
    Text::new(value).size(size).color(color)
}

// Hyprland supplies the backdrop blur through the amane-bar layer rule.
// Keep the widget fill translucent and do not apply Amane's own blur here.
fn panel(width: f32, horizontal: impl Into<Align>, child: impl Widget + 'static) -> Rectangle {
    Rectangle::new()
        .width(width)
        .height(PANEL_HEIGHT)
        .fill(palette().panel)
        .radius(8.0)
        .border(1.0, palette().edge)
        .opacity(1.0)
        .padding(Padding {
            top: 1.0,
            right: 5.0,
            bottom: 1.0,
            left: 5.0,
        })
        .align_child(horizontal, Center)
        .child(child)
}

fn metric_chip(value: String) -> Rectangle {
    Rectangle::new()
        .width(42.0)
        .height(22.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .border(1.0, accent(80))
        .align_child(Center, Center)
        .child(label(value, 8.0, palette().ink))
}

fn workspace_panel() -> Rectangle {
    let mut cells: Vec<Box<dyn Widget>> = Vec::new();
    let resources = ResourceStats::read();
    cells.push(Box::new(metric_chip(format!(
        "CPU {}%",
        resources.cpu_percent
    ))));
    cells.push(Box::new(metric_chip(format!(
        "RAM {}%",
        resources.ram_percent
    ))));
    drop(resources);
    let workspaces = Workspaces::read();

    for workspace in workspaces.list() {
        if !workspace.active() && workspace.windows() == 0 {
            continue;
        }
        let id = workspace.id();
        let index = workspace.index();
        let focused = workspace.focused();
        let (fill, text_color) = if focused {
            (palette().accent, Color::WHITE)
        } else {
            (palette().panel_soft, palette().muted)
        };

        let cell = Rectangle::new()
            .width(19.0)
            .height(22.0)
            .fill(fill)
            .radius(5.0)
            .border(1.0, accent(90))
            .align_child(Center, Center)
            .child(label(format!("{index}"), 10.0, text_color))
            .on_click(move |_| Workspaces::focus(id));

        cells.push(Box::new(cell));
    }
    let workspace_count = workspaces
        .list()
        .iter()
        .filter(|workspace| workspace.active() || workspace.windows() > 0)
        .count();
    drop(workspaces);

    // CPU/RAM + 19px per actual workspace, gaps, and inner edge padding.
    let width = 97.0 + workspace_count as f32 * 22.0;
    panel(width, Start, Row::new(cells).gap(3.0))
}

// Amane reports positive Scroll.y for downward scrolling. Raising the wheel
// raises volume; lowering it reduces volume. Two percent per wheel line is
// responsive without making a normal wheel notch too aggressive.
fn adjust_default_output_volume(scroll: Scroll) {
    if !scroll.y.is_finite() || scroll.y.abs() < 0.05 {
        return;
    }

    let amount = (scroll.y.abs() * 2.0).clamp(0.5, 10.0);
    let direction = if scroll.y < 0.0 { "+" } else { "-" };
    let adjustment = format!("{amount:.1}%{direction}");

    let _ = Command::new("wpctl")
        .args([
            "set-volume",
            "@DEFAULT_SINK@",
            adjustment.as_str(),
            "--limit",
            "1.0",
        ])
        .status();
}

fn audio_chip() -> Rectangle {
    let audio = Audio::read();
    let value = if audio.muted() {
        String::from("MUTE")
    } else {
        format!("VOL {:02}%", audio.volume())
    };
    drop(audio);

    Rectangle::new()
        .width(42.0)
        .height(22.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .border(1.0, accent(80))
        .align_child(Center, Center)
        .child(label(value, 8.2, palette().ink))
        .on_click(|_| launch_audio_device_menu())
        .on_scroll(adjust_default_output_volume)
}

fn launch_audio_device_menu() {
    launch_amane_script("audio-device-menu");
}

struct NetworkdStatus {
    active: bool,
    ethernet_in_use: bool,
}

impl Service for NetworkdStatus {
    fn new() -> Self {
        Self {
            active: networkd_is_active(),
            ethernet_in_use: networkd_ethernet_in_use(),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3)
    }

    fn update(&mut self) -> bool {
        let active = networkd_is_active();
        let ethernet_in_use = networkd_ethernet_in_use();
        if active == self.active && ethernet_in_use == self.ethernet_in_use {
            return false;
        }
        self.active = active;
        self.ethernet_in_use = ethernet_in_use;
        true
    }
}

fn networkd_is_active() -> bool {
    Command::new("systemctl")
        .args(["is-active", "--quiet", "systemd-networkd.service"])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn networkd_ethernet_in_use() -> bool {
    let Ok(output) = Command::new("networkctl")
        .args(["--no-pager", "--no-legend", "list"])
        .output()
    else {
        return false;
    };
    if !output.status.success() {
        return false;
    }

    String::from_utf8_lossy(&output.stdout).lines().any(|line| {
        let mut fields = line.split_whitespace();
        let _index = fields.next();
        let _name = fields.next();
        let kind = fields.next();
        let operational = fields.next();
        matches!(kind, Some("ether")) && matches!(operational, Some("routable" | "degraded"))
    })
}

fn network_chip() -> Option<Rectangle> {
    let status = NetworkdStatus::read();
    let active = status.active;
    let ethernet_in_use = status.ethernet_in_use;
    drop(status);
    if !active || ethernet_in_use {
        return None;
    }

    Some(
        Rectangle::new()
            .width(48.0)
            .height(22.0)
            .fill(palette().panel_soft)
            .radius(5.0)
            .border(1.0, accent(80))
            .align_child(Center, Center)
            .child(label("NETD ON", 8.2, palette().accent))
            .on_click(|_| launch_amane_script("network-manager-menu")),
    )
}

struct NotificationHistory {
    open: bool,
    entries: Vec<String>,
    last_history_refresh: Instant,
}

impl Service for NotificationHistory {
    fn new() -> Self {
        // Never restore an open popup across app restarts. The fallback state
        // path lives in the persistent cache when XDG_RUNTIME_DIR is absent.
        write_notification_dropdown_phase("closed");
        Self {
            open: false,
            entries: read_notification_history(),
            last_history_refresh: Instant::now(),
        }
    }

    fn interval() -> Duration {
        Duration::from_millis(100)
    }

    fn update(&mut self) -> bool {
        let mut open = read_notification_dropdown_state();
        if open && notification_hovered_outside_long_enough(Duration::from_millis(600)) {
            write_notification_dropdown_phase("closing");
            open = false;
        }
        let entries = if self.last_history_refresh.elapsed() >= Duration::from_secs(1) {
            self.last_history_refresh = Instant::now();
            read_notification_history()
        } else {
            self.entries.clone()
        };
        if open == self.open && entries == self.entries {
            return false;
        }
        self.open = open;
        self.entries = entries;
        true
    }
}

fn notification_dropdown_state_path() -> Option<PathBuf> {
    if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
        return Some(PathBuf::from(runtime_dir).join("amane-notifications-open"));
    }
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".cache/amane/notifications-open"))
}

fn read_notification_dropdown_state() -> bool {
    read_notification_dropdown_phase() == "open"
}

fn notification_dropdown_phase_path() -> Option<PathBuf> {
    notification_dropdown_state_path()
}

fn read_notification_dropdown_phase() -> String {
    notification_dropdown_phase_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|phase| match phase.trim() {
            "open" | "1" => String::from("open"),
            "closing" => String::from("closing"),
            _ => String::from("closed"),
        })
        .unwrap_or_else(|| String::from("closed"))
}

fn notification_dropdown_pins_bar() -> bool {
    read_notification_dropdown_phase() != "closed"
}

fn notification_surface_hovered() -> bool {
    let state = notification_hover_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.bar || state.dropdown
}

fn write_notification_dropdown_phase(phase: &str) {
    let Some(path) = notification_dropdown_phase_path() else {
        return;
    };
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_err()
    {
        return;
    }
    let _ = fs::write(path, phase);
}

fn toggle_notification_dropdown() {
    let next = if read_notification_dropdown_state() {
        "closing"
    } else {
        "open"
    };
    write_notification_dropdown_phase(next);
}

#[derive(Default)]
struct NotificationHover {
    bar: bool,
    dropdown: bool,
    outside_since: Option<Instant>,
}

fn notification_hover_state() -> &'static Mutex<NotificationHover> {
    static HOVER: OnceLock<Mutex<NotificationHover>> = OnceLock::new();
    HOVER.get_or_init(|| Mutex::new(NotificationHover::default()))
}

fn update_notification_hover(bar: bool, hovering: bool) {
    let mut state = notification_hover_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if bar {
        state.bar = hovering;
    } else {
        state.dropdown = hovering;
    }
    if state.bar || state.dropdown {
        state.outside_since = None;
    } else if state.outside_since.is_none() {
        state.outside_since = Some(Instant::now());
    }
}

fn notification_hovered_outside_long_enough(grace: Duration) -> bool {
    let mut state = notification_hover_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if state.bar || state.dropdown {
        state.outside_since = None;
        return false;
    }
    let since = state.outside_since.get_or_insert_with(Instant::now);
    since.elapsed() >= grace
}

fn notification_bar_hover(hovering: bool) {
    update_notification_hover(true, hovering);
}

fn notification_dropdown_hover(hovering: bool) {
    update_notification_hover(false, hovering);
}

fn notification_ticker_state() -> &'static Mutex<Option<Instant>> {
    static TICKER_STARTED: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    TICKER_STARTED.get_or_init(|| Mutex::new(None))
}

fn notification_text_offset(index: usize, overflow: f32, open: bool) -> f32 {
    if !open || overflow <= 0.5 {
        return 0.0;
    }
    let elapsed = {
        let mut started = notification_ticker_state()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let instant = started.get_or_insert_with(Instant::now);
        instant.elapsed().as_secs_f32() + index as f32 * 0.37
    };
    request_frame();

    let phase = elapsed % 8.0;
    let progress = if phase < 1.0 {
        0.0
    } else if phase < 4.0 {
        let value = (phase - 1.0) / 3.0;
        value * value * (3.0 - 2.0 * value)
    } else if phase < 5.0 {
        1.0
    } else {
        let value = 1.0 - (phase - 5.0) / 3.0;
        value * value * (3.0 - 2.0 * value)
    };
    overflow * progress
}

fn reset_notification_ticker(open: bool) {
    let mut started = notification_ticker_state()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if open {
        started.get_or_insert_with(Instant::now);
    } else {
        *started = None;
    }
}

fn notification_row(entry: String, index: usize, open: bool) -> Rectangle {
    const TEXT_VIEWPORT: f32 = 358.0;
    let text = label(entry, 8.0, palette().ink);
    let text_width = match text.width() {
        Size::Fixed(width) => width.max(1.0),
        Size::Parent => TEXT_VIEWPORT,
    };
    let offset = notification_text_offset(index, text_width - TEXT_VIEWPORT, open);
    let scrolling_text = Rectangle::new()
        .width(text_width)
        .height(22.0)
        .align_child(Start, Center)
        .translate(-offset, 0.0)
        .child(text);

    Rectangle::new()
        .width(Parent)
        .height(32.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .padding(Padding {
            top: 2.0,
            right: 6.0,
            bottom: 2.0,
            left: 6.0,
        })
        .align_child(Start, Center)
        .clip()
        .child(scrolling_text)
}

fn read_notification_history() -> Vec<String> {
    const SCRIPT: &str = r#"
if ! command -v makoctl >/dev/null 2>&1 || ! command -v jq >/dev/null 2>&1; then
    exit 0
fi
active="$(makoctl list -j 2>/dev/null || printf '[]')"
history="$(makoctl history -j 2>/dev/null || printf '[]')"
jq -nr --argjson active "$active" --argjson history "$history" '
    ($active + $history) | unique_by(.id) | sort_by(.id) | reverse | .[:10][] |
    ((.app_name // "Notification") + "  ·  " + (.summary // "Notification") +
      (if (.body // "") == "" then "" else "  —  " + (.body | gsub("[\\r\\n\\t]+"; " ")) end))
' 2>/dev/null || true
"#;

    Command::new("sh")
        .arg("-c")
        .arg(SCRIPT)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .take(10)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn notification_button() -> Rectangle {
    let count = NotificationHistory::read().entries.len();
    Rectangle::new()
        .width(48.0)
        .height(22.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .border(1.0, accent(80))
        .align_child(Center, Center)
        .child(label(format!("NTF {count:02}"), 8.0, palette().ink))
        .on_click(|_| toggle_notification_dropdown())
}

fn notification_history_dropdown() -> LayerWindow {
    let history = NotificationHistory::read();
    let is_open = history.open;
    let entries = history.entries.clone();
    drop(history);
    reset_notification_ticker(is_open);

    static DROPDOWN_ANIMATION: OnceLock<Mutex<Animation<f32>>> = OnceLock::new();
    let animation = DROPDOWN_ANIMATION.get_or_init(|| {
        Mutex::new(
            Animation::new(0.0)
                .duration(Duration::from_millis(360))
                .easing(Easing::Out),
        )
    });
    let progress = {
        let mut animation = animation
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        animation.to(if is_open { 1.0 } else { 0.0 });
        animation.value()
    };
    let visible = is_open || progress > 0.001;
    if !is_open && progress <= 0.001 && read_notification_dropdown_phase() == "closing" {
        write_notification_dropdown_phase("closed");
    }
    let bounce = ((progress * std::f32::consts::PI * 2.0).sin() * (1.0 - progress) * 0.06).max(0.0);

    let mut rows: Vec<Box<dyn Widget>> = vec![Box::new(
        Text::new("Recent notifications")
            .size(10.0)
            .color(palette().ink),
    )];
    if entries.is_empty() {
        rows.push(Box::new(label(
            "No recent notifications",
            9.0,
            palette().muted,
        )));
    } else {
        for (index, entry) in entries.into_iter().enumerate() {
            rows.push(Box::new(notification_row(entry, index, is_open)));
        }
    }

    let content = Column::new(rows).width(Parent).height(Parent).gap(5.0);
    let child = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(palette().panel)
        .radius_top_left(0.0)
        .radius_top_right(0.0)
        .radius_bottom_right(10.0)
        .radius_bottom_left(10.0)
        .border(1.0, palette().edge)
        .padding(Padding {
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
            left: 10.0,
        })
        .opacity(progress.clamp(0.0, 1.0))
        .scale(0.96 + progress * 0.04 + bounce)
        .translate(0.0, -20.0 * (1.0 - progress) + bounce * 30.0)
        .child(content)
        .on_hover(notification_dropdown_hover);

    let window = LayerWindow::new()
        .width(390.0)
        .height(420.0)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Right)
        .margin(Margin {
            top: 31,
            right: 18,
            bottom: 0,
            left: 0,
        })
        .layer(Layer::Overlay)
        .space(Zone::Ignore)
        .namespace("amane-notification-history")
        .visible(visible)
        .child(child);

    if is_open {
        window
    } else {
        // Hidden and closing layer surfaces must not retain a stale pointer
        // region over windows underneath them.
        window.click_through()
    }
}

fn battery_chip() -> Option<Rectangle> {
    let battery = Battery::read();
    let present = battery.present();
    let percent = battery.percent();
    let charging = battery.charging();
    drop(battery);

    if !present {
        return None;
    }

    let marker = if charging { "+" } else { "" };
    Some(
        Rectangle::new()
            .width(42.0)
            .height(22.0)
            .fill(palette().panel_soft)
            .radius(5.0)
            .border(1.0, accent(80))
            .align_child(Center, Center)
            .child(label(
                format!("BAT {percent:02}%{marker}"),
                7.2,
                palette().ink,
            )),
    )
}

struct PowerDaemon {
    available: bool,
}

impl Service for PowerDaemon {
    fn new() -> Self {
        Self {
            available: power_daemon_active(),
        }
    }

    fn interval() -> Duration {
        Duration::from_secs(3)
    }

    fn update(&mut self) -> bool {
        let next = power_daemon_active();
        if next == self.available {
            return false;
        }
        self.available = next;
        true
    }
}

fn power_daemon_active() -> bool {
    Command::new("sh")
        .arg("-c")
        .arg("command -v powerprofilesctl >/dev/null 2>&1 && systemctl is-active --quiet power-profiles-daemon.service")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn amane_config_dir() -> Option<PathBuf> {
    if let Some(config_home) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(config_home).join("amane"));
    }

    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".config/amane"))
}

fn launch_amane_script(name: &str) {
    if let Some(config_dir) = amane_config_dir() {
        let script = config_dir.join("scripts").join(name);
        let _ = Command::new(script).spawn();
    }
}

fn power_profile_name() -> String {
    Command::new("powerprofilesctl")
        .arg("get")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| String::from("unavailable"))
}

fn power_chip() -> Option<Rectangle> {
    let battery_present = Battery::read().present();
    if !battery_present {
        return None;
    }

    let daemon = PowerDaemon::read();
    let available = daemon.available;
    drop(daemon);
    if !available {
        return None;
    }

    let profile = power_profile_name();
    let (value, color) = match profile.as_str() {
        "performance" => ("PWR HIGH", palette().accent),
        "balanced" => ("PWR BAL", palette().ink),
        "power-saver" => ("PWR SAVE", palette().muted),
        _ => ("PWR --", palette().muted),
    };

    Some(
        Rectangle::new()
            .width(48.0)
            .height(22.0)
            .fill(palette().panel_soft)
            .radius(7.0)
            .border(1.0, accent(100))
            .align_child(Center, Center)
            .child(label(value, 7.2, color))
            .on_click(|_| launch_amane_script("power-profile-menu")),
    )
}

fn power_menu_button() -> Rectangle {
    Rectangle::new()
        .width(28.0)
        .height(22.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .border(1.0, accent(120))
        .align_child(Center, Center)
        .child(label("⏻", 12.0, palette().accent))
        .on_click(|_| launch_amane_script("power-menu"))
}

fn monitor_button() -> Rectangle {
    Rectangle::new()
        .width(34.0)
        .height(22.0)
        .fill(palette().panel_soft)
        .radius(5.0)
        .border(1.0, accent(100))
        .align_child(Center, Center)
        .child(label("MON", 7.2, palette().ink))
        .on_click(|_| launch_amane_script("monitor-manager-menu"))
}

fn status_panel() -> Rectangle {
    let mut items: Vec<Box<dyn Widget>> = Vec::new();
    let mut item_widths = Vec::new();

    if let Some(network) = network_chip() {
        items.push(Box::new(network));
        item_widths.push(48.0);
    }
    items.push(Box::new(audio_chip()));
    item_widths.push(42.0);

    if let Some(battery) = battery_chip() {
        items.push(Box::new(battery));
        item_widths.push(42.0);
    }
    if let Some(power) = power_chip() {
        items.push(Box::new(power));
        item_widths.push(48.0);
    }
    items.push(Box::new(notification_button()));
    item_widths.push(48.0);
    items.push(Box::new(monitor_button()));
    item_widths.push(34.0);
    items.push(Box::new(power_menu_button()));
    item_widths.push(28.0);

    let width =
        10.0 + item_widths.iter().sum::<f32>() + (items.len().saturating_sub(1) as f32 * 2.0);
    panel(width, End, Row::new(items).gap(2.0))
}

fn clock_panel() -> Rectangle {
    let clock = Clock::read();
    let date = clock.date.clone();
    let time = clock.time.clone();
    let items = Row::new(vec![
        Box::new(label(date, 9.0, palette().ink)),
        Box::new(label(time, 10.0, palette().ink)),
    ])
    .width(Parent)
    .height(Parent)
    .justify(SpaceEvenly)
    .align(Center);
    panel(230.0, Center, items)
}

fn bar() -> LayerWindow {
    let visible = BarVisibility::read().visible;
    let side_panels = Row::new(vec![Box::new(workspace_panel()), Box::new(status_panel())])
        .width(Parent)
        .height(Parent)
        .justify(SpaceBetween)
        .align(Center);
    let centered_clock = Rectangle::new()
        .width(Parent)
        .height(Parent)
        .fill(Color::TRANSPARENT)
        .align_child(Center, Center)
        .child(clock_panel());
    let content = Stack::new(vec![Box::new(side_panels), Box::new(centered_clock)])
        .width(Parent)
        .height(Parent);

    LayerWindow::new()
        .width(Full)
        .height(BAR_HEIGHT)
        .anchor_vertical(Vertical::Top)
        .anchor_horizontal(Horizontal::Middle)
        .margin(Margin {
            top: TOP_MARGIN,
            right: 18,
            bottom: 0,
            left: 18,
        })
        .layer(Layer::Top)
        .space(Zone::Respect)
        .namespace("amane-bar")
        .visible(visible)
        .child(
            Rectangle::new()
                .width(Parent)
                .height(Parent)
                .fill(Color::TRANSPARENT)
                .child(content)
                .on_hover(notification_bar_hover),
        )
}

fn main() {
    App::new()
        .font("JetBrainsMono Nerd Font")
        .window(bar)
        .window(notification_history_dropdown)
        .run();
}
