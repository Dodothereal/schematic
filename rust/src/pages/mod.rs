pub mod components;
pub mod data;
pub mod forms;
pub mod foundations;
pub mod icons_page;
pub mod motion_lab;
pub mod navigation;
pub mod overlays;
pub mod overview;
pub mod primitives;
pub mod workspace;
use crate::runtime::{Page, Route};
pub fn create() -> std::collections::HashMap<Route, Box<dyn Page>> {
    let mut pages: std::collections::HashMap<Route, Box<dyn Page>> =
        std::collections::HashMap::new();
    pages.insert(Route::Overview, Box::new(overview::State::default()));
    pages.insert(Route::Foundations, Box::new(foundations::State::default()));
    pages.insert(Route::Motion, Box::new(motion_lab::State::default()));
    pages.insert(Route::Icons, Box::new(icons_page::State::default()));
    pages.insert(Route::Components, Box::new(components::State::default()));
    pages.insert(Route::Primitives, Box::new(primitives::State::default()));
    pages.insert(Route::Navigation, Box::new(navigation::State::default()));
    pages.insert(Route::Forms, Box::new(forms::State::default()));
    pages.insert(Route::Data, Box::new(data::State::default()));
    pages.insert(Route::Overlays, Box::new(overlays::State::default()));
    pages.insert(Route::Workspace, Box::new(workspace::State::default()));
    pages
}
