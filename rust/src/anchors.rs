//! Resolve reference catalog destinations against anchors from the current
//! native page. This module neither changes routes nor manufactures geometry.
//!
//! Shell integration (after the current page has rendered):
//! `resolve(rt.route, requested_id, &rt.anchors)` returns the actual `Anchor` to
//! scroll to. Clear the pending jump only after resolution succeeds. A missing
//! target returns `None`; it must not be converted into a jump to the page top.
//! `native_id` is available when an explicit catalog-to-native ID is needed.
use crate::{
    catalog::ENTRIES,
    runtime::{Anchor, Route},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecimenAlias {
    pub catalog_id: &'static str,
    pub page: &'static str,
    pub native_id: &'static str,
}
const fn alias(
    catalog_id: &'static str,
    page: &'static str,
    native_id: &'static str,
) -> SpecimenAlias {
    SpecimenAlias {
        catalog_id,
        page,
        native_id,
    }
}

/// Explicit identities, including pages whose native specimen names differ
/// from their reference catalog IDs. `ui::demo` normally registers the catalog
/// ID by title; these aliases also support custom/manual native anchors.
pub const ALIASES: &[SpecimenAlias] = &[
    alias("workspace-0", "workspace", "workspace"),
    alias("primitives-0", "primitives", "pr-accordion"),
    alias("primitives-1", "primitives", "pr-collapsible"),
    alias("primitives-2", "primitives", "pr-pinned-list"),
    alias("primitives-3", "primitives", "pr-sortable-list"),
    alias("primitives-4", "primitives", "pr-text-swap"),
    alias("primitives-5", "primitives", "pr-counter"),
    alias("primitives-6", "primitives", "pr-flip-card"),
    alias("primitives-7", "primitives", "pr-expandable"),
    alias("navigation-0", "navigation", "nv-menu"),
    alias("navigation-1", "navigation", "nv-combo"),
    alias("navigation-2", "navigation", "nv-breadcrumb"),
    alias("navigation-3", "navigation", "nv-pagination"),
    alias("navigation-4", "navigation", "nv-tree"),
    alias("navigation-5", "navigation", "nv-dock"),
    alias("navigation-6", "navigation", "nv-wizard"),
    alias("navigation-7", "navigation", "nv-carousel"),
    alias("components-0", "components", "components-0"),
    alias("components-1", "components", "components-1"),
    alias("components-2", "components", "components-2"),
    alias("components-3", "components", "components-3"),
    alias("components-4", "components", "components-4"),
    alias("components-5", "components", "components-5"),
    alias("components-6", "components", "components-6"),
    alias("components-7", "components", "components-7"),
    alias("components-8", "components", "components-8"),
    alias("components-9", "components", "components-9"),
    alias("components-10", "components", "components-10"),
    alias("components-11", "components", "components-11"),
    alias("components-12", "components", "components-12"),
    alias("forms-0", "forms", "forms-0"),
    alias("forms-1", "forms", "forms-1"),
    alias("forms-2", "forms", "forms-2"),
    alias("forms-3", "forms", "forms-3"),
    alias("forms-4", "forms", "forms-4"),
    alias("forms-5", "forms", "forms-5"),
    alias("forms-6", "forms", "forms-6"),
    alias("forms-7", "forms", "forms-7"),
    alias("forms-8", "forms", "forms-8"),
    alias("forms-9", "forms", "forms-9"),
    alias("forms-10", "forms", "forms-10"),
    alias("data-0", "data", "dt-line"),
    alias("data-1", "data", "dt-bars"),
    alias("data-2", "data", "dt-ranks"),
    alias("data-3", "data", "dt-donut"),
    alias("data-4", "data", "dt-goals"),
    alias("overlays-0", "overlays", "overlays-0"),
    alias("overlays-1", "overlays", "overlays-1"),
    alias("overlays-2", "overlays", "overlays-2"),
    alias("overlays-3", "overlays", "overlays-3"),
    alias("overlays-4", "overlays", "overlays-4"),
    alias("overlays-5", "overlays", "overlays-5"),
    alias("overlays-6", "overlays", "overlays-6"),
    alias("overlays-7", "overlays", "overlays-7"),
    alias("overlays-8", "overlays", "overlays-8"),
    alias("overlays-9", "overlays", "overlays-9"),
    alias("overlays-10", "overlays", "overlays-10"),
    alias("overlays-11", "overlays", "overlays-11"),
    alias("overlays-12", "overlays", "overlays-12"),
    alias("overlays-13", "overlays", "overlays-13"),
    alias("overlays-14", "overlays", "overlays-14"),
    alias("motion-0", "motion", "motion-workbench"),
    alias("motion-1", "motion", "motion-spring"),
    alias("motion-2", "motion", "motion-list"),
    alias("motion-3", "motion", "motion-continuity"),
    alias("icons-0", "icons", "icons-0"),
    alias("icons-1", "icons", "icons-1"),
    alias("foundations-0", "foundations", "fd-spaces"),
    alias("foundations-1", "foundations", "fd-controls"),
];

pub fn native_id(catalog_id: &str) -> Option<&'static str> {
    ALIASES
        .iter()
        .find(|a| a.catalog_id == catalog_id)
        .map(|a| a.native_id)
}

