# Native Rust implementation contract

Confirmed target: native Rust. No browser, WebView, HTML/CSS rendering,
JavaScript implementation, DOM, wasm-bindgen or embedded JS engine. All
presentation, layout, state, event handling, vector drawing and animation
scheduling are implemented in Rust with eframe/egui 0.33.3.

The existing website remains the behavioral and visual reference. It has 69
specimens across 11 routes. Read the relevant original source completely. Port
all capabilities, not only a small subset exercised by the browser suite.

The reference HTML/CSS/SVG are inspection inputs, not native application
dependencies. The native binary runs independently of the website.

## Page module contract

Each page exports `pub struct State` with `impl Default`, and implements:

```rust
impl crate::runtime::Page for State {
    fn show(&mut self, ui: &mut egui::Ui, rt: &mut crate::runtime::Runtime) { /* ... */ }
    fn enter(&mut self, rt: &mut crate::runtime::Runtime) { /* optional */ }
    fn leave(&mut self, rt: &mut crate::runtime::Runtime) { /* optional */ }
}
```

Use `eframe::egui` (or `use eframe::egui as egui`). Pages remain alive and retain
their state between routes. Leave cancels drags, recurring work and pending demo
operations; async operation durations use `rt.now` in real seconds, independently
of motion scale. Request repaint only while visible finite work needs frames.
Implement pure state methods that tests can exercise without a window.

## Shared API

`crate::runtime`:

- `pub trait Page { fn show(...); fn enter(...) {} fn leave(...) {} }`
- `pub struct Runtime` fields: `pub now:f64`, `pub tokens:crate::tokens::Tokens`,
  `pub prefs:Preferences`, `pub route:Route`, `pub anchors:Vec<Anchor>`
- Preferences: `pub dark:bool`, `pub mono:bool`, `pub hue:f32`,
  `pub radius:f32` (0/sharp, 1/default, 1.6/round), `pub motion:MotionPreference`
- `MotionPreference::{Full,Slow,Reduced,System}`
- `Route::{Overview,Foundations,Motion,Icons,Components,Primitives,Navigation,Forms,Data,Overlays,Workspace}`;
  `.id()->&'static str`, `.title()->&'static str`, `.icon()->&'static str`
- `rt.navigate(Route)`, `rt.jump(Route, specimen_id:&str)` (queued until frame end)
- `rt.set_theme(bool)`, `rt.set_palette(bool)`, `rt.set_hue(f32)`, `rt.set_motion(MotionPreference)`
- `rt.toast(title:&str,desc:&str,tone:&str)`
- `rt.copy(ctx:&egui::Context,text:impl Into<String>)->bool` reports the native clipboard result.
- `rt.download(name:&str,contents:&str,mime:&str)->Result<Option<std::path::PathBuf>,String>`
- `rt.motion_scale()->f32`, `rt.reduced()->bool`
- `rt.animate(id:impl std::hash::Hash,target:f32,seconds:f32,curve:crate::motion::Curve)->f32`
  Retargets from displayed value, requests repaint via stored context.
- `rt.progress(start:f64,seconds:f32)->f32` (motion-scaled; reduced returns 1)
- `rt.repaint()`, `rt.repaint_after(seconds:f64)`; `rt.context()->&egui::Context`
- `rt.icon_progress(id:impl Hash,engaged:bool)->f32` (finite semantic verb)
- `rt.anchor(id:&str,title:&str,rect:egui::Rect)` and `rt.record(id:&str,response:&egui::Response)` for QA
- `rt.request_guide()`, `rt.request_commands()`

`crate::tokens::Tokens` (Clone): colors `bg,bg_sunken,surface,surface2,surface3,
surface_raised,border,border_soft,border_strong,text,text2,text3,text4,
accent,accent_hover,accent_press,accent_text,accent_contrast,accent_soft,
accent_soft2,accent_ring,success,success_soft,success_text,warning,warning_soft,
warning_text,danger,danger_soft,danger_text,info,info_soft,info_text,overlay`
are `egui::Color32`; `charts:[egui::Color32;5]`, `radius:f32`.

`crate::motion`: `Curve::{Linear,Out,OutExpo,In,InOut,Overshoot,SpringSmooth,SpringSnappy,SpringBouncy}`;
`ease(Curve,t:f32)->f32`; `crate::easing::Spring::simulate(k:f64,damping:f64,mass:f64)`
returns `.duration_ms:u32`, `.values:Vec<f64>`, `.at(progress:f64)->f64`.

`crate::ui`:

- `ButtonKind::{Primary,Secondary,Ghost,Soft,Danger}`
- `button(ui:&mut Ui,rt:&mut Runtime,id:impl Hash,label:&str,icon:Option<&str>,kind:ButtonKind)->Response`
- `icon_button(ui,rt,id,icon:&str,label:&str)->Response`
- `segmented(ui,rt,id,labels:&[&str],selected:&mut usize)->bool`
- `toggle(ui,rt,id,label:&str,value:&mut bool)->Response`
- `checkbox(ui,rt,id,label:&str,value:&mut bool)->Response`
- `badge(ui:&mut Ui,rt:&Runtime,label:&str,tone:&str)->Response`
- `page_head(ui:&mut Ui,rt:&mut Runtime,icon:&str,eyebrow:&str,title:&str,lead:&str)`
- `section(ui:&mut Ui,rt:&mut Runtime,id:&str,num:&str,title:&str,desc:&str)`
- `card<R>(ui,rt,id,body:impl FnOnce(&mut Ui,&mut Runtime)->R)->InnerResponse<R>`
- `demo<R>(ui,rt,id,title:&str,desc:&str,height:f32,body:impl FnOnce(&mut Ui,&mut Runtime)->R)->InnerResponse<R>`
- `progress(ui:&mut Ui,rt:&Runtime,fraction:f32,height:f32)->Response`
- `field(ui:&mut Ui,rt:&Runtime,id:impl Hash,label:&str,value:&mut String,hint:&str)->Response`
- `muted(ui:&mut Ui,rt:&Runtime,text:&str)->Response`

`crate::icons`: `paint(painter:&egui::Painter,name:&str,rect:egui::Rect,color:Color32,stroke:f32,progress:f32)`;
`names()->&'static [&'static str]`; all 82 icons are Rust vector primitives,
including their individual semantic animation. No HTML/SVG renderer in the app.

`crate::catalog`: `pub const ENTRIES:&[Entry]`; Entry has `id,page,title,desc,collection,family,icon:&'static str`,
`index:usize`. `crate::runtime::Route::from_id(&str)->Route`.

`crate::sortable` supplies pointer and keyboard drag transactions, rollback,
cross-group moves, edge scrolling and interruptible item motion. Primitives and
Overview use the same engine. Keep shared interaction behavior in this module
so the live previews and full specimens remain consistent.

Keep labels literal, preserve all state transitions and failure paths, and do
not add slogans or emojis. Do not claim pixel identity or full parity until
native rendering and interactions have been measured.
