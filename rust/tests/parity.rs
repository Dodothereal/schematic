use eframe::egui::*;
use schematic_native::{
    app::SchematicApp,
    catalog,
    runtime::{MotionPreference, Preferences, Route},
};

fn frame(
    app: &mut SchematicApp,
    ctx: &Context,
    width: f32,
    height: f32,
    time: f64,
    events: Vec<Event>,
) -> FullOutput {
    ctx.enable_accesskit();
    let mut input = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(width, height))),
        time: Some(time),
        events,
        focused: true,
        ..Default::default()
    };
    input.viewports.entry(ViewportId::ROOT).or_default().focused = Some(true);
    let output = ctx.run(input, |ctx| app.render(ctx));
    if let Some(update) = &output.platform_output.accesskit_update {
        assert!(
            update.nodes.iter().any(|(id, _)| *id == update.focus),
            "native focus {:?} is absent from the accessibility tree (route {:?}, time {time}); commands={} guide={}",
            update.focus,
            app.rt.route,
            Id::new("commands").value(),
            Id::new("guide-launch").value()
        );
    }
    output
}

#[test]
fn all_routes_render_natively_across_appearances_and_window_widths() {
    for (width, height) in [(1440.0, 1000.0), (390.0, 844.0), (320.0, 568.0)] {
        for dark in [false, true] {
            for mono in [false, true] {
                let ctx = Context::default();
                let prefs = Preferences {
                    dark,
                    mono,
                    motion: MotionPreference::Reduced,
                    ..Default::default()
                };
                let mut app = SchematicApp::with_context(ctx.clone(), prefs);
                for (index, route) in Route::ALL.into_iter().enumerate() {
                    app.rt.navigate(route);
                    let _ = frame(&mut app, &ctx, width, height, index as f64 * 2.0, vec![]);
                    let output = frame(
                        &mut app,
                        &ctx,
                        width,
                        height,
                        index as f64 * 2.0 + 0.5,
                        vec![],
                    );
                    assert_eq!(app.rt.route, route);
                    assert!(!output.shapes.is_empty(), "empty native route {route:?}");
                    let primitives = ctx.tessellate(output.shapes, output.pixels_per_point);
                    assert!(
                        !primitives.is_empty(),
                        "no native render primitives: {route:?}"
                    );
                    for widget in app.rt.records.iter().filter(|w| {
                        w.id.starts_with("route:")
                            || w.id.starts_with("icon:")
                            || w.id == "button:Guide"
                            || w.id == "button:Commands"
                    }) {
                        assert!(
                            widget.rect.iter().all(|v| v.is_finite()),
                            "invalid widget geometry: {widget:?}"
                        );
                        if widget.rect[1] >= 0.0 && widget.rect[3] <= height {
                            assert!(
                                widget.rect[0] >= -1.0 && widget.rect[2] <= width + 1.0,
                                "shell overflow at {width}px {route:?}: {widget:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn catalog_identifies_every_native_collection_and_real_specimen() {
    assert_eq!(catalog::ENTRIES.len(), 69);
    let mut ids = std::collections::HashSet::new();
    for entry in catalog::ENTRIES {
        assert!(ids.insert(entry.id));
        assert_eq!(Route::from_id(entry.page).id(), entry.page);
        assert!(!entry.title.is_empty());
        assert!(schematic_native::icons::names().contains(&entry.icon));
    }
}

#[test]
fn native_sidebar_pointer_input_navigates_and_retains_a_single_active_route() {
    let ctx = Context::default();
    let mut app = SchematicApp::with_context(ctx.clone(), Preferences::default());
    let _ = frame(&mut app, &ctx, 1440.0, 1000.0, 0.0, vec![]);
    for (index, route) in [Route::Forms, Route::Data, Route::Workspace]
        .into_iter()
        .enumerate()
    {
        let widget = app
            .rt
            .records
            .iter()
            .find(|w| w.id == format!("route:{}", route.id()))
            .unwrap();
        let pos = pos2(
            (widget.rect[0] + widget.rect[2]) / 2.0,
            (widget.rect[1] + widget.rect[3]) / 2.0,
        );
        let time = 1.0 + index as f64;
        let _ = frame(
            &mut app,
            &ctx,
            1440.0,
            1000.0,
            time,
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        let _ = frame(
            &mut app,
            &ctx,
            1440.0,
            1000.0,
            time + 0.05,
            vec![Event::PointerButton {
                pos,
                button: PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        let _ = frame(&mut app, &ctx, 1440.0, 1000.0, time + 0.1, vec![]);
        assert_eq!(app.rt.route, route);
    }
    assert_eq!(app.rt.mounted.len(), 4);
}

#[test]
fn native_command_search_and_enter_navigate_without_losing_the_application() {
    let ctx = Context::default();
    let mut app = SchematicApp::with_context(ctx.clone(), Preferences::default());
    let _ = frame(&mut app, &ctx, 1440.0, 1000.0, 0.0, vec![]);
    app.rt.request_commands();
    let _ = frame(&mut app, &ctx, 1440.0, 1000.0, 0.1, vec![]);
    let _ = frame(
        &mut app,
        &ctx,
        1440.0,
        1000.0,
        0.2,
        vec![Event::Text("workspace".into())],
    );
    let _ = frame(&mut app, &ctx, 1440.0, 1000.0, 0.3, vec![]);
    let _ = frame(
        &mut app,
        &ctx,
        1440.0,
        1000.0,
        0.4,
        vec![Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    let _ = frame(
        &mut app,
        &ctx,
        1440.0,
        1000.0,
        0.5,
        vec![Event::Key {
            key: Key::Enter,
            physical_key: Some(Key::Enter),
            pressed: false,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
    );
    assert_eq!(app.rt.route, Route::Workspace);
    assert!(app.rt.records.iter().any(|w| w.id.contains("ws-")));
}

#[test]
fn native_guide_keeps_controls_reachable_through_interrupted_steps_and_close() {
    for (width, height) in [(1440.0, 1000.0), (390.0, 844.0), (320.0, 568.0)] {
        let ctx = Context::default();
        let mut app = SchematicApp::with_context(ctx.clone(), Preferences::default());
        frame(&mut app, &ctx, width, height, 0.0, vec![]);
        app.rt.request_guide();
        frame(&mut app, &ctx, width, height, 0.1, vec![]);
        for (index, step) in [0, 1, 2, 1, 3, 2, 3].into_iter().enumerate() {
            app.rt.now = 1.0 + index as f64;
            app.guide.change_step(step, &mut app.rt);
            for fraction in [0.0, 0.07, 0.15, 0.3, 0.8] {
                let output = frame(
                    &mut app,
                    &ctx,
                    width,
                    height,
                    1.0 + index as f64 + fraction,
                    vec![],
                );
                let update = output.platform_output.accesskit_update.as_ref().unwrap();
                assert!(
                    update.nodes.iter().any(|(_, node)| node.label()
                        == Some(if step == 3 { "Finish guide" } else { "Next" })),
                    "guide footer absent from accesskit at {width}: {:?}",
                    update
                        .nodes
                        .iter()
                        .filter_map(|(_, node)| node.label())
                        .collect::<Vec<_>>()
                );
                let nodes: std::collections::HashMap<_, _> =
                    update.nodes.iter().map(|(id, node)| (*id, node)).collect();
                let mut reachable = std::collections::HashSet::new();
                let mut pending = vec![update.tree.as_ref().unwrap().root];
                while let Some(id) = pending.pop() {
                    if reachable.insert(id)
                        && let Some(node) = nodes.get(&id)
                    {
                        pending.extend_from_slice(node.children());
                    }
                }
                let next_id = update
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.label() == Some(if step == 3 { "Finish guide" } else { "Next" })
                    })
                    .unwrap()
                    .0;
                assert!(
                    reachable.contains(&next_id),
                    "Guide Next node exists but is disconnected from accessibility root at {width}px"
                );
                for id in ["guide-next", "guide-back", "guide-close"] {
                    let r = app.rt.records.iter().find(|r| r.id == id).unwrap();
                    assert!(
                        r.rect[0] >= 0.0
                            && r.rect[2] <= width
                            && r.rect[1] >= 0.0
                            && r.rect[3] <= height,
                        "guide overflow: {r:?}"
                    );
                }
            }
        }
        app.guide.close();
        for t in [9.0, 9.2, 10.0] {
            frame(&mut app, &ctx, width, height, t, vec![]);
        }
        assert!(!app.guide.visible());
    }
}

#[test]
fn every_catalog_result_resolves_to_live_native_specimen_geometry() {
    let ctx = Context::default();
    let mut app = SchematicApp::with_context(
        ctx.clone(),
        Preferences {
            motion: MotionPreference::Reduced,
            ..Default::default()
        },
    );
    for (index, entry) in catalog::ENTRIES.iter().enumerate() {
        let route = Route::from_id(entry.page);
        app.rt.jump(route, entry.id);
        frame(&mut app, &ctx, 1440.0, 1000.0, index as f64 * 2.0, vec![]);
        frame(
            &mut app,
            &ctx,
            1440.0,
            1000.0,
            index as f64 * 2.0 + 0.1,
            vec![],
        );
        assert!(
            schematic_native::anchors::resolve(route, entry.id, &app.rt.anchors).is_some(),
            "missing landing: {} {}",
            entry.id,
            entry.title
        );
        assert!(
            app.rt.jump_to.is_none(),
            "unconsumed specimen jump: {}",
            entry.id
        );
    }
}

#[test]
fn history_keyboard_shortcuts_restore_routes_without_adding_history_entries() {
    let ctx = Context::default();
    let mut app = SchematicApp::with_context(ctx.clone(), Preferences::default());
    frame(&mut app, &ctx, 1440.0, 1000.0, 0.0, vec![]);
    for (index, route) in [Route::Forms, Route::Data, Route::Icons]
        .into_iter()
        .enumerate()
    {
        app.rt.navigate(route);
        frame(&mut app, &ctx, 1440.0, 1000.0, index as f64 + 0.1, vec![]);
    }
    for (time, key, expected) in [
        (4.0, Key::OpenBracket, Route::Data),
        (5.0, Key::OpenBracket, Route::Forms),
        (6.0, Key::CloseBracket, Route::Data),
    ] {
        frame(
            &mut app,
            &ctx,
            1440.0,
            1000.0,
            time,
            vec![Event::Key {
                key,
                physical_key: Some(key),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
        );
        frame(
            &mut app,
            &ctx,
            1440.0,
            1000.0,
            time + 0.1,
            vec![Event::Key {
                key,
                physical_key: Some(key),
                pressed: false,
                repeat: false,
                modifiers: Modifiers::COMMAND,
            }],
        );
        assert_eq!(app.rt.route, expected);
    }
}
