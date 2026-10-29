use crate::{
    motion::{Curve, Tween},
    tokens::Tokens,
};
use eframe::egui::{self, Context, Id, Rect, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash, path::PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Route {
    #[default]
    Overview,
    Foundations,
    Motion,
    Icons,
    Components,
    Primitives,
    Navigation,
    Forms,
    Data,
    Overlays,
    Workspace,
}
impl Route {
    pub const ALL: [Route; 11] = [
        Self::Overview,
        Self::Foundations,
        Self::Motion,
        Self::Icons,
        Self::Components,
        Self::Primitives,
        Self::Navigation,
        Self::Forms,
        Self::Data,
        Self::Overlays,
        Self::Workspace,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Overview => "overview",
            Self::Foundations => "foundations",
            Self::Motion => "motion",
            Self::Icons => "icons",
            Self::Components => "components",
            Self::Primitives => "primitives",
            Self::Navigation => "navigation",
            Self::Forms => "forms",
            Self::Data => "data",
            Self::Overlays => "overlays",
            Self::Workspace => "workspace",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Foundations => "Foundations",
            Self::Motion => "Motion Lab",
            Self::Icons => "Icons",
            Self::Components => "Components",
            Self::Primitives => "Primitives",
            Self::Navigation => "Navigation",
            Self::Forms => "Forms & Inputs",
            Self::Data => "Data & Charts",
            Self::Overlays => "Overlays & Feedback",
            Self::Workspace => "Coding workspace",
        }
    }
    pub fn icon(self) -> &'static str {
        match self {
            Self::Overview => "home",
            Self::Foundations => "droplet",
            Self::Motion => "wave",
            Self::Icons => "sparkles",
            Self::Components => "cube",
            Self::Primitives => "layers",
            Self::Navigation => "layers",
            Self::Forms => "input",
            Self::Data => "chart-bar",
            Self::Overlays => "window",
            Self::Workspace => "code",
        }
    }
    pub fn from_id(id: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|r| r.id() == id)
            .unwrap_or_default()
    }
    pub fn group(self) -> &'static str {
        match self {
            Self::Overview => "Start",
            Self::Foundations | Self::Motion | Self::Icons => "Foundations",
            Self::Workspace => "Examples",
            _ => "Build",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MotionPreference {
    Full,
    Slow,
    Reduced,
    #[default]
    System,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub dark: bool,
    pub mono: bool,
    pub hue: f32,
    pub radius: f32,
    pub motion: MotionPreference,
    pub guide_completed: bool,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            dark: false,
            mono: false,
            hue: 168.0,
            radius: 1.0,
            motion: MotionPreference::System,
            guide_completed: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Anchor {
    pub id: String,
    pub title: String,
    pub rect: Rect,
}
#[derive(Clone, Debug, Serialize)]
pub struct Recorded {
    pub id: String,
    #[serde(skip)]
    pub widget_id: Id,
    pub rect: [f32; 4],
    pub enabled: bool,
    pub focused: bool,
}
#[derive(Clone, Debug)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub desc: String,
    pub tone: String,
    pub created: f64,
    pub duration: f64,
}
pub trait Page {
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut Runtime);
    fn enter(&mut self, _rt: &mut Runtime) {}
    fn leave(&mut self, _rt: &mut Runtime) {}
}
pub struct Runtime {
    pub now: f64,
    pub tokens: Tokens,
    pub prefs: Preferences,
    pub route: Route,
    pub anchors: Vec<Anchor>,
    pub records: Vec<Recorded>,
    pub requested: Option<Route>,
    pub jump_to: Option<String>,
    pub guide_requested: bool,
    pub commands_requested: bool,
    pub toasts: Vec<Toast>,
    pub system_reduced: bool,
    pub mounted: std::collections::HashSet<Route>,
    pub frame_number: u64,
    context: Context,
    animations: HashMap<Id, Tween>,
    icons: HashMap<Id, IconClock>,
    next_toast: u64,
    pub theme_changed_at: f64,
    pub previous_dark: bool,
    pub theme_origin: Option<egui::Pos2>,
    pub desired_theme: Option<bool>,
}
struct IconClock {
    engaged: bool,
    start: Option<f64>,
}
impl Runtime {
    pub fn new(ctx: Context, prefs: Preferences) -> Self {
        Self {
            tokens: Tokens::new(prefs.dark, prefs.mono, prefs.hue, prefs.radius),
            now: 0.0,
            route: Route::Overview,
            prefs,
            anchors: vec![],
            records: vec![],
            requested: None,
            jump_to: None,
            guide_requested: false,
            commands_requested: false,
            toasts: vec![],
            system_reduced: false,
            mounted: Default::default(),
            frame_number: 0,
            context: ctx,
            animations: HashMap::new(),
            icons: HashMap::new(),
            next_toast: 0,
            theme_changed_at: -10.0,
            previous_dark: false,
            theme_origin: None,
            desired_theme: None,
        }
    }
    pub fn begin_frame(&mut self) {
        self.now = self.context.input(|i| i.time);
        self.frame_number += 1;
        self.anchors.clear();
        self.records.clear();
        self.context.data_mut(|data| {
            data.insert_temp(
                Id::new("schematic-live-focus-ids"),
                std::collections::HashSet::<Id>::new(),
            )
        });
        self.tokens = Tokens::new(
            self.prefs.dark,
            self.prefs.mono,
            self.prefs.hue,
            self.prefs.radius,
        );
        self.toasts
            .retain(|t| self.now - t.created < t.duration + 0.3);
    }
    pub fn navigate(&mut self, route: Route) {
        self.requested = Some(route);
    }
    pub fn jump(&mut self, route: Route, specimen_id: &str) {
        self.jump_to = Some(specimen_id.into());
        self.navigate(route);
    }
    pub fn set_theme(&mut self, dark: bool) {
        if self.prefs.dark != dark || self.desired_theme.is_some() {
            self.previous_dark = self.prefs.dark;
            self.desired_theme = Some(dark);
            self.theme_changed_at = self.now;
            self.theme_origin = self.context.pointer_hover_pos();
            if self.reduced() {
                self.prefs.dark = dark;
                self.desired_theme = None;
                self.tokens = Tokens::new(dark, self.prefs.mono, self.prefs.hue, self.prefs.radius);
            }
            self.repaint();
        }
    }
    pub fn set_palette(&mut self, mono: bool) {
        self.prefs.mono = mono;
        self.tokens = Tokens::new(self.prefs.dark, mono, self.prefs.hue, self.prefs.radius);
        self.repaint();
    }
    pub fn set_hue(&mut self, hue: f32) {
        self.prefs.hue = hue.rem_euclid(360.0);
        self.set_palette(false);
    }
    pub fn set_motion(&mut self, preference: MotionPreference) {
        self.prefs.motion = preference;
        if self.reduced() {
            for tween in self.animations.values_mut() {
                tween.from = tween.to;
                tween.duration = 0.0;
            }
            self.icons.clear();
        }
        self.repaint();
    }
    pub fn toast(&mut self, title: &str, desc: &str, tone: &str) {
        self.next_toast += 1;
        self.toasts.push(Toast {
            id: self.next_toast,
            title: title.into(),
            desc: desc.into(),
            tone: tone.into(),
            created: self.now,
            duration: 5.0,
        });
        self.repaint();
    }
    pub fn copy(&self, _ctx: &Context, text: impl Into<String>) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text.into()))
            .is_ok()
    }
    pub fn download(
        &mut self,
        name: &str,
        contents: &str,
        _mime: &str,
    ) -> Result<Option<PathBuf>, String> {
        if let Some(path) = rfd::FileDialog::new().set_file_name(name).save_file() {
            std::fs::write(&path, contents).map_err(|e| e.to_string())?;
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }
    pub fn reduced(&self) -> bool {
        self.prefs.motion == MotionPreference::Reduced
            || (self.prefs.motion == MotionPreference::System && self.system_reduced)
    }
    pub fn motion_scale(&self) -> f32 {
        if self.reduced() {
            0.0
        } else if self.prefs.motion == MotionPreference::Slow {
            4.0
        } else {
            1.0
        }
    }
    pub fn animate(&mut self, id: impl Hash, target: f32, seconds: f32, curve: Curve) -> f32 {
        let scale = self.motion_scale();
        let id = Id::new(id);
        let tween = self
            .animations
            .entry(id)
            .or_insert_with(|| Tween::new(target, self.now));
        tween.retarget(target, self.now, seconds * scale, curve);
        if scale == 0.0 {
            tween.from = target;
            tween.to = target;
            tween.duration = 0.0;
        }
        let moving = tween.moving(self.now);
        let v = tween.value(self.now);
        if moving {
            self.context.request_repaint();
        }
        v
    }
    pub fn progress(&self, start: f64, seconds: f32) -> f32 {
        if self.reduced() || seconds <= 0.0 {
            return 1.0;
        }
        let p =
            ((self.now - start) / (seconds * self.motion_scale()) as f64).clamp(0.0, 1.0) as f32;
        if p < 1.0 {
            self.repaint();
        }
        p
    }
    pub fn repaint(&self) {
        self.context.request_repaint();
    }
    pub fn repaint_after(&self, seconds: f64) {
        self.context
            .request_repaint_after(std::time::Duration::from_secs_f64(seconds.max(0.0)));
    }
    pub fn context(&self) -> &Context {
        &self.context
    }
    pub fn icon_progress(&mut self, id: impl Hash, engaged: bool) -> f32 {
        if self.reduced() {
            return 0.0;
        }
        let now = self.now;
        let scale = self.motion_scale();
        let clock = self.icons.entry(Id::new(id)).or_insert(IconClock {
            engaged: false,
            start: None,
        });
        if engaged && !clock.engaged && clock.start.is_none() {
            clock.start = Some(now);
        }
        clock.engaged = engaged;
        let Some(start) = clock.start else { return 0.0 };
        let progress = ((now - start) / (1.2 * scale) as f64) as f32;
        if progress >= 1.0 {
            clock.start = None;
            return 0.0;
        }
        self.context.request_repaint();
        progress.max(0.00001)
    }
    pub fn anchor(&mut self, id: &str, title: &str, rect: Rect) {
        self.anchors.push(Anchor {
            id: id.into(),
            title: title.into(),
            rect,
        });
    }
    pub fn record(&mut self, id: &str, response: &Response) {
        self.context.data_mut(|data| {
            let ids = data.get_temp_mut_or_default::<std::collections::HashSet<Id>>(Id::new(
                "schematic-live-focus-ids",
            ));
            if response.enabled() {
                ids.insert(response.id);
            }
        });
        self.records.push(Recorded {
            id: id.into(),
            widget_id: response.id,
            rect: [
                response.rect.min.x,
                response.rect.min.y,
                response.rect.max.x,
                response.rect.max.y,
            ],
            enabled: response.enabled(),
            focused: response.has_focus(),
        });
    }
    pub fn request_guide(&mut self) {
        self.guide_requested = true;
    }
    pub fn request_commands(&mut self) {
        self.commands_requested = true;
    }
    pub fn clear_page_motion(&mut self) {
        self.animations.clear();
        self.icons.clear();
    }
}
/// Restore only to a control drawn in this frame; stale IDs crash native
/// accessibility backends and are not meaningful keyboard destinations.
pub fn restore_focus(ctx: &Context, target: Option<Id>) {
    let valid = target.filter(|id| {
        ctx.data(|data| {
            data.get_temp::<std::collections::HashSet<Id>>(Id::new("schematic-live-focus-ids"))
        })
        .is_some_and(|ids| ids.contains(id))
    });
    ctx.memory_mut(|memory| {
        if let Some(current) = memory.focused() {
            memory.surrender_focus(current);
        }
        if let Some(id) = valid {
            memory.request_focus(id);
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn routes_have_stable_identifiers() {
        for route in Route::ALL {
            assert_eq!(Route::from_id(route.id()), route);
        }
    }
    #[test]
    fn reduced_motion_does_not_change_real_time() {
        let mut rt = Runtime::new(Context::default(), Preferences::default());
        rt.now = 10.0;
        rt.set_motion(MotionPreference::Reduced);
        assert_eq!(rt.motion_scale(), 0.0);
        assert_eq!(rt.now, 10.0);
        assert_eq!(rt.progress(10.0, 0.5), 1.0);
    }
}