fn usable(anchor: &Anchor) -> bool {
    let r = anchor.rect;
    r.min.x.is_finite()
        && r.min.y.is_finite()
        && r.max.x.is_finite()
        && r.max.y.is_finite()
        && r.width() > 0.
        && r.height() > 0.
}

/// Prefer the exact requested live ID, then its explicit counterpart, then a
/// unique exact catalog title. Known catalog/native IDs from another route are
/// rejected even if stale anchors were accidentally passed. Unmapped native
/// section IDs remain usable when present on the current page.
pub fn resolve<'a>(route: Route, requested_id: &str, anchors: &'a [Anchor]) -> Option<&'a Anchor> {
    let mapping = ALIASES
        .iter()
        .find(|a| a.catalog_id == requested_id || a.native_id == requested_id);
    if mapping.is_some_and(|a| a.page != route.id()) {
        return None;
    }
    let entry = ENTRIES
        .iter()
        .find(|entry| entry.id == mapping.map(|a| a.catalog_id).unwrap_or(requested_id));
    if entry.is_some_and(|entry| entry.page != route.id()) {
        return None;
    }
    let find = |id: &str| anchors.iter().find(|a| a.id == id && usable(a));
    if let Some(anchor) = find(requested_id) {
        return Some(anchor);
    }
    if let Some(mapping) = mapping
        && let Some(anchor) = find(mapping.catalog_id).or_else(|| find(mapping.native_id))
    {
        return Some(anchor);
    }
    let title = entry?.title;
    let mut matches = anchors.iter().filter(|a| a.title == title && usable(a));
    let anchor = matches.next()?;
    matches.next().is_none().then_some(anchor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{self, Rect, pos2, vec2};
    use std::collections::HashSet;

    fn at(id: &str, title: &str, y: f32) -> Anchor {
        Anchor {
            id: id.into(),
            title: title.into(),
            rect: Rect::from_min_size(pos2(24., y), vec2(360., 120.)),
        }
    }

    #[test]
    fn explicit_map_covers_every_reference_specimen_once() {
        assert_eq!(ENTRIES.len(), 69);
        assert_eq!(ALIASES.len(), ENTRIES.len());
        let mut ids = HashSet::new();
        let mut native_ids = HashSet::new();
        for mapping in ALIASES {
            assert!(
                ids.insert(mapping.catalog_id),
                "duplicate {}",
                mapping.catalog_id
            );
            assert!(
                native_ids.insert((mapping.page, mapping.native_id)),
                "duplicate native {}",
                mapping.native_id
            );
            let entry = ENTRIES
                .iter()
                .find(|e| e.id == mapping.catalog_id)
                .expect("alias must name a catalog specimen");
            assert_eq!(entry.page, mapping.page);
            assert_eq!(native_id(entry.id), Some(mapping.native_id));
        }
        for entry in ENTRIES {
            assert!(ids.contains(entry.id), "unmapped {}", entry.id);
        }
        assert_eq!(native_id("missing-0"), None);
    }

    #[test]
    fn all_69_resolve_against_native_only_anchor_names() {
        for (index, mapping) in ALIASES.iter().enumerate() {
            let entry = ENTRIES.iter().find(|e| e.id == mapping.catalog_id).unwrap();
            let expected_y = 200. + index as f32 * 160.;
            let anchors = [
                at("section-top", "Section", 0.),
                at(mapping.native_id, entry.title, expected_y),
            ];
            let route = Route::from_id(mapping.page);
            let target = resolve(route, mapping.catalog_id, &anchors).unwrap_or_else(|| {
                panic!("missing {} -> {}", mapping.catalog_id, mapping.native_id)
            });
            assert!(std::ptr::eq(target, &anchors[1]));
            assert_eq!(target.rect.top(), expected_y);
            assert!(std::ptr::eq(
                resolve(route, mapping.native_id, &anchors).unwrap(),
                target
            ));
        }
        assert_eq!(native_id("workspace-0"), Some("workspace"));
        assert_eq!(native_id("primitives-5"), Some("pr-counter"));
        assert_eq!(native_id("navigation-6"), Some("nv-wizard"));
        assert_eq!(native_id("motion-2"), Some("motion-list"));
        assert_eq!(native_id("data-4"), Some("dt-goals"));
        assert_eq!(native_id("foundations-0"), Some("fd-spaces"));
    }

    #[test]
    fn exact_ids_win_and_title_fallback_is_exact_and_unambiguous() {
        let mut anchors = vec![
            at("pr-counter", "Animated counter", 100.),
            at("primitives-5", "Animated counter", 300.),
        ];
        assert_eq!(
            resolve(Route::Primitives, "primitives-5", &anchors)
                .unwrap()
                .rect
                .top(),
            300.
        );
        assert_eq!(
            resolve(Route::Primitives, "pr-counter", &anchors)
                .unwrap()
                .rect
                .top(),
            100.
        );
        anchors = vec![at("new-native-counter", "Animated counter", 500.)];
        assert_eq!(
            resolve(Route::Primitives, "primitives-5", &anchors)
                .unwrap()
                .rect
                .top(),
            500.
        );
        anchors.push(at("other-counter", "Animated counter", 700.));
        assert!(resolve(Route::Primitives, "primitives-5", &anchors).is_none());
        anchors = vec![at("wrong-title", "Counter", 500.)];
        assert!(resolve(Route::Primitives, "primitives-5", &anchors).is_none());
    }

    #[test]
    fn missing_wrong_route_and_invalid_geometry_never_fall_back_to_top() {
        let mut anchors = vec![
            at("section-top", "Animated counter", 0.),
            at("pr-counter", "Animated counter", 300.),
        ];
        assert!(resolve(Route::Navigation, "primitives-5", &anchors).is_none());
        assert!(resolve(Route::Navigation, "pr-counter", &anchors).is_none());
        assert!(resolve(Route::Primitives, "primitives-99", &anchors).is_none());
        assert!(resolve(Route::Primitives, "primitives-5", &[]).is_none());
        assert_eq!(
            resolve(Route::Primitives, "section-top", &anchors)
                .unwrap()
                .id,
            "section-top"
        );
        anchors.remove(0);
        for rect in [
            Rect::NOTHING,
            Rect::from_min_size(pos2(0., 0.), vec2(0., 10.)),
            Rect::from_min_max(pos2(0., f32::NAN), pos2(10., 10.)),
        ] {
            anchors[0].rect = rect;
            assert!(resolve(Route::Primitives, "primitives-5", &anchors).is_none());
        }
    }

    #[test]
    fn every_catalog_destination_resolves_to_a_real_rendered_native_specimen() {
        use crate::runtime::{MotionPreference, Preferences, Runtime};
        let mut pages = crate::pages::create();
        let mut resolved = HashSet::new();
        for route in Route::ALL {
            let entries: Vec<_> = ENTRIES.iter().filter(|e| e.page == route.id()).collect();
            if entries.is_empty() {
                continue;
            }
            let ctx = egui::Context::default();
            let mut rt = Runtime::new(
                ctx.clone(),
                Preferences {
                    motion: MotionPreference::Reduced,
                    ..Default::default()
                },
            );
            rt.route = route;
            let page = pages.get_mut(&route).unwrap();
            page.enter(&mut rt);
            for frame in 0..2 {
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(1440., 1000.))),
                        time: Some(frame as f64 / 60.),
                        ..Default::default()
                    },
                    |ctx| {
                        rt.begin_frame();
                        egui::CentralPanel::default().show(ctx, |ui| {
                            if route == Route::Workspace {
                                page.show(ui, &mut rt);
                            } else {
                                egui::ScrollArea::vertical().show(ui, |ui| page.show(ui, &mut rt));
                            }
                        });
                    },
                );
            }
            for entry in entries {
                let target = resolve(route, entry.id, &rt.anchors).unwrap_or_else(|| {
                    panic!(
                        "{} has no native destination; available: {:?}",
                        entry.id,
                        rt.anchors
                            .iter()
                            .map(|a| (&a.id, &a.title))
                            .collect::<Vec<_>>()
                    )
                });
                assert_eq!(
                    target.title, entry.title,
                    "{} landed on {}",
                    entry.id, target.id
                );
                assert!(usable(target));
                assert!(resolved.insert(entry.id));
            }
            page.leave(&mut rt);
        }
        assert_eq!(resolved.len(), 69);
    }
}
