use crate::{
    icons,
    motion::{self, Curve},
    runtime::{Page, Runtime},
    ui,
};
use eframe::egui::{self, Align, Align2, FontId, Id, Key, Layout, Rect, RichText, Sense, Stroke};
use std::collections::{HashMap, HashSet};

const COLLECTIONS: [(&str, &str, &str, &str); 6] = [
    (
        "Foundations",
        "Colour, type & spacing",
        "layers",
        "12 references",
    ),
    (
        "Motion studies",
        "Curves with a purpose",
        "wave",
        "8 references",
    ),
    (
        "Navigation",
        "Menus, trees and flows",
        "grid",
        "8 references",
    ),
    (
        "Field notes",
        "Thoughts from the process",
        "document",
        "6 references",
    ),
    (
        "Interface patterns",
        "Buttons, forms and dialogs",
        "cube",
        "16 references",
    ),
    ("Team rituals", "Team processes", "users", "4 references"),
];
type MenuGroup = (
    &'static str,
    &'static str,
    [(&'static str, &'static str, &'static str); 2],
);
const MENU: [MenuGroup; 3] = [
    (
        "Library",
        "YOUR COLLECTION",
        [
            ("Foundations", "Colour, type and space", "layers"),
            ("Motion studies", "Timing with intention", "wave"),
        ],
    ),
    (
        "Workspace",
        "SHARED SPACE",
        [
            ("Project board", "The work in progress", "grid"),
            ("Team directory", "People behind the work", "users"),
        ],
    ),
    (
        "Resources",
        "GOOD TO HAVE",
        [
            ("Field guide", "Principles for the everyday", "document"),
            ("Release notes", "What changed, and why", "clock"),
        ],
    ),
];
const RECORDS: [(&str, &str, &str); 12] = [
    ("Navigation study", "Exploration", "Today"),
    ("Colour in context", "Reference", "Today"),
    ("Toolbar keyboard shortcuts", "Prototype", "Yesterday"),
    ("Spacing that breathes", "Reference", "Oct 24"),
    ("The first five minutes", "Research", "Oct 23"),
    ("Keyboard paths", "Checklist", "Oct 22"),
    ("A measured spring", "Exploration", "Oct 21"),
    ("Naming the layers", "Reference", "Oct 20"),
    ("The empty canvas", "Prototype", "Oct 19"),
    ("Small confirmations", "Exploration", "Oct 18"),
    ("An honest loading state", "Prototype", "Oct 17"),
    ("A better way back", "Research", "Oct 16"),
];
const DIRECTIONS: [(&str, &str, &str, &str); 3] = [
    (
        "Editorial",
        "Component library",
        "Tokens, controls and documentation.",
        "TYPE / RHYTHM / SPACE",
    ),
    (
        "Structural",
        "Project dashboard",
        "Track tasks, reviews and releases.",
        "GRID / CONNECTION / ORDER",
    ),
    (
        "Essential",
        "Admin dashboard",
        "Tables, filters and account settings.",
        "FOCUS / CLARITY / INTENT",
    ),
];
const TREE: [(&str, &str, Option<usize>, &str); 8] = [
    ("source", "source", None, ""),
    ("components", "components", Some(0), ""),
    (
        "navigation-file",
        "navigation.js",
        Some(1),
        "8 working specimens · JavaScript",
    ),
    (
        "button-file",
        "button.js",
        Some(1),
        "Actions & feedback · JavaScript",
    ),
    (
        "tokens-file",
        "tokens.css",
        Some(0),
        "Semantic design tokens · CSS",
    ),
    ("notes", "notes", None, ""),
    (
        "readme-file",
        "README.md",
        Some(5),
        "A place to begin · Markdown",
    ),
    (
        "decisions-file",
        "decisions.md",
        Some(5),
        "The why behind the work · Markdown",
    ),
];
const FOLDERS: [(&str, &str, Option<usize>, &[usize]); 8] = [
    ("workspace", "Workspace", None, &[1, 6]),
    ("projects", "Projects", Some(0), &[2, 3]),
    ("fieldwork", "Fieldwork", Some(1), &[4, 5]),
    ("atlas", "Atlas", Some(1), &[]),
    ("research", "Research", Some(2), &[]),
    ("prototype", "Prototype", Some(2), &[]),
    ("reference", "Reference", Some(0), &[7]),
    ("principles", "Principles", Some(6), &[]),
];

#[derive(Clone, Copy, Debug)]
struct Layer {
    from: [f32; 2],
    to: [f32; 2],
    start: f64,
    delay: f32,
    duration: f32,
    curve: Curve,
    scale: f32,
}
impl Layer {
    fn sample(self, now: f64, reduced: bool) -> [f32; 2] {
        let p = if reduced || self.scale == 0. {
            1.
        } else {
            (((now - self.start) as f32 - self.delay * self.scale) / (self.duration * self.scale))
                .clamp(0., 1.)
        };
        let p = motion::ease(self.curve, p);
        [
            self.from[0] + (self.to[0] - self.from[0]) * p,
            self.from[1] + (self.to[1] - self.from[1]) * p,
        ]
    }
    fn moving(self, now: f64) -> bool {
        now < self.start + ((self.delay + self.duration) * self.scale) as f64
            && self.from != self.to
    }
}
#[derive(Clone, Debug)]
struct Stack {
    active: usize,
    layers: Vec<Layer>,
}
impl Stack {
    fn new(count: usize) -> Self {
        Self {
            active: 0,
            layers: (0..count)
                .map(|i| {
                    let v = if i == 0 { [1., 0.] } else { [0., 20.] };
                    Layer {
                        from: v,
                        to: v,
                        start: 0.,
                        delay: 0.,
                        duration: 0.42,
                        curve: Curve::OutExpo,
                        scale: 1.,
                    }
                })
                .collect(),
        }
    }
    fn switch(&mut self, next: usize, direction: f32, now: f64, scale: f32) {
        if next >= self.layers.len() {
            return;
        }
        let values: Vec<_> = self
            .layers
            .iter()
            .map(|l| l.sample(now, scale == 0.))
            .collect();
        let outgoing = values
            .iter()
            .enumerate()
            .any(|(i, v)| i != next && v[0] > 0.001);
        self.active = next;
        for (i, value) in values.into_iter().enumerate() {
            let active = i == next;
            let visible = value[0] > 0.001;
            self.layers[i] = Layer {
                from: if active && !visible {
                    [0., direction * 20.]
                } else {
                    value
                },
                to: if active {
                    [1., 0.]
                } else {
                    [0., -direction * 16.]
                },
                start: now,
                delay: if active && !visible && outgoing {
                    0.14
                } else {
                    0.
                },
                duration: if active { 0.42 } else { 0.14 },
                curve: if active { Curve::OutExpo } else { Curve::In },
                scale,
            };
        }
    }
    fn settle(&mut self) {
        for (i, l) in self.layers.iter_mut().enumerate() {
            l.from = if i == self.active { [1., 0.] } else { [0., 0.] };
            l.to = l.from;
            l.scale = 0.;
        }
    }
}
#[derive(Clone, Copy)]
enum WizardFocus {
    Name,
    Email,
    Heading,
    Success,
}
pub struct State {
    pub menu: Option<usize>,
    pub combo_open: bool,
    pub query: String,
    pub collection: usize,
    pub combo_active: Option<usize>,
    pub folder: usize,
    pub page: usize,
    pub dock: usize,
    pub wizard: usize,
    pub completed: bool,
    pub name: String,
    pub email: String,
    pub shared: bool,
    pub slide: usize,
    pub tree_selected: usize,
    pub tree_focus: usize,
    pub expanded: HashSet<usize>,
    menu_stack: Stack,
    page_stack: Stack,
    dock_stack: Stack,
    wizard_stack: Stack,
    carousel_stack: Stack,
    statuses: [String; 8],
    name_error: bool,
    email_error: bool,
    wizard_focus: Option<WizardFocus>,
    focus_request: Option<String>,
    ids: HashMap<String, Id>,
    active: bool,
    folder_changed: f64,
    folder_scale: f32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            menu: None,
            combo_open: false,
            query: String::new(),
            collection: 0,
            combo_active: Some(0),
            folder: 0,
            page: 0,
            dock: 0,
            wizard: 0,
            completed: false,
            name: String::new(),
            email: String::new(),
            shared: false,
            slide: 0,
            tree_selected: 0,
            tree_focus: 0,
            expanded: HashSet::from([0]),
            menu_stack: Stack::new(3),
            page_stack: Stack::new(4),
            dock_stack: Stack::new(3),
            wizard_stack: Stack::new(3),
            carousel_stack: Stack::new(3),
            statuses: [
                "Choose Library, Workspace or Resources.".into(),
                "Foundations selected. Search six collections.".into(),
                "Workspace. Two folders to explore.".into(),
                "Page 1 of 4. Notes 1 to 3 of 12.".into(),
                "source is expanded. Two items inside.".into(),
                "Board view. Two open tasks and one completed task.".into(),
                "Step 1 of 3. Give your workspace a name.".into(),
                "Slide 1 of 3. Component library.".into(),
            ],
            name_error: false,
            email_error: false,
            wizard_focus: None,
            focus_request: None,
            ids: HashMap::new(),
            active: false,
            folder_changed: -10.,
            folder_scale: 1.,
        }
    }
}
impl State {
    pub fn collection_matches(&self) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        COLLECTIONS
            .iter()
            .enumerate()
            .filter(|(_, c)| format!("{} {}", c.0, c.1).to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect()
    }
    pub fn filter_collections(&mut self) {
        let count = self.collection_matches().len();
        self.combo_open = true;
        self.combo_active = if count > 0 { Some(0) } else { None };
        self.statuses[1] = if count == 0 {
            "No collections found. Try another search.".into()
        } else {
            format!(
                "{count} {} found{}.",
                if count == 1 {
                    "collection"
                } else {
                    "collections"
                },
                if self.query.trim().is_empty() {
                    String::new()
                } else {
                    format!(" for “{}”", self.query.trim())
                }
            )
        };
    }
    pub fn choose_collection(&mut self, index: usize) {
        if index >= 6 {
            return;
        }
        self.collection = index;
        self.query.clear();
        self.combo_active = Some(index);
        self.combo_open = false;
        self.statuses[1] = format!(
            "{} selected. {}.",
            COLLECTIONS[index].0, COLLECTIONS[index].3
        );
        self.focus_request = Some("nv-combo-input".into());
    }
    pub fn folder_path(&self) -> Vec<usize> {
        let mut path = vec![self.folder];
        let mut parent = FOLDERS[self.folder].2;
        while let Some(p) = parent {
            path.push(p);
            parent = FOLDERS[p].2;
        }
        path.reverse();
        path
    }
    pub fn show_folder(&mut self, index: usize, now: f64) {
        if index >= 8 {
            return;
        }
        self.folder = index;
        self.folder_changed = now;
        let path = self
            .folder_path()
            .iter()
            .map(|i| FOLDERS[*i].1)
            .collect::<Vec<_>>()
            .join(" / ");
        let children = FOLDERS[index].3;
        self.statuses[2] = format!(
            "{path}. {}",
            if children.is_empty() {
                "This folder is ready for your work.".into()
            } else {
                format!("{} folders to explore.", children.len())
            }
        );
    }
    pub fn show_page(&mut self, next: usize, now: f64, scale: f32) {
        if next >= 4 || next == self.page {
            return;
        }
        let direction = if next > self.page { 1. } else { -1. };
        self.page = next;
        self.page_stack.switch(next, direction, now, scale);
        self.statuses[3] = format!(
            "Page {} of 4. Notes {} to {} of 12.",
            next + 1,
            next * 3 + 1,
            next * 3 + 3
        );
    }
    pub fn visible_tree(&self) -> Vec<usize> {
        (0..TREE.len())
            .filter(|i| {
                let mut parent = TREE[*i].2;
                while let Some(p) = parent {
                    if !self.expanded.contains(&p) {
                        return false;
                    }
                    parent = TREE[p].2;
                }
                true
            })
            .collect()
    }
    fn tree_children(index: usize) -> Vec<usize> {
        (0..TREE.len())
            .filter(|i| TREE[*i].2 == Some(index))
            .collect()
    }
    pub fn expand_tree(&mut self, index: usize, open: bool) {
        if Self::tree_children(index).is_empty() {
            return;
        }
        if open {
            self.expanded.insert(index);
        } else {
            self.expanded.remove(&index);
            let mut p = TREE[self.tree_focus].2;
            while let Some(parent) = p {
                if parent == index {
                    self.tree_focus = index;
                    self.focus_request = Some(format!("nv-node-{index}"));
                    break;
                }
                p = TREE[parent].2;
            }
        }
        self.statuses[4] = format!(
            "{} {}.",
            TREE[index].1,
            if open { "expanded" } else { "collapsed" }
        );
    }
    pub fn select_tree(&mut self, index: usize) {
        self.tree_selected = index;
        let mut path = vec![TREE[index].1];
        let mut parent = TREE[index].2;
        while let Some(p) = parent {
            path.push(TREE[p].1);
            parent = TREE[p].2;
        }
        path.reverse();
        self.statuses[4] = format!(
            "{} selected. {}",
            path.join(" / "),
            if TREE[index].3.is_empty() {
                format!("{} items in this folder.", Self::tree_children(index).len())
            } else {
                TREE[index].3.into()
            }
        );
    }
    pub fn tree_key(&mut self, key: Key) {
        let list = self.visible_tree();
        let index = self.tree_focus;
        let pos = list.iter().position(|i| *i == index).unwrap_or(0);
        let next = match key {
            Key::ArrowDown => list.get((pos + 1).min(list.len() - 1)).copied(),
            Key::ArrowUp => list.get(pos.saturating_sub(1)).copied(),
            Key::Home => list.first().copied(),
            Key::End => list.last().copied(),
            Key::ArrowRight => {
                if !Self::tree_children(index).is_empty() {
                    if !self.expanded.contains(&index) {
                        self.expand_tree(index, true);
                        None
                    } else {
                        list.get(pos + 1).copied()
                    }
                } else {
                    None
                }
            }
            Key::ArrowLeft => {
                if self.expanded.contains(&index) {
                    self.expand_tree(index, false);
                    None
                } else {
                    TREE[index].2
                }
            }
            Key::Enter | Key::Space => {
                self.select_tree(index);
                if !Self::tree_children(index).is_empty() {
                    self.expand_tree(index, !self.expanded.contains(&index));
                }
                None
            }
            _ => None,
        };
        if let Some(next) = next {
            self.tree_focus = next;
            self.focus_request = Some(format!("nv-node-{next}"));
        }
    }
    pub fn valid_email(value: &str) -> bool {
        let value = value.trim();
        let Some((local, domain)) = value.split_once('@') else {
            return false;
        };
        !local.is_empty()
            && !domain.is_empty()
            && !domain.contains('@')
            && local
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || ".!#$%&'*+-/=?^_`{|}~".contains(c))
            && domain.split('.').all(|s| {
                !s.is_empty()
                    && s.as_bytes().first().is_some_and(u8::is_ascii_alphanumeric)
                    && s.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                    && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            })
    }
    pub fn wizard_next(&mut self, now: f64, scale: f32) -> bool {
        if self.completed {
            self.completed = false;
            self.name.clear();
            self.email.clear();
            self.shared = false;
            self.name_error = false;
            self.email_error = false;
            self.show_wizard(0, now, scale);
            return true;
        }
        if self.wizard == 0 {
            self.name_error = self.name.trim().encode_utf16().count() < 2
                || self.name.encode_utf16().count() > 40;
            if self.name_error {
                self.wizard_focus = Some(WizardFocus::Name);
                self.statuses[6] = "Use at least two characters for the workspace name.".into();
                return false;
            }
        }
        if self.wizard == 1 {
            self.email_error = !Self::valid_email(&self.email);
            if self.email_error {
                self.wizard_focus = Some(WizardFocus::Email);
                self.statuses[6] = "Enter a complete email, such as you@example.com.".into();
                return false;
            }
        }
        if self.wizard < 2 {
            self.show_wizard(self.wizard + 1, now, scale)
        } else {
            self.completed = true;
            self.wizard_focus = Some(WizardFocus::Success);
            self.statuses[6] = format!(
                "{} created as a local sample. No data was sent.",
                self.name.trim()
            );
        }
        true
    }
    pub fn show_wizard(&mut self, next: usize, now: f64, scale: f32) {
        if next > 2 {
            return;
        }
        let direction = if next >= self.wizard { 1. } else { -1. };
        self.wizard = next;
        self.wizard_stack.switch(next, direction, now, scale);
        self.wizard_focus = Some(WizardFocus::Heading);
        self.statuses[6] = match next {
            0 => "Step 1 of 3. Give your workspace a name.".into(),
            1 => "Step 2 of 3. Choose an owner and visibility.".into(),
            _ => format!(
                "Step 3 of 3. Review {} before creating the local sample.",
                if self.name.trim().is_empty() {
                    "your workspace"
                } else {
                    self.name.trim()
                }
            ),
        };
    }
    pub fn show_slide(&mut self, next: i32, direction: Option<f32>, now: f64, scale: f32) {
        let next = next.rem_euclid(3) as usize;
        if next == self.slide {
            return;
        }
        let direction = direction.unwrap_or(if next > self.slide { 1. } else { -1. });
        self.slide = next;
        self.carousel_stack.switch(next, direction, now, scale);
        self.statuses[7] = format!(
            "Direction {} of 3. {}: {}",
            next + 1,
            DIRECTIONS[next].0,
            DIRECTIONS[next].1
        );
    }
    fn register(&mut self, rt: &mut Runtime, name: &str, response: &egui::Response) {
        rt.record(name, response);
        self.ids.insert(name.into(), response.id);
        if response.has_focus()
            && (name.starts_with("nv-menu-")
                || name.starts_with("nv-destination-")
                || name.starts_with("nv-node-")
                || name.starts_with("nv-dock-view-")
                || name.starts_with("nv-slide-")
                || name == "nv-carousel"
                || name == "nv-combo-input")
        {
            rt.context().memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                        ..Default::default()
                    },
                )
            });
        }
        if self.focus_request.as_deref() == Some(name) {
            response.request_focus();
            self.focus_request = None;
        }
    }
    fn status(&self, ui: &mut egui::Ui, rt: &Runtime, index: usize) {
        ui.add_space(12.);
        let response = ui::muted(ui, rt, &self.statuses[index]);
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_live(egui::accesskit::Live::Polite)
        });
    }
    fn open_menu(&mut self, index: usize, now: f64, scale: f32, first: bool) {
        let old = self.menu;
        self.menu = Some(index);
        if let Some(old) = old {
            self.menu_stack
                .switch(index, if old <= index { 1. } else { -1. }, now, scale)
        } else {
            self.menu_stack.active = index;
            self.menu_stack.settle();
        }
        if first {
            self.focus_request = Some(format!("nv-destination-{index}-0"));
        }
        self.statuses[0] = format!("{} open. Two destinations available.", MENU[index].0);
    }
    fn close_menu(&mut self, restore: bool) {
        if let Some(old) = self.menu.take()
            && restore
        {
            self.focus_request = Some(format!("nv-menu-trigger-{old}"));
        }
    }
    fn menu_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let start = ui.cursor().min;
        let mut trigger_rects = vec![];
        ui.horizontal(|ui| {
            for (i, menu) in MENU.iter().enumerate() {
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-menu-trigger-{i}"),
                    menu.0,
                    Some("chevron-down"),
                    self.menu == Some(i),
                    true,
                    0.,
                );
                trigger_rects.push(b.rect);
                self.register(rt, &format!("nv-menu-trigger-{i}"), &b);
                if b.clicked() {
                    if self.menu == Some(i) {
                        self.close_menu(true)
                    } else {
                        self.open_menu(i, rt.now, rt.motion_scale(), false)
                    }
                }
                if b.has_focus() {
                    if let Some(next) = horizontal_key(ui, i, 3) {
                        self.focus_request = Some(format!("nv-menu-trigger-{next}"));
                        if self.menu.is_some() {
                            self.open_menu(next, rt.now, rt.motion_scale(), false)
                        }
                    }
                    if consume(ui, Key::ArrowDown) {
                        self.open_menu(i, rt.now, rt.motion_scale(), true)
                    }
                }
            }
        });
        if let Some(index) = self.menu
            && let Some(target) = trigger_rects.get(index)
        {
            let x = rt.animate(
                "nv-menu-marker-x",
                target.left() - start.x,
                0.42,
                Curve::SpringSmooth,
            );
            let w = rt.animate(
                "nv-menu-marker-w",
                target.width(),
                0.42,
                Curve::SpringSmooth,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(start.x + x, target.bottom() + 2.),
                    egui::pos2(start.x + x + w, target.bottom() + 2.),
                ],
                Stroke::new(2., rt.tokens.accent),
            );
        }
        ui.add_space(18.);
        let space = Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 238.));
        let open = rt.animate(
            "nv-menu-open",
            if self.menu.is_some() { 1. } else { 0. },
            if self.menu.is_some() { 0.32 } else { 0.22 },
            if self.menu.is_some() {
                Curve::OutExpo
            } else {
                Curve::In
            },
        );
        if open < 0.999 {
            let color = rt.tokens.text3.gamma_multiply(1. - open);
            ui.painter().text(
                space.center() - egui::vec2(0., 10.),
                Align2::CENTER_CENTER,
                "Where shall we go?",
                FontId::proportional(18.),
                color,
            );
            ui.painter().text(
                space.center() + egui::vec2(0., 18.),
                Align2::CENTER_CENTER,
                "Choose a destination above.",
                FontId::proportional(12.),
                color,
            );
        }
        if open > 0.001 {
            let mut panel = ui.new_child(
                egui::UiBuilder::new()
                    .id_salt("nv-menu-viewport")
                    .max_rect(space.translate(egui::vec2(0., -6. * (1. - open))))
                    .layout(Layout::top_down(Align::Min)),
            );
            panel.set_opacity(open);
            panel.set_clip_rect(space.intersect(ui.clip_rect()));
            if self.menu.is_none() {
                panel.disable()
            }
            let stack = self.menu_stack.clone();
            stack_ui(
                &mut panel,
                rt,
                "nv-menu-stack",
                &stack,
                230.,
                |ui, rt, index| {
                    ui.label(RichText::new(MENU[index].1).small().color(rt.tokens.text3));
                    ui.heading(MENU[index].0);
                    ui.add_space(8.);
                    for (j, (name, desc, icon)) in MENU[index].2.iter().enumerate() {
                        let b = option_row(
                            ui,
                            rt,
                            &format!("nv-destination-{index}-{j}"),
                            name,
                            desc,
                            icon,
                            false,
                            true,
                        );
                        self.register(rt, &format!("nv-destination-{index}-{j}"), &b);
                        if b.clicked() {
                            self.statuses[0] =
                                format!("{name} selected. Your destination is ready.");
                            self.close_menu(true);
                        }
                    }
                },
            );
        }
        ui.allocate_space(space.size());
        let bounds = Rect::from_min_max(start, space.max);
        if self.menu.is_some() {
            if consume(ui, Key::Escape) {
                self.close_menu(true)
            }
            if ui.input(|i| {
                i.pointer.primary_pressed()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !bounds.contains(p))
            }) {
                self.close_menu(false)
            }
            let focused = ui.memory(|m| m.focused());
            if self.focus_request.is_none()
                && focused.is_some()
                && !self
                    .ids
                    .iter()
                    .filter(|(name, _)| {
                        name.starts_with("nv-menu-") || name.starts_with("nv-destination-")
                    })
                    .any(|(_, id)| Some(*id) == focused)
            {
                self.close_menu(false)
            }
        }
        self.status(ui, rt, 0);
    }
    fn combo_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let start = ui.cursor().min;
        let input_id = ui.make_persistent_id("nv-collection-input");
        let focused = ui.memory(|m| m.has_focus(input_id) || m.had_focus_last_frame(input_id));
        let matches = self.collection_matches();
        if focused {
            for key in [
                Key::ArrowDown,
                Key::ArrowUp,
                Key::Home,
                Key::End,
                Key::Enter,
                Key::Escape,
                Key::Tab,
            ] {
                if (key == Key::Home || key == Key::End) && !self.combo_open {
                    continue;
                }
                let pressed = if key == Key::Tab {
                    ui.input(|i| i.key_pressed(key))
                } else {
                    consume(ui, key)
                };
                if !pressed {
                    continue;
                }
                if key == Key::Escape && self.combo_open {
                    self.combo_open = false;
                    self.focus_request = Some("nv-combo-input".into());
                    self.statuses[1] = format!(
                        "Search closed. {} remains selected.",
                        COLLECTIONS[self.collection].0
                    );
                } else if key == Key::Enter && self.combo_open {
                    if let Some(index) = self.combo_active.and_then(|a| matches.get(a)).copied() {
                        self.choose_collection(index)
                    }
                } else if key == Key::Tab {
                    self.combo_open = false;
                } else {
                    let was_open = self.combo_open;
                    self.combo_open = true;
                    let last = matches.len().saturating_sub(1);
                    self.combo_active = if matches.is_empty() {
                        None
                    } else {
                        Some(match key {
                            Key::Home => 0,
                            Key::End => last,
                            Key::ArrowDown => {
                                if was_open {
                                    (self.combo_active.unwrap_or(0) + 1).min(last)
                                } else {
                                    0
                                }
                            }
                            Key::ArrowUp => {
                                if was_open {
                                    self.combo_active.unwrap_or(0).saturating_sub(1)
                                } else {
                                    last
                                }
                            }
                            _ => self.combo_active.unwrap_or(0),
                        })
                    };
                }
            }
        }
        ui.label(
            RichText::new("Find a collection                       6 available")
                .small()
                .color(rt.tokens.text2),
        );
        let input = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .id(input_id)
                .hint_text("Search collections…")
                .desired_width(f32::INFINITY)
                .margin(egui::vec2(12., 10.)),
        );
        self.register(rt, "nv-combo-input", &input);
        if input.gained_focus() || input.clicked() {
            self.combo_open = true
        }
        if input.changed() {
            self.filter_collections()
        }
        ui.add_space(12.);
        let mut chosen = None;
        let matches = self.collection_matches();
        let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 250.), Sense::hover());
        let mut content = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("nv-collection-list")
                .max_rect(r)
                .layout(Layout::top_down(Align::Min)),
        );
        content.set_clip_rect(r.intersect(ui.clip_rect()));
        if self.combo_open {
            egui::ScrollArea::vertical()
                .id_salt("nv-options-scroll")
                .max_height(246.)
                .show(&mut content, |ui| {
                    if matches.is_empty() {
                        ui.label("No matching collections. Try “motion” or “field”.");
                    }
                    for (position, &index) in matches.iter().enumerate() {
                        let c = COLLECTIONS[index];
                        let highlighted = self.combo_active == Some(position);
                        let b = option_row(
                            ui,
                            rt,
                            &format!("nv-option-{index}"),
                            c.0,
                            c.1,
                            c.2,
                            highlighted,
                            false,
                        );
                        self.register(rt, &format!("nv-option-{index}"), &b);
                        b.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::WidgetType::SelectableLabel,
                                ui.is_enabled(),
                                self.collection == index,
                                format!("{}. {}", c.0, c.1),
                            )
                        });
                        if b.clicked() {
                            chosen = Some(index)
                        }
                        if highlighted && focused {
                            b.scroll_to_me(Some(Align::Center));
                        }
                    }
                });
        } else {
            content.add_space(50.);
            content.vertical_centered(|ui| {
                paint_icon(ui, "layers", rt.tokens.accent_text, 36.);
                ui.label(
                    RichText::new("CURRENT COLLECTION")
                        .small()
                        .color(rt.tokens.text3),
                );
                ui.heading(COLLECTIONS[self.collection].0);
                ui.label(
                    RichText::new(format!(
                        "{} · {}",
                        COLLECTIONS[self.collection].3, COLLECTIONS[self.collection].1
                    ))
                    .small()
                    .color(rt.tokens.text3),
                );
            });
        }
        if let Some(index) = chosen {
            self.choose_collection(index);
            input.request_focus();
            self.focus_request = None;
        }
        // A native option press must keep the combobox editor focused, matching
        // the reference's pointerdown prevention. Otherwise TextEdit dismisses
        // the list on press before an option can receive its release click.
        let pointer_in_list = ui.input(|i| i.pointer.interact_pos().is_some_and(|p| r.contains(p)));
        if self.combo_open && pointer_in_list && ui.input(|i| i.pointer.primary_down()) {
            input.request_focus();
        }
        ui::muted(
            ui,
            rt,
            "Type to filter · Up / Down to explore · Enter to choose",
        );
        let bounds = Rect::from_min_max(start, egui::pos2(r.right(), ui.cursor().min.y));
        if self.combo_open
            && ui.input(|i| {
                i.pointer.primary_pressed()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !bounds.contains(p))
            })
        {
            self.combo_open = false
        }
        if self.combo_open && input.lost_focus() && chosen.is_none() && !pointer_in_list {
            self.combo_open = false
        }
        self.status(ui, rt, 1);
    }
    fn breadcrumbs_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        let mut target = None;
        ui.horizontal_wrapped(|ui| {
            for (i, folder) in self.folder_path().iter().enumerate() {
                if i > 0 {
                    ui.label(RichText::new("›").color(rt.tokens.text3));
                }
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-crumb-{folder}"),
                    FOLDERS[*folder].1,
                    if *folder == 0 { Some("home") } else { None },
                    *folder == self.folder,
                    true,
                    0.,
                );
                self.register(rt, &format!("nv-crumb-{folder}"), &b);
                if b.clicked() {
                    target = Some((*folder, true))
                }
            }
        });
        let progress = rt.animate(
            "nv-path-line",
            self.folder_path().len() as f32 / 4.,
            0.32,
            Curve::Out,
        );
        ui::progress(ui, rt, progress, 2.);
        ui.add_space(18.);
        let p = if rt.reduced() || self.folder_scale <= 0. {
            1.
        } else {
            ((rt.now - self.folder_changed) as f32 / (0.32 * self.folder_scale)).clamp(0., 1.)
        };
        if p < 1. {
            rt.repaint()
        }
        let start = ui.cursor().min;
        let mut content = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("nv-folder-view")
                .max_rect(Rect::from_min_size(
                    start + egui::vec2((1. - motion::ease(Curve::OutExpo, p)) * 12., 0.),
                    egui::vec2(ui.available_width(), 210.),
                ))
                .layout(Layout::top_down(Align::Min)),
        );
        content.set_opacity(motion::ease(Curve::OutExpo, p));
        content.label(
            RichText::new(if FOLDERS[self.folder].3.is_empty() {
                "INSIDE THE FOLDER"
            } else {
                "FOLDERS"
            })
            .small()
            .color(rt.tokens.text3),
        );
        if FOLDERS[self.folder].3.is_empty() {
            content.add_space(40.);
            content.vertical_centered(|ui| {
                paint_icon(ui, "document", rt.tokens.text3, 30.);
                ui.heading("Empty folder");
                ui.label(format!(
                    "{} is ready for your next idea.",
                    FOLDERS[self.folder].1
                ));
            });
        } else {
            for &child in FOLDERS[self.folder].3 {
                let desc = if FOLDERS[child].3.is_empty() {
                    "Ready for your work".into()
                } else {
                    format!("{} folders", FOLDERS[child].3.len())
                };
                let b = option_row(
                    &mut content,
                    rt,
                    &format!("nv-folder-{child}"),
                    FOLDERS[child].1,
                    &desc,
                    "folder",
                    false,
                    true,
                );
                self.register(rt, &format!("nv-folder-{child}"), &b);
                if b.clicked() {
                    target = Some((child, false))
                }
            }
        }
        ui.allocate_space(egui::vec2(ui.available_width(), 210.));
        ui.horizontal(|ui| {
            ui::muted(ui, rt, "Local workspace");
            let b = ui
                .add_enabled_ui(FOLDERS[self.folder].2.is_some(), |ui| {
                    native_button(
                        ui,
                        rt,
                        "nv-folder-up",
                        "Up one level",
                        Some("arrow-left"),
                        false,
                        true,
                        0.,
                    )
                })
                .inner;
            self.register(rt, "nv-folder-up", &b);
            if b.clicked() {
                target = FOLDERS[self.folder].2.map(|i| (i, false));
            }
        });
        if let Some((index, crumb)) = target {
            self.show_folder(index, rt.now);
            self.folder_scale = rt.motion_scale();
            self.focus_request = Some(if crumb {
                format!("nv-crumb-{index}")
            } else if let Some(first) = FOLDERS[index].3.first() {
                format!("nv-folder-{first}")
            } else {
                format!("nv-crumb-{index}")
            });
        }
        self.status(ui, rt, 2);
    }
    fn pagination_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PROJECT NOTES")
                    .small()
                    .color(rt.tokens.text3),
            );
            ui.label(
                RichText::new(format!(
                    "{:02}—{:02} / 12",
                    self.page * 3 + 1,
                    self.page * 3 + 3
                ))
                .small()
                .monospace()
                .color(rt.tokens.text3),
            );
        });
        let stack = self.page_stack.clone();
        stack_ui(ui, rt, "nv-page-stack", &stack, 224., |ui, rt, page| {
            for j in 0..3 {
                let i = page * 3 + j;
                ui.add_space(8.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!("{:02}", i + 1))
                            .monospace()
                            .color(rt.tokens.text3),
                    );
                    ui.vertical(|ui| {
                        ui.label(RichText::new(RECORDS[i].0).strong());
                        ui.label(RichText::new(RECORDS[i].1).small().color(rt.tokens.text3));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new(RECORDS[i].2).small().color(rt.tokens.text3));
                    });
                });
                ui.add_space(12.);
                ui.separator();
            }
        });
        let mut next = None;
        ui.horizontal(|ui| {
            let b = ui
                .add_enabled_ui(self.page > 0, |ui| {
                    native_button(
                        ui,
                        rt,
                        "nv-page-prev",
                        "",
                        Some("arrow-left"),
                        false,
                        true,
                        34.,
                    )
                })
                .inner;
            self.register(rt, "nv-page-prev", &b);
            if b.clicked() {
                next = Some(self.page - 1)
            }
            for i in 0..4 {
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-page-{i}"),
                    &(i + 1).to_string(),
                    None,
                    self.page == i,
                    true,
                    34.,
                );
                self.register(rt, &format!("nv-page-{i}"), &b);
                if b.clicked() {
                    next = Some(i)
                }
            }
            let b = ui
                .add_enabled_ui(self.page < 3, |ui| {
                    native_button(
                        ui,
                        rt,
                        "nv-page-next",
                        "",
                        Some("arrow-right"),
                        false,
                        true,
                        34.,
                    )
                })
                .inner;
            self.register(rt, "nv-page-next", &b);
            if b.clicked() {
                next = Some(self.page + 1)
            }
        });
        if let Some(next) = next {
            self.show_page(next, rt.now, rt.motion_scale())
        }
        self.status(ui, rt, 3);
    }
    fn tree_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.label(
            RichText::new("FIELDWORK / FILES              Up / Down / Left / Right")
                .small()
                .color(rt.tokens.text3),
        );
        ui.add_space(12.);
        for index in self.visible_tree() {
            let mut depth = 0;
            let mut parent = TREE[index].2;
            while let Some(p) = parent {
                depth += 1;
                parent = TREE[p].2;
            }
            let folder = !Self::tree_children(index).is_empty();
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 18.);
                let prefix = if folder {
                    if self.expanded.contains(&index) {
                        "⌄"
                    } else {
                        "›"
                    }
                } else {
                    " "
                };
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-node-{index}"),
                    &format!("{prefix}  {}", TREE[index].1),
                    Some(if folder { "folder" } else { "document" }),
                    self.tree_selected == index,
                    self.tree_focus == index,
                    ui.available_width(),
                );
                self.register(rt, &format!("nv-node-{index}"), &b);
                if b.clicked() {
                    self.tree_focus = index;
                    b.request_focus();
                    self.select_tree(index);
                    if folder {
                        self.expand_tree(index, !self.expanded.contains(&index));
                    }
                }
                if b.has_focus() {
                    self.tree_focus = index;
                    for key in [
                        Key::ArrowDown,
                        Key::ArrowUp,
                        Key::Home,
                        Key::End,
                        Key::ArrowRight,
                        Key::ArrowLeft,
                    ] {
                        if consume(ui, key) {
                            self.tree_key(key)
                        }
                    }
                }
            });
        }
        ui.add_space(14.);
        ui.separator();
        ui.horizontal(|ui| {
            paint_icon(
                ui,
                if Self::tree_children(self.tree_selected).is_empty() {
                    "document"
                } else {
                    "folder"
                },
                rt.tokens.text2,
                22.,
            );
            ui.vertical(|ui| {
                ui.label(RichText::new(TREE[self.tree_selected].1).strong());
                ui.label(
                    RichText::new(if TREE[self.tree_selected].3.is_empty() {
                        format!(
                            "{} items · Folder",
                            Self::tree_children(self.tree_selected).len()
                        )
                    } else {
                        TREE[self.tree_selected].3.into()
                    })
                    .small()
                    .color(rt.tokens.text3),
                );
            });
        });
        self.status(ui, rt, 4);
    }
    fn show_dock(&mut self, next: usize, rt: &Runtime) {
        if next >= 3 {
            return;
        }
        let old = self.dock;
        self.dock = next;
        self.dock_stack.switch(
            next,
            if next >= old { 1. } else { -1. },
            rt.now,
            rt.motion_scale(),
        );
        self.statuses[5] = [
            "Board view. Two open tasks and one completed task.",
            "List view. All three tasks, with their current status.",
            "Activity view. Three updates from this morning.",
        ][next]
            .into();
    }
    fn dock_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("FIELDWORK").small().color(rt.tokens.text3));
            ui::badge(ui, rt, "3 open tasks", "");
        });
        ui.add_space(12.);
        let stack = self.dock_stack.clone();
        stack_ui(
            ui,
            rt,
            "nv-dock-stack",
            &stack,
            240.,
            |ui, rt, index| match index {
                0 => {
                    ui.columns(2, |cols| {
                        cols[0].label(RichText::new("TO DO  2").small().color(rt.tokens.text3));
                        for (title, sub) in [
                            ("Add search tests", "Research · In progress"),
                            ("Update navigation", "Interface · Next"),
                        ] {
                            egui::Frame::new()
                                .fill(rt.tokens.surface2)
                                .corner_radius(6.)
                                .inner_margin(10.)
                                .show(&mut cols[0], |ui| {
                                    ui.label(title);
                                    ui.label(RichText::new(sub).small().color(rt.tokens.text3));
                                });
                            cols[0].add_space(8.);
                        }
                        cols[1].label(RichText::new("DONE  1").small().color(rt.tokens.text3));
                        egui::Frame::new()
                            .fill(rt.tokens.surface2)
                            .corner_radius(6.)
                            .inner_margin(10.)
                            .show(&mut cols[1], |ui| {
                                ui.label(RichText::new("Review design").strikethrough());
                                ui.label(
                                    RichText::new("Planning · Complete")
                                        .small()
                                        .color(rt.tokens.text3),
                                );
                            });
                    });
                }
                1 => {
                    for (icon, title, sub, tag) in [
                        (
                            "check-circle",
                            "Review design",
                            "Planning · Complete",
                            "Done",
                        ),
                        ("clock", "Add search tests", "Research · In progress", "Now"),
                        ("document", "Update navigation", "Interface · Next", "Next"),
                    ] {
                        ui.horizontal(|ui| {
                            paint_icon(ui, icon, rt.tokens.text2, 18.);
                            ui.vertical(|ui| {
                                ui.label(title);
                                ui.label(RichText::new(sub).small().color(rt.tokens.text3));
                            });
                            ui::badge(ui, rt, tag, "");
                        });
                        ui.add_space(18.);
                    }
                }
                _ => {
                    for (time, title, sub) in [
                        ("09:42", "Design approved.", "Planning moved to Done."),
                        ("10:08", "Tests started.", "Search tests are running."),
                        (
                            "10:24",
                            "Navigation task added.",
                            "Navigation update moved to Next.",
                        ),
                    ] {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(time)
                                    .monospace()
                                    .small()
                                    .color(rt.tokens.text3),
                            );
                            ui.vertical(|ui| {
                                ui.label(RichText::new(title).strong());
                                ui.label(RichText::new(sub).small().color(rt.tokens.text3));
                            });
                        });
                        ui.add_space(16.);
                    }
                }
            },
        );
        let mut rects = vec![];
        ui.horizontal(|ui| {
            for (i, (name, icon)) in [("Board", "grid"), ("List", "menu"), ("Activity", "clock")]
                .iter()
                .enumerate()
            {
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-dock-view-{i}"),
                    name,
                    Some(icon),
                    self.dock == i,
                    self.dock == i,
                    0.,
                );
                rects.push(b.rect);
                self.register(rt, &format!("nv-dock-view-{i}"), &b);
                if b.clicked() {
                    self.show_dock(i, rt)
                }
                if b.has_focus()
                    && let Some(next) = horizontal_key(ui, i, 3)
                {
                    self.show_dock(next, rt);
                    self.focus_request = Some(format!("nv-dock-view-{next}"));
                }
            }
        });
        if let Some(target) = rects.get(self.dock) {
            let base = rects[0].left();
            let x = rt.animate(
                "nv-dock-marker",
                target.left() - base,
                0.42,
                Curve::SpringSmooth,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(base + x, target.bottom() + 2.),
                    egui::pos2(base + x + target.width(), target.bottom() + 2.),
                ],
                Stroke::new(2., rt.tokens.accent),
            );
        }
        self.status(ui, rt, 5);
    }
    fn wizard_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            for (i, label) in ["Name", "Owner", "Review"].iter().enumerate() {
                ui.label(
                    RichText::new(format!(
                        "{}  {label}",
                        if i < self.wizard {
                            "✓".into()
                        } else {
                            (i + 1).to_string()
                        }
                    ))
                    .color(if i == self.wizard {
                        rt.tokens.accent_text
                    } else {
                        rt.tokens.text3
                    }),
                );
                if i < 2 {
                    ui.separator();
                }
            }
        });
        ui.add_space(20.);
        let stack = self.wizard_stack.clone();
        let mut submit = false;
        stack_ui(ui, rt, "nv-wizard-stack", &stack, 280., |ui, rt, index| {
            let title = if self.completed && index == 2 {
                "Workspace created"
            } else {
                ["Workspace name", "Workspace owner", "Review workspace"][index]
            };
            let heading = ui.add(
                egui::Label::new(RichText::new(title).size(20.).strong())
                    .sense(Sense::focusable_noninteractive()),
            );
            self.register(rt, &format!("nv-wizard-heading-{index}"), &heading);
            if index == self.wizard && matches!(self.wizard_focus, Some(WizardFocus::Heading))
                || index == 2
                    && self.completed
                    && matches!(self.wizard_focus, Some(WizardFocus::Success))
            {
                heading.request_focus();
                self.wizard_focus = None;
            }
            ui.add_space(8.);
            match index {
                0 => {
                    ui::muted(ui, rt, "Enter a name to continue.");
                    ui.add_space(14.);
                    ui.label(
                        RichText::new("Workspace name")
                            .small()
                            .color(rt.tokens.text2),
                    );
                    let field = ui.add(
                        egui::TextEdit::singleline(&mut self.name)
                            .id_source("nv-workspace-name")
                            .char_limit(40)
                            .hint_text("e.g. Fieldwork")
                            .desired_width(f32::INFINITY)
                            .margin(egui::vec2(12., 9.)),
                    );
                    self.register(rt, "nv-workspace-name", &field);
                    if matches!(self.wizard_focus, Some(WizardFocus::Name)) {
                        field.request_focus();
                        self.wizard_focus = None;
                    }
                    if field.changed() {
                        let mut units = 0;
                        let end = self.name.char_indices().find_map(|(offset, c)| {
                            units += c.len_utf16();
                            (units > 40).then_some(offset)
                        });
                        if let Some(end) = end {
                            self.name.truncate(end);
                        }
                        self.name_error = false;
                    }
                    if self.name_error {
                        ui.label(
                            RichText::new("Use at least two characters for the workspace name.")
                                .small()
                                .color(rt.tokens.danger_text),
                        );
                    }
                    submit |= (field.has_focus() || field.lost_focus())
                        && ui.input(|i| i.key_pressed(Key::Enter));
                }
                1 => {
                    ui::muted(ui, rt, "A local sample. No invitations are sent.");
                    ui.add_space(14.);
                    let field = ui::field(
                        ui,
                        rt,
                        "nv-workspace-email",
                        "Owner's email",
                        &mut self.email,
                        "you@example.com",
                    );
                    self.register(rt, "nv-workspace-email", &field);
                    if matches!(self.wizard_focus, Some(WizardFocus::Email)) {
                        field.request_focus();
                        self.wizard_focus = None;
                    }
                    if field.changed() {
                        self.email_error = false;
                    }
                    if self.email_error {
                        ui.label(
                            RichText::new("Enter a complete email, such as you@example.com.")
                                .small()
                                .color(rt.tokens.danger_text),
                        );
                    }
                    ui.add_space(14.);
                    ui.label(RichText::new("Visibility").small().color(rt.tokens.text2));
                    egui::ComboBox::from_id_salt("nv-workspace-access")
                        .selected_text(if self.shared {
                            "Shared · ready for a team"
                        } else {
                            "Private · just the owner"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.shared,
                                false,
                                "Private · just the owner",
                            );
                            ui.selectable_value(
                                &mut self.shared,
                                true,
                                "Shared · ready for a team",
                            );
                        });
                    submit |= (field.has_focus() || field.lost_focus())
                        && ui.input(|i| i.key_pressed(Key::Enter));
                }
                _ => {
                    if self.completed {
                        ui.add_space(18.);
                        paint_icon(ui, "check", rt.tokens.success, 36.);
                        ui.label(format!(
                            "{} is ready for {}.",
                            self.name.trim(),
                            self.email.trim()
                        ));
                        ui.add_space(16.);
                        ui::badge(ui, rt, "Local sample created", "accent");
                    } else {
                        ui::muted(ui, rt, "Review your choices before creating the sample.");
                        ui.add_space(16.);
                        review_fact(ui, rt, "Workspace", self.name.trim());
                        review_fact(ui, rt, "Owner", self.email.trim());
                        review_fact(
                            ui,
                            rt,
                            "Visibility",
                            if self.shared { "Shared" } else { "Private" },
                        );
                    }
                }
            }
        });
        ui.horizontal(|ui| {
            let b = ui
                .add_enabled_ui(self.wizard > 0 && !self.completed, |ui| {
                    native_button(
                        ui,
                        rt,
                        "nv-wizard-back",
                        "Back",
                        Some("arrow-left"),
                        false,
                        true,
                        84.,
                    )
                })
                .inner;
            self.register(rt, "nv-wizard-back", &b);
            if b.clicked() {
                self.show_wizard(self.wizard - 1, rt.now, rt.motion_scale())
            }
            ui.label(
                RichText::new(format!("0{} / 03", self.wizard + 1))
                    .monospace()
                    .small()
                    .color(rt.tokens.text3),
            );
            let b = native_button(
                ui,
                rt,
                "nv-wizard-next",
                if self.completed {
                    "Start again"
                } else if self.wizard == 2 {
                    "Create"
                } else {
                    "Next"
                },
                Some("arrow-right"),
                true,
                true,
                130.,
            );
            self.register(rt, "nv-wizard-next", &b);
            submit |= b.clicked();
        });
        if submit {
            self.wizard_next(rt.now, rt.motion_scale());
            rt.repaint();
        }
        self.status(ui, rt, 6);
    }
    fn carousel_ui(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("LAYOUTS").small().color(rt.tokens.text3));
            ui.label(
                RichText::new(format!("0{} / 03", self.slide + 1))
                    .monospace()
                    .small()
                    .color(rt.tokens.text3),
            );
        });
        let stack = self.carousel_stack.clone();
        let bounds = stack_ui(
            ui,
            rt,
            "nv-carousel-stack",
            &stack,
            306.,
            |ui, rt, index| {
                direction_art(ui, rt, index);
                ui.add_space(16.);
                ui.label(
                    RichText::new(DIRECTIONS[index].0)
                        .small()
                        .color(rt.tokens.accent_text),
                );
                ui.heading(DIRECTIONS[index].1);
                ui.label(
                    RichText::new(DIRECTIONS[index].2)
                        .small()
                        .color(rt.tokens.text3),
                );
            },
        );
        let region = ui.interact(
            bounds,
            ui.make_persistent_id("nv-carousel-region"),
            Sense::click(),
        );
        if region.clicked() {
            region.request_focus();
        }
        self.register(rt, "nv-carousel", &region);
        ui.horizontal(|ui| {
            let b = native_button(
                ui,
                rt,
                "nv-slide-prev",
                "",
                Some("arrow-left"),
                false,
                true,
                34.,
            );
            self.register(rt, "nv-slide-prev", &b);
            if b.clicked() {
                self.show_slide(self.slide as i32 - 1, Some(-1.), rt.now, rt.motion_scale())
            }
            for i in 0..3 {
                let b = native_button(
                    ui,
                    rt,
                    &format!("nv-slide-{i}"),
                    if self.slide == i { "●" } else { "○" },
                    None,
                    self.slide == i,
                    true,
                    28.,
                );
                self.register(rt, &format!("nv-slide-{i}"), &b);
                if b.clicked() {
                    self.show_slide(i as i32, None, rt.now, rt.motion_scale())
                }
            }
            let b = native_button(
                ui,
                rt,
                "nv-slide-next",
                "",
                Some("arrow-right"),
                false,
                true,
                34.,
            );
            self.register(rt, "nv-slide-next", &b);
            if b.clicked() {
                self.show_slide(self.slide as i32 + 1, Some(1.), rt.now, rt.motion_scale())
            }
        });
        let b = native_button(
            ui,
            rt,
            "nv-use-direction",
            "Use this direction",
            Some("arrow-right"),
            false,
            true,
            0.,
        );
        self.register(rt, "nv-use-direction", &b);
        if b.clicked() {
            self.statuses[7] = format!(
                "{} selected for your project. {}",
                DIRECTIONS[self.slide].0, DIRECTIONS[self.slide].2
            );
        }
        let focused = ui.memory(|m| m.focused());
        if self
            .ids
            .iter()
            .filter(|(name, _)| {
                name.starts_with("nv-slide-")
                    || *name == "nv-carousel"
                    || *name == "nv-use-direction"
            })
            .any(|(_, id)| Some(*id) == focused)
        {
            for key in [Key::ArrowLeft, Key::ArrowRight, Key::Home, Key::End] {
                if consume(ui, key) {
                    let next = if key == Key::Home {
                        0
                    } else if key == Key::End {
                        2
                    } else {
                        self.slide as i32 + if key == Key::ArrowRight { 1 } else { -1 }
                    };
                    self.show_slide(
                        next,
                        Some(if key == Key::ArrowLeft || key == Key::Home {
                            -1.
                        } else {
                            1.
                        }),
                        rt.now,
                        rt.motion_scale(),
                    )
                }
            }
        }
        self.status(ui, rt, 7);
    }
    fn render_demo(&mut self, index: usize, ui: &mut egui::Ui, rt: &mut Runtime) {
        if rt.reduced() {
            for stack in [
                &mut self.menu_stack,
                &mut self.page_stack,
                &mut self.dock_stack,
                &mut self.wizard_stack,
                &mut self.carousel_stack,
            ] {
                stack.settle();
            }
        }
        let (id, title, desc, height) = match index {
            0 => (
                "nv-menu",
                "Travelling navigation menu",
                "Switch destinations inside one continuous viewport. Escape returns to the trigger; a click outside closes it.",
                390.,
            ),
            1 => (
                "nv-combo",
                "Searchable collection picker",
                "Filter real collections. Arrow keys explore matches; Enter chooses. Your input keeps keyboard focus.",
                390.,
            ),
            2 => (
                "nv-breadcrumb",
                "Living breadcrumb trail",
                "Drill into a real folder hierarchy. Every ancestor remains a working way back.",
                390.,
            ),
            3 => (
                "nv-pagination",
                "Directional pagination",
                "Twelve project notes, three at a time. The frame stays steady while each page travels in the direction you chose.",
                390.,
            ),
            4 => (
                "nv-tree",
                "Keyboard file tree",
                "Open nested folders and select a file. Use arrows to traverse, Right to expand and Left to return to its parent.",
                420.,
            ),
            5 => (
                "nv-dock",
                "Workspace action dock",
                "Board, list and activity show the same project from different angles.",
                420.,
            ),
            6 => (
                "nv-wizard",
                "Three-step workspace wizard",
                "Name a workspace, choose its owner and review the details. Back preserves your work.",
                450.,
            ),
            _ => (
                "nv-carousel",
                "Carousel",
                "Use arrows or dots to change slides.",
                450.,
            ),
        };
        let response = ui::demo(ui, rt, id, title, desc, height, |ui, rt| match index {
            0 => self.menu_ui(ui, rt),
            1 => self.combo_ui(ui, rt),
            2 => self.breadcrumbs_ui(ui, rt),
            3 => self.pagination_ui(ui, rt),
            4 => self.tree_ui(ui, rt),
            5 => self.dock_ui(ui, rt),
            6 => self.wizard_ui(ui, rt),
            _ => self.carousel_ui(ui, rt),
        });
        rt.anchor(id, title, response.response.rect);
    }
}
impl Page for State {
    fn enter(&mut self, _: &mut Runtime) {
        self.active = true;
    }
    fn leave(&mut self, _: &mut Runtime) {
        self.active = false;
        self.menu = None;
        self.combo_open = false;
        self.focus_request = None;
        self.wizard_focus = None;
        for stack in [
            &mut self.menu_stack,
            &mut self.page_stack,
            &mut self.dock_stack,
            &mut self.wizard_stack,
            &mut self.carousel_stack,
        ] {
            stack.settle();
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime) {
        ui::page_head(
            ui,
            rt,
            "layers",
            "Build / Paths & places",
            "Navigation",
            "Menus, search, trees and multi-step flows.",
        );
        for (i, (id, title)) in [
            ("nv-menus", "Menus and search"),
            ("nv-wayfinding", "Breadcrumbs and pagination"),
            ("nv-structure", "Tree and dock"),
            ("nv-flows", "Wizard and carousel"),
        ]
        .iter()
        .enumerate()
        {
            ui::section(ui, rt, id, &format!("{:02}", i + 1), title, "");
            if ui.available_width() >= 740. {
                ui.columns(2, |cols| {
                    self.render_demo(i * 2, &mut cols[0], rt);
                    self.render_demo(i * 2 + 1, &mut cols[1], rt);
                });
            } else {
                self.render_demo(i * 2, ui, rt);
                ui.add_space(16.);
                self.render_demo(i * 2 + 1, ui, rt);
            }
            ui.add_space(24.);
        }
        if let Some(name) = self.focus_request.clone()
            && let Some(id) = self.ids.get(&name)
        {
            ui.memory_mut(|m| m.request_focus(*id));
            self.focus_request = None;
            rt.repaint();
        }
    }
}
fn consume(ui: &mut egui::Ui, key: Key) -> bool {
    let consumed = ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key));
    if consumed {
        ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    consumed
}
fn horizontal_key(ui: &mut egui::Ui, index: usize, len: usize) -> Option<usize> {
    for key in [Key::Home, Key::End, Key::ArrowRight, Key::ArrowLeft] {
        if consume(ui, key) {
            return Some(match key {
                Key::Home => 0,
                Key::End => len - 1,
                Key::ArrowRight => (index + 1) % len,
                _ => (index + len - 1) % len,
            });
        }
    }
    None
}
#[allow(clippy::too_many_arguments)]
fn native_button(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    label: &str,
    icon: Option<&str>,
    selected: bool,
    focusable: bool,
    width: f32,
) -> egui::Response {
    let id = ui.make_persistent_id(id);
    let font = FontId::proportional(13.);
    let text = ui
        .painter()
        .layout_no_wrap(label.into(), font.clone(), rt.tokens.text);
    let width = if width > 0. {
        width
    } else {
        text.size().x + 24. + if icon.is_some() { 24. } else { 0. }
    }
    .min(ui.available_width().max(28.));
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, 34.), Sense::hover());
    let sense = if focusable {
        Sense::click()
    } else {
        Sense::click().difference(Sense::FOCUSABLE)
    };
    let response = ui.interact(r, id, sense);
    if response.clicked() {
        response.request_focus();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Button, ui.is_enabled(), selected, label)
    });
    let hover = rt.animate(
        (id, "hover"),
        if response.hovered() || response.has_focus() {
            1.
        } else {
            0.
        },
        0.14,
        Curve::Out,
    );
    let color = if selected {
        rt.tokens.accent_text
    } else {
        rt.tokens.text2
    };
    ui.painter().rect_filled(
        r,
        ui::rounded(6. * rt.tokens.radius),
        if selected {
            rt.tokens.accent_soft
        } else {
            rt.tokens.surface2.gamma_multiply(hover)
        },
    );
    let mut x = r.left() + 12.;
    if let Some(icon) = icon {
        icons::paint(
            ui.painter(),
            icon,
            Rect::from_center_size(egui::pos2(x + 7., r.center().y), egui::vec2(15., 15.)),
            color,
            1.6,
            rt.icon_progress((id, "icon"), response.hovered() || response.has_focus()),
        );
        x += 24.;
    }
    ui.painter().galley(
        egui::pos2(x, r.center().y - text.size().y / 2.),
        text,
        color,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            r.expand(2.),
            ui::rounded(7. * rt.tokens.radius),
            Stroke::new(2., rt.tokens.accent_ring),
            egui::StrokeKind::Outside,
        );
    }
    response
}
#[allow(clippy::too_many_arguments)]
fn option_row(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    title: &str,
    desc: &str,
    icon: &str,
    selected: bool,
    focusable: bool,
) -> egui::Response {
    let id = ui.make_persistent_id(id);
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 62.), Sense::hover());
    let response = ui.interact(
        r,
        id,
        if focusable {
            Sense::click()
        } else {
            Sense::click().difference(Sense::FOCUSABLE)
        },
    );
    if focusable && response.clicked() {
        response.request_focus();
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            format!("{title}. {desc}"),
        )
    });
    let hover = rt.animate(
        (id, "hover"),
        if response.hovered() || response.has_focus() {
            1.
        } else {
            0.
        },
        0.14,
        Curve::Out,
    );
    ui.painter().rect_filled(
        r,
        ui::rounded(6. * rt.tokens.radius),
        if selected {
            rt.tokens.accent_soft
        } else {
            rt.tokens.surface2.gamma_multiply(hover)
        },
    );
    icons::paint(
        ui.painter(),
        icon,
        Rect::from_center_size(
            egui::pos2(r.left() + 20., r.center().y),
            egui::vec2(20., 20.),
        ),
        rt.tokens.text2,
        1.6,
        0.,
    );
    ui.painter().text(
        egui::pos2(r.left() + 42., r.top() + 17.),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(13.),
        rt.tokens.text,
    );
    ui.painter().text(
        egui::pos2(r.left() + 42., r.top() + 39.),
        Align2::LEFT_CENTER,
        desc,
        FontId::proportional(11.),
        rt.tokens.text3,
    );
    if response.has_focus() {
        ui.painter().rect_stroke(
            r,
            ui::rounded(6. * rt.tokens.radius),
            Stroke::new(2., rt.tokens.accent_ring),
            egui::StrokeKind::Inside,
        );
    }
    response
}
fn stack_ui(
    ui: &mut egui::Ui,
    rt: &mut Runtime,
    id: &str,
    stack: &Stack,
    height: f32,
    mut body: impl FnMut(&mut egui::Ui, &mut Runtime, usize),
) -> Rect {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), height), Sense::hover());
    for (i, layer) in stack.layers.iter().enumerate() {
        let [alpha, x] = layer.sample(rt.now, rt.reduced());
        if alpha <= 0.001 && i != stack.active {
            continue;
        }
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, i))
                .max_rect(r.translate(egui::vec2(x, 0.)))
                .layout(Layout::top_down(Align::Min)),
        );
        child.set_clip_rect(r.intersect(ui.clip_rect()));
        child.set_opacity(alpha);
        if i != stack.active {
            child.disable()
        }
        body(&mut child, rt, i);
        if layer.moving(rt.now) && !rt.reduced() {
            rt.repaint()
        }
    }
    r
}
fn paint_icon(ui: &mut egui::Ui, name: &str, color: egui::Color32, size: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    icons::paint(ui.painter(), name, r, color, 1.6, 0.);
}
fn review_fact(ui: &mut egui::Ui, rt: &Runtime, name: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(name).small().color(rt.tokens.text3));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).color(rt.tokens.text));
        });
    });
    ui.add_space(12.);
}
fn direction_art(ui: &mut egui::Ui, rt: &Runtime, index: usize) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 168.), Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, ui::rounded(8. * rt.tokens.radius), rt.tokens.surface2);
    p.text(
        r.min + egui::vec2(16., 16.),
        Align2::LEFT_TOP,
        format!("0{}", index + 1),
        FontId::monospace(11.),
        rt.tokens.text3,
    );
    match index {
        0 => {
            p.text(
                r.min + egui::vec2(28., 46.),
                Align2::LEFT_TOP,
                "Design\nsystem",
                FontId::proportional(30.),
                rt.tokens.text,
            );
            for y in [r.top() + 70., r.top() + 88., r.top() + 106.] {
                p.line_segment(
                    [
                        egui::pos2(r.center().x + 28., y),
                        egui::pos2(r.right() - 24., y),
                    ],
                    Stroke::new(2., rt.tokens.border_strong),
                );
            }
        }
        1 => {
            let points = [
                r.center() + egui::vec2(-70., -30.),
                r.center() + egui::vec2(70., -30.),
                r.center() + egui::vec2(-70., 30.),
                r.center() + egui::vec2(70., 30.),
            ];
            for pair in [[0, 1], [0, 2], [1, 3], [2, 3]] {
                p.line_segment(
                    [points[pair[0]], points[pair[1]]],
                    Stroke::new(1., rt.tokens.border_strong),
                );
            }
            for point in points {
                p.rect(
                    Rect::from_center_size(point, egui::vec2(50., 34.)),
                    6.,
                    rt.tokens.surface,
                    Stroke::new(1., rt.tokens.accent_ring),
                    egui::StrokeKind::Inside,
                );
            }
        }
        _ => {
            for i in 0..3 {
                let rect = Rect::from_min_size(
                    r.min + egui::vec2(40., 42. + i as f32 * 26.),
                    egui::vec2(r.width() - 80., 18.),
                );
                p.rect(
                    rect,
                    4.,
                    if i == 1 {
                        rt.tokens.accent_soft
                    } else {
                        rt.tokens.surface
                    },
                    Stroke::new(
                        1.,
                        if i == 1 {
                            rt.tokens.accent_ring
                        } else {
                            rt.tokens.border
                        },
                    ),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }
    p.text(
        r.left_bottom() + egui::vec2(16., -12.),
        Align2::LEFT_BOTTOM,
        DIRECTIONS[index].3,
        FontId::monospace(9.),
        rt.tokens.text3,
    );
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    fn frame(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut State,
        demo: usize,
        time: f64,
        events: Vec<egui::Event>,
    ) {
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 900.),
                )),
                time: Some(time),
                events,
                ..Default::default()
            },
            |ctx| {
                rt.begin_frame();
                egui::CentralPanel::default().show(ctx, |ui| {
                    s.render_demo(demo, ui, rt);
                    if let Some(name) = s.focus_request.take()
                        && let Some(id) = s.ids.get(&name)
                    {
                        ui.memory_mut(|m| m.request_focus(*id));
                    }
                });
            },
        );
    }
    fn click(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut State,
        demo: usize,
        time: &mut f64,
        name: &str,
    ) {
        let r = rt
            .records
            .iter()
            .find(|r| r.id == name && r.enabled)
            .unwrap_or_else(|| panic!("missing enabled {name}"));
        let pos = egui::pos2((r.rect[0] + r.rect[2]) / 2., (r.rect[1] + r.rect[3]) / 2.);
        for pressed in [true, false] {
            *time += 0.02;
            frame(
                ctx,
                rt,
                s,
                demo,
                *time,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
            );
        }
        *time += 0.02;
        frame(ctx, rt, s, demo, *time, vec![]);
    }
    #[test]
    fn native_clicks_operate_all_eight_navigation_specimens() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let mut s = State::default();
        s.enter(&mut rt);
        let mut t = 0.;
        for demo in 0..8 {
            t += 1.;
            frame(&ctx, &mut rt, &mut s, demo, t, vec![]);
            t += 0.02;
            frame(&ctx, &mut rt, &mut s, demo, t, vec![]);
            match demo {
                0 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-menu-trigger-1");
                    assert_eq!(s.menu, Some(1));
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-destination-1-0");
                    assert!(s.menu.is_none());
                    assert!(s.statuses[0].contains("Project board selected"));
                }
                1 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-combo-input");
                    assert!(s.combo_open);
                    s.query = "motion".into();
                    s.filter_collections();
                    t += 0.02;
                    frame(&ctx, &mut rt, &mut s, demo, t, vec![]);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-option-1");
                    assert_eq!(s.collection, 1);
                    assert!(!s.combo_open);
                }
                2 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-folder-1");
                    assert_eq!(s.folder, 1);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-folder-2");
                    assert_eq!(s.folder_path(), vec![0, 1, 2]);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-crumb-0");
                    assert_eq!(s.folder, 0);
                }
                3 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-page-next");
                    assert_eq!(s.page, 1);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-page-3");
                    assert_eq!(s.page, 3);
                    assert!(
                        !rt.records
                            .iter()
                            .find(|r| r.id == "nv-page-next")
                            .unwrap()
                            .enabled
                    );
                }
                4 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-node-1");
                    assert!(s.expanded.contains(&1));
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-node-2");
                    assert_eq!(s.tree_selected, 2);
                }
                5 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-dock-view-2");
                    assert_eq!(s.dock, 2);
                }
                6 => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-wizard-next");
                    assert!(s.name_error);
                    s.name = "Native workspace".into();
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-wizard-next");
                    assert_eq!(s.wizard, 1);
                    s.email = "owner@example.com".into();
                    s.shared = true;
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-wizard-next");
                    assert_eq!(s.wizard, 2);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-wizard-next");
                    assert!(s.completed);
                    assert!(s.statuses[6].contains("No data was sent"));
                }
                _ => {
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-slide-prev");
                    assert_eq!(s.slide, 2);
                    click(&ctx, &mut rt, &mut s, demo, &mut t, "nv-use-direction");
                    assert!(s.statuses[7].contains("Essential selected for your project"));
                }
            }
        }
        s.leave(&mut rt);
        assert!(s.menu.is_none() && !s.combo_open);
        assert_eq!(s.collection, 1);
        assert!(s.completed && s.shared);
        assert_eq!(s.name, "Native workspace");
        s.enter(&mut rt);
        assert_eq!(s.slide, 2);
    }
    fn key(
        ctx: &egui::Context,
        rt: &mut Runtime,
        s: &mut State,
        demo: usize,
        t: &mut f64,
        key: Key,
    ) {
        for pressed in [true, false] {
            *t += 0.02;
            frame(
                ctx,
                rt,
                s,
                demo,
                *t,
                vec![egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
            );
        }
    }
    #[test]
    fn native_keyboard_menu_combo_tree_dock_wizard_and_carousel() {
        let ctx = egui::Context::default();
        let mut rt = Runtime::new(ctx.clone(), Default::default());
        rt.set_motion(crate::runtime::MotionPreference::Reduced);
        let mut s = State::default();
        s.enter(&mut rt);
        let mut t = 0.;
        frame(&ctx, &mut rt, &mut s, 0, t, vec![]);
        t += 0.02;
        frame(&ctx, &mut rt, &mut s, 0, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["nv-menu-trigger-0"]));
        key(&ctx, &mut rt, &mut s, 0, &mut t, Key::ArrowDown);
        assert_eq!(s.menu, Some(0));
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(s.ids["nv-destination-0-0"]),
            "destination ids {:?}",
            s.ids
        );
        key(&ctx, &mut rt, &mut s, 0, &mut t, Key::Escape);
        assert!(s.menu.is_none());
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(s.ids["nv-menu-trigger-0"])
        );
        key(&ctx, &mut rt, &mut s, 0, &mut t, Key::End);
        key(&ctx, &mut rt, &mut s, 0, &mut t, Key::Enter);
        assert_eq!(s.menu, Some(2));
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 1, t, vec![]);
        click(&ctx, &mut rt, &mut s, 1, &mut t, "nv-combo-input");
        t += 0.02;
        frame(
            &ctx,
            &mut rt,
            &mut s,
            1,
            t,
            vec![egui::Event::Text("motion".into())],
        );
        assert_eq!(s.collection_matches(), vec![1]);
        key(&ctx, &mut rt, &mut s, 1, &mut t, Key::Enter);
        assert_eq!(s.collection, 1);
        assert!(!s.combo_open);
        assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["nv-combo-input"]));
        t += 0.02;
        frame(
            &ctx,
            &mut rt,
            &mut s,
            1,
            t,
            vec![egui::Event::Text("no-match-448".into())],
        );
        assert!(s.combo_open && s.combo_active.is_none());
        key(&ctx, &mut rt, &mut s, 1, &mut t, Key::Escape);
        assert!(!s.combo_open);
        assert_eq!(s.collection, 1);
        assert_eq!(ctx.memory(|m| m.focused()), Some(s.ids["nv-combo-input"]));
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 4, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["nv-node-0"]));
        for _ in 0..3 {
            key(&ctx, &mut rt, &mut s, 4, &mut t, Key::ArrowRight);
        }
        key(&ctx, &mut rt, &mut s, 4, &mut t, Key::Enter);
        assert_eq!(s.tree_selected, 2);
        assert_eq!(s.tree_focus, 2);
        key(&ctx, &mut rt, &mut s, 4, &mut t, Key::ArrowLeft);
        key(&ctx, &mut rt, &mut s, 4, &mut t, Key::ArrowLeft);
        assert_eq!(s.tree_focus, 1);
        assert!(!s.expanded.contains(&1));
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 5, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["nv-dock-view-0"]));
        key(&ctx, &mut rt, &mut s, 5, &mut t, Key::ArrowRight);
        assert_eq!(s.dock, 1);
        key(&ctx, &mut rt, &mut s, 5, &mut t, Key::End);
        assert_eq!(s.dock, 2);
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 6, t, vec![]);
        click(&ctx, &mut rt, &mut s, 6, &mut t, "nv-workspace-name");
        t += 0.02;
        frame(
            &ctx,
            &mut rt,
            &mut s,
            6,
            t,
            vec![egui::Event::Text("Keyboard workspace".into())],
        );
        key(&ctx, &mut rt, &mut s, 6, &mut t, Key::Enter);
        assert_eq!(s.wizard, 1);
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(s.ids["nv-wizard-heading-1"])
        );
        t += 1.;
        frame(&ctx, &mut rt, &mut s, 7, t, vec![]);
        ctx.memory_mut(|m| m.request_focus(s.ids["nv-carousel"]));
        key(&ctx, &mut rt, &mut s, 7, &mut t, Key::End);
        assert_eq!(s.slide, 2);
        key(&ctx, &mut rt, &mut s, 7, &mut t, Key::ArrowRight);
        assert_eq!(s.slide, 0);
        key(&ctx, &mut rt, &mut s, 7, &mut t, Key::ArrowLeft);
        assert_eq!(s.slide, 2);
    }
    #[test]
    fn native_navigation_renders_four_appearances_at_mobile_and_desktop_widths() {
        for width in [320., 390., 1440.] {
            for dark in [false, true] {
                for mono in [false, true] {
                    let ctx = egui::Context::default();
                    let mut prefs = crate::runtime::Preferences::default();
                    prefs.dark = dark;
                    prefs.mono = mono;
                    let mut rt = Runtime::new(ctx.clone(), prefs);
                    let mut s = State::default();
                    s.enter(&mut rt);
                    let out = ctx.run(
                        egui::RawInput {
                            screen_rect: Some(Rect::from_min_size(
                                egui::Pos2::ZERO,
                                egui::vec2(width, 1000.),
                            )),
                            time: Some(1.),
                            ..Default::default()
                        },
                        |ctx| {
                            rt.begin_frame();
                            egui::CentralPanel::default().show(ctx, |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| s.show(ui, &mut rt));
                            });
                        },
                    );
                    assert!(!out.shapes.is_empty());
                    for r in rt
                        .records
                        .iter()
                        .filter(|r| r.enabled && r.id.starts_with("nv-"))
                    {
                        assert!(
                            r.rect[0] >= -1. && r.rect[2] <= width + 1.,
                            "{} escapes {width}: {:?}",
                            r.id,
                            r.rect
                        );
                    }
                    assert_eq!(
                        rt.anchors
                            .iter()
                            .filter(|a| a.id.starts_with("nv-"))
                            .count(),
                        12
                    );
                }
            }
        }
    }
    #[test]
    fn collection_filter_select_and_no_matches() {
        let mut s = State::default();
        s.query = "motion".into();
        s.filter_collections();
        assert_eq!(s.collection_matches(), vec![1]);
        s.choose_collection(1);
        assert_eq!(s.collection, 1);
        assert!(!s.combo_open);
        assert!(s.query.is_empty());
        s.query = "no-match-448".into();
        s.filter_collections();
        assert_eq!(s.combo_active, None);
        assert_eq!(s.collection, 1);
    }
    #[test]
    fn folder_tree_and_keyboard_paths() {
        let mut s = State::default();
        s.show_folder(4, 0.);
        assert_eq!(s.folder_path(), vec![0, 1, 2, 4]);
        s.tree_key(Key::ArrowRight);
        assert_eq!(s.tree_focus, 1);
        s.tree_key(Key::ArrowRight);
        assert!(s.expanded.contains(&1));
        s.tree_key(Key::ArrowRight);
        assert_eq!(s.tree_focus, 2);
        s.tree_key(Key::Enter);
        assert_eq!(s.tree_selected, 2);
        s.tree_key(Key::ArrowLeft);
        assert_eq!(s.tree_focus, 1);
        s.tree_key(Key::ArrowLeft);
        assert!(!s.expanded.contains(&1));
        s.tree_key(Key::End);
        assert_eq!(s.tree_focus, 5);
    }
    #[test]
    fn wizard_validates_preserves_then_resets() {
        let mut s = State::default();
        assert!(!s.wizard_next(0., 1.));
        assert!(s.name_error);
        s.name = "Fieldwork".into();
        assert!(s.wizard_next(0., 1.));
        assert_eq!(s.wizard, 1);
        s.email = "invalid".into();
        assert!(!s.wizard_next(0., 1.));
        s.email = "owner@example.com".into();
        s.shared = true;
        assert!(s.wizard_next(0., 1.));
        s.show_wizard(1, 0., 1.);
        assert_eq!(s.email, "owner@example.com");
        s.wizard_next(0., 1.);
        s.wizard_next(0., 1.);
        assert!(s.completed);
        assert!(s.statuses[6].contains("No data was sent"));
        s.wizard_next(0., 1.);
        assert_eq!(s.wizard, 0);
        assert!(s.name.is_empty() && s.email.is_empty());
        assert!(!s.shared && !s.completed);
    }
    #[test]
    fn email_matches_native_validation_contract() {
        for value in ["a@b", "you@example.com", "a+b@sub.example.com"] {
            assert!(State::valid_email(value), "{value}")
        }
        for value in [
            "",
            "a",
            "@example.com",
            "a@@b",
            "a@-example.com",
            "a@b.",
            "a b@c",
        ] {
            assert!(!State::valid_email(value), "{value}")
        }
    }
    #[test]
    fn panel_reversal_has_no_discontinuity() {
        let mut s = Stack::new(4);
        s.switch(3, 1., 0., 1.);
        let before: Vec<_> = s.layers.iter().map(|l| l.sample(0.25, false)).collect();
        s.switch(0, -1., 0.25, 1.);
        for (i, value) in before
            .iter()
            .enumerate()
            .filter(|(_, value)| value[0] > 0.001)
        {
            assert_eq!(*value, s.layers[i].sample(0.25, false));
        }
        assert_eq!(s.layers[0].sample(1., false), [1., 0.]);
    }
    #[test]
    fn carousel_wrap_and_reduced_motion() {
        let mut s = State::default();
        s.show_slide(-1, Some(-1.), 0., 0.);
        assert_eq!(s.slide, 2);
        assert_eq!(s.carousel_stack.layers[2].sample(0., true), [1., 0.]);
        s.show_slide(3, Some(1.), 0., 1.);
        assert_eq!(s.slide, 0);
    }
}
