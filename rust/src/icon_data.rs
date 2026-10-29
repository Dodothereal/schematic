//! Native vector primitives. Path strings are coordinate data, not SVG markup.
#[derive(Clone, Copy, Debug)]
pub enum Geometry {
    Path(&'static str),
    Circle {
        cx: f32,
        cy: f32,
        r: f32,
    },
    Ellipse {
        cx: f32,
        cy: f32,
        rx: f32,
        ry: f32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        rx: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
}
#[derive(Clone, Copy, Debug)]
pub struct Part {
    pub group: &'static str,
    pub geometry: Geometry,
    pub fill: bool,
    pub stroke: bool,
    pub stroke_width: Option<f32>,
}
#[derive(Clone, Copy, Debug)]
pub struct IconDef {
    pub name: &'static str,
    pub parts: &'static [Part],
}
pub const ICONS: &[IconDef] = &[
    IconDef {
        name: "arrow-right",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M5 12h14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M13 6l6 6-6 6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "arrow-left",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M19 12H5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M11 6l-6 6 6 6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "arrow-up",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 19V5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M6 11l6-6 6 6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "arrow-down",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 5v14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M6 13l6 6 6-6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "arrow-up-right",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M7 17L17 7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M8 7h9v9"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "chevron-right",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M9.5 6l6 6-6 6"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "chevron-down",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M6 9.5l6 6 6-6"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "chevrons-up-down",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M8 9l4-4 4 4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M8 15l4 4 4-4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "download",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 4v11"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M7.5 10.5L12 15l4.5-4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M4 16.5V19a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "upload",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 15V4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M7.5 8.5L12 4l4.5 4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M4 16.5V19a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "refresh",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M20 12a8 8 0 1 1-8-8c2.24 0 4.38.9 5.99 2.43L20 8.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M20 4v4.5h-4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "logout",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M10 4.5H6.5a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2H10"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M10 12h10"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M16 8l4 4-4 4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "external",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M13 5.5H6.5a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V11",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M11 13l9-9"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M14.5 4H20v5.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "check",
        parts: &[Part {
            group: "draw",
            geometry: Geometry::Path("M5 12.5l4.5 4.5L19 7.5"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "x",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M6.5 6.5l11 11"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M17.5 6.5l-11 11"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "plus",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 5v14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M5 12h14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "minus",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M5 12h14"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "search",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Circle {
                    cx: 11.0,
                    cy: 11.0,
                    r: 6.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M20 20l-4.2-4.2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "trash",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M4 7h16"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M9 7V5a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M10 11v6M14 11v6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "copy",
        parts: &[
            Part {
                group: "b",
                geometry: Geometry::Path("M16 8V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v8a2 2 0 0 0 2 2h2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Rect {
                    x: 8.0,
                    y: 8.0,
                    width: 12.0,
                    height: 12.0,
                    rx: 2.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "pencil",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M15.5 4.5l4 4L8 20H4v-4z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M13.5 6.5l4 4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "send",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M21 3L10 14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M21 3l-6.5 18-4.5-7-7-4.5z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "link",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M10 13.5a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M14 10.5a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "filter",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M4 5h16l-6 7.5V19l-4 1.5v-8z"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "sliders",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M4 7h16M4 17h16"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "k1",
                geometry: Geometry::Circle {
                    cx: 8.0,
                    cy: 7.0,
                    r: 2.2,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "k2",
                geometry: Geometry::Circle {
                    cx: 16.0,
                    cy: 17.0,
                    r: 2.2,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "play",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path(
                "M7.5 4.8v14.4a.8.8 0 0 0 1.2.7l11.3-7.2a.8.8 0 0 0 0-1.4L8.7 4.1a.8.8 0 0 0-1.2.7z",
            ),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "pause",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M8 5v14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M16 5v14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "stop",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Rect {
                x: 6.0,
                y: 6.0,
                width: 12.0,
                height: 12.0,
                rx: 2.0,
            },
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "pin",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M9 3.5h6l-1 6 4 4H6l4-4z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M12 13.5v7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "grip",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Circle {
                    cx: 9.0,
                    cy: 5.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Circle {
                    cx: 9.0,
                    cy: 12.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Circle {
                    cx: 9.0,
                    cy: 19.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 15.0,
                    cy: 5.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 15.0,
                    cy: 12.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 15.0,
                    cy: 19.0,
                    r: 0.9,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "more",
        parts: &[
            Part {
                group: "d1",
                geometry: Geometry::Circle {
                    cx: 5.5,
                    cy: 12.0,
                    r: 1.3,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d2",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 1.3,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d3",
                geometry: Geometry::Circle {
                    cx: 18.5,
                    cy: 12.0,
                    r: 1.3,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "menu",
        parts: &[
            Part {
                group: "l1",
                geometry: Geometry::Path("M4 7h16"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "l2",
                geometry: Geometry::Path("M4 12h16"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "l3",
                geometry: Geometry::Path("M4 17h10"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "command",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path(
                "M15 6v12a3 3 0 1 0 3-3H6a3 3 0 1 0 3 3V6a3 3 0 1 0-3 3h12a3 3 0 1 0-3-3",
            ),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "cursor",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M6 5l5 14 2-6 6-2z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M6 1.5V3M2.5 5H4M3.4 2.4l1 1"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "bell",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M6 10a6 6 0 0 1 12 0c0 4 1.5 6 2 7H4c.5-1 2-3 2-7z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M10 20a2 2 0 0 0 4 0"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "home",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M3.5 11L12 4l8.5 7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M5.5 9.5V19a1 1 0 0 0 1 1H10v-5h4v5h3.5a1 1 0 0 0 1-1V9.5",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "mail",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 3.0,
                    y: 5.5,
                    width: 18.0,
                    height: 13.0,
                    rx: 2.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M3.5 7l8.5 6 8.5-6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "lock",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 5.0,
                    y: 10.5,
                    width: 14.0,
                    height: 10.0,
                    rx: 2.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M8 10.5V7.5a4 4 0 0 1 8 0v3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M12 14.5v2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "eye",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path(
                    "M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 3.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "heart",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path(
                "M12 20s-7-4.35-7-10a4 4 0 0 1 7-2.65A4 4 0 0 1 19 10c0 5.65-7 10-7 10z",
            ),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "star",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path(
                "M12 3.5l2.6 5.3 5.9.9-4.25 4.1 1 5.8L12 16.9l-5.25 2.7 1-5.8L3.5 9.7l5.9-.9z",
            ),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "bookmark",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M6.5 4.5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v16l-5.5-4-5.5 4z"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "calendar",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 4.0,
                    y: 5.5,
                    width: 16.0,
                    height: 15.0,
                    rx: 2.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M4 10h16"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M8.5 3.5v4M15.5 3.5v4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M8.5 14h.01M12 14h.01M15.5 14h.01M8.5 17h.01M12 17h.01"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "clock",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 12V7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M12 12l3 2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "document",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M14 3.5H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M14 3.5v5h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "l1 draw",
                geometry: Geometry::Path("M8.5 12.5h7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "l2 draw",
                geometry: Geometry::Path("M8.5 16h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "file-diff",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M14 3.5H7a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5zM14 3.5v5h5",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M8.5 12h7M12 9.5v5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b draw",
                geometry: Geometry::Path("M8.5 17h7"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "book-open",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M12 5.5C9 3.8 5.8 4 3 5v14c2.8-1 6-1.2 9 .5 3-1.7 6.2-1.5 9-.5V5c-2.8-1-6-1.2-9 .5ZM12 5.5v14",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a draw",
                geometry: Geometry::Path("M6 8.5c1-.2 2-.1 3 .4M6 12c1-.2 2-.1 3 .4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b draw",
                geometry: Geometry::Path("M15 8.9c1-.5 2-.6 3-.4M15 12.4c1-.5 2-.6 3-.4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "folder",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path(
                    "M3.5 7.5a2 2 0 0 1 2-2h3.6a2 2 0 0 1 1.4.6l1.4 1.4h6.6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M3.5 11h17"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "chat",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M20.5 12a8 8 0 0 1-11.6 7.1L4 20.5l1.4-4.6A8 8 0 1 1 20.5 12z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "d1",
                geometry: Geometry::Circle {
                    cx: 8.5,
                    cy: 12.0,
                    r: 1.1,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d2",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 1.1,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d3",
                geometry: Geometry::Circle {
                    cx: 15.5,
                    cy: 12.0,
                    r: 1.1,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "user",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 8.0,
                    r: 3.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M5 20a7 7 0 0 1 14 0"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "users",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 9.0,
                    cy: 8.0,
                    r: 3.2,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M3 19.5a6 6 0 0 1 12 0"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M15.5 4.9a3.2 3.2 0 0 1 0 6.2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M17.5 14.2a6 6 0 0 1 3.5 5.3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "globe",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M3.5 12h17"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Ellipse {
                    cx: 12.0,
                    cy: 12.0,
                    rx: 3.6,
                    ry: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "gauge",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M4.6 17.5a8.5 8.5 0 1 1 14.8 0"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 14l3.5-4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 14.0,
                    r: 1.2,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "droplet",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M12 3.5s-6 6.5-6 10.5a6 6 0 0 0 12 0c0-4-6-10.5-6-10.5z"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "palette",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path(
                    "M12 3.5a8.5 8.5 0 1 0 0 17c1.1 0 1.8-.8 1.8-1.7 0-.5-.2-.9-.5-1.2-.3-.3-.5-.7-.5-1.2 0-.9.8-1.7 1.7-1.7h2a4 4 0 0 0 4-4C20.5 6.9 16.7 3.5 12 3.5z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "d1",
                geometry: Geometry::Circle {
                    cx: 7.5,
                    cy: 11.5,
                    r: 1.15,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d2",
                geometry: Geometry::Circle {
                    cx: 9.0,
                    cy: 7.6,
                    r: 1.15,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d3",
                geometry: Geometry::Circle {
                    cx: 13.2,
                    cy: 6.8,
                    r: 1.15,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
            Part {
                group: "d4",
                geometry: Geometry::Circle {
                    cx: 16.6,
                    cy: 9.2,
                    r: 1.15,
                },
                fill: true,
                stroke: false,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "ruler",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 2.5,
                    y: 8.0,
                    width: 19.0,
                    height: 8.0,
                    rx: 1.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "t1",
                geometry: Geometry::Path("M6.5 8v3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "t2",
                geometry: Geometry::Path("M10 8v2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "t3",
                geometry: Geometry::Path("M13.5 8v3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "t4",
                geometry: Geometry::Path("M17 8v2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "type",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M5 7V5h14v2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 5v14"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M9.5 19h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "cube",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 3l8 4.5v9L12 21l-8-4.5v-9z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M4 7.5l8 4.5 8-4.5M12 12v9"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "layers",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M12 3.5l8.5 4.5-8.5 4.5L3.5 8z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M3.5 12l8.5 4.5 8.5-4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "c",
                geometry: Geometry::Path("M3.5 16l8.5 4.5 8.5-4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "window",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 3.0,
                    y: 4.0,
                    width: 18.0,
                    height: 16.0,
                    rx: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Rect {
                    x: 7.0,
                    y: 8.5,
                    width: 10.0,
                    height: 7.0,
                    rx: 1.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "input",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 2.5,
                    y: 6.5,
                    width: 19.0,
                    height: 11.0,
                    rx: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M7.5 9.5v5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "sidebar",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 3.0,
                    y: 4.5,
                    width: 18.0,
                    height: 15.0,
                    rx: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M9 4.5v15"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "code",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path("M8.5 7.5L4 12l4.5 4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M15.5 7.5L20 12l-4.5 4.5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "c",
                geometry: Geometry::Path("M13.5 5.5l-3 13"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "terminal",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Rect {
                    x: 3.0,
                    y: 4.5,
                    width: 18.0,
                    height: 15.0,
                    rx: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("m6.5 9 3 3-3 3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b draw",
                geometry: Geometry::Path("M12.5 15h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "git-branch",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M6 8v8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a draw",
                geometry: Geometry::Path("M6 16c0-6 12-1 12-8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 18.0,
                    cy: 5.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 5.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 18.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "git-pull-request",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M6 8v8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 5.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 18.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a draw",
                geometry: Geometry::Path("M18 16V9a3.5 3.5 0 0 0-3.5-3.5H12"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("m15 2.5-3 3 3 3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 18.0,
                    cy: 18.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "git-merge",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M6 8v8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 5.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 6.0,
                    cy: 18.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a draw",
                geometry: Geometry::Path("M6 8c0 5 12 1 12 8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 18.0,
                    cy: 18.5,
                    r: 2.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "git-commit",
        parts: &[
            Part {
                group: "a draw",
                geometry: Geometry::Path("M3 12h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 4.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "c draw",
                geometry: Geometry::Path("M16 12h5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "grid",
        parts: &[
            Part {
                group: "s1",
                geometry: Geometry::Rect {
                    x: 4.0,
                    y: 4.0,
                    width: 6.5,
                    height: 6.5,
                    rx: 1.6,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "s2",
                geometry: Geometry::Rect {
                    x: 13.5,
                    y: 4.0,
                    width: 6.5,
                    height: 6.5,
                    rx: 1.6,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "s3",
                geometry: Geometry::Rect {
                    x: 13.5,
                    y: 13.5,
                    width: 6.5,
                    height: 6.5,
                    rx: 1.6,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "s4",
                geometry: Geometry::Rect {
                    x: 4.0,
                    y: 13.5,
                    width: 6.5,
                    height: 6.5,
                    rx: 1.6,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "chart-bar",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M3.5 20.5h17"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b1",
                geometry: Geometry::Rect {
                    x: 5.5,
                    y: 12.0,
                    width: 3.0,
                    height: 6.0,
                    rx: 1.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b2",
                geometry: Geometry::Rect {
                    x: 10.5,
                    y: 6.0,
                    width: 3.0,
                    height: 12.0,
                    rx: 1.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b3",
                geometry: Geometry::Rect {
                    x: 15.5,
                    y: 9.0,
                    width: 3.0,
                    height: 9.0,
                    rx: 1.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "trending",
        parts: &[
            Part {
                group: "draw",
                geometry: Geometry::Path("M3 17l6-6 4 4 8-8"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "b",
                geometry: Geometry::Path("M15 7h6v6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "wave",
        parts: &[Part {
            group: "draw",
            geometry: Geometry::Path(
                "M2.5 12c1.6-4 3.2-6 4.75-6S10.4 18 12 18s3.2-12 4.75-12S19.9 10 21.5 12",
            ),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "pie",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Path("M20.5 13.5A8.5 8.5 0 1 1 10.5 3.6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M14 3.5a7 7 0 0 1 6.5 6.5H14z"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "info",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "ic",
                geometry: Geometry::Path("M12 11v5"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 7.8v.01"),
                fill: false,
                stroke: true,
                stroke_width: Some(2.2),
            },
        ],
    },
    IconDef {
        name: "alert",
        parts: &[
            Part {
                group: "a",
                geometry: Geometry::Path(
                    "M10.3 4.3L2.9 17a2 2 0 0 0 1.7 3h14.8a2 2 0 0 0 1.7-3L13.7 4.3a2 2 0 0 0-3.4 0z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 9.5v4"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M12 16.8v.01"),
                fill: false,
                stroke: true,
                stroke_width: Some(2.2),
            },
        ],
    },
    IconDef {
        name: "check-circle",
        parts: &[
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "draw",
                geometry: Geometry::Path("M8.3 12.3l2.6 2.6 4.9-5.2"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "x-circle",
        parts: &[
            Part {
                group: "b",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 8.5,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M9.2 9.2l5.6 5.6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path("M14.8 9.2l-5.6 5.6"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "bolt",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M13 2.5L4.5 13.5H12l-1 8 8.5-11H12l1-8z"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "sparkles",
        parts: &[
            Part {
                group: "s1",
                geometry: Geometry::Path(
                    "M10 3c.5 3.5 2.5 5.5 6 6-3.5.5-5.5 2.5-6 6-.5-3.5-2.5-5.5-6-6 3.5-.5 5.5-2.5 6-6z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "s2",
                geometry: Geometry::Path(
                    "M18 13.5c.2 1.4 1 2.3 2.5 2.5-1.5.2-2.3 1-2.5 2.5-.2-1.5-1-2.3-2.5-2.5 1.5-.2 2.3-1.1 2.5-2.5z",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "s3",
                geometry: Geometry::Path("M5.5 17.5v3M4 19h3"),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "sun",
        parts: &[
            Part {
                group: "ic",
                geometry: Geometry::Circle {
                    cx: 12.0,
                    cy: 12.0,
                    r: 4.0,
                },
                fill: false,
                stroke: true,
                stroke_width: None,
            },
            Part {
                group: "a",
                geometry: Geometry::Path(
                    "M12 2.5v2M12 19.5v2M4.6 4.6L6 6M18 18l1.4 1.4M2.5 12h2M19.5 12h2M4.6 19.4L6 18M18 6l1.4-1.4",
                ),
                fill: false,
                stroke: true,
                stroke_width: None,
            },
        ],
    },
    IconDef {
        name: "moon",
        parts: &[Part {
            group: "a",
            geometry: Geometry::Path("M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
    IconDef {
        name: "loader",
        parts: &[Part {
            group: "ic",
            geometry: Geometry::Path("M12 3.5a8.5 8.5 0 1 0 8.5 8.5"),
            fill: false,
            stroke: true,
            stroke_width: None,
        }],
    },
];
