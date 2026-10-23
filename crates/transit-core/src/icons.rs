//! Built-in pictogram library. Every glyph is drawn in a 24×24 box on top of a
//! rounded background tile; `{fg}` is replaced with the icon color.

pub struct IconDef {
    pub name: &'static str,
    pub label: &'static str,
    pub body: &'static str,
}

pub const ICONS: &[IconDef] = &[
    IconDef {
        name: "wheelchair",
        label: "Step-free access",
        body: r#"<circle cx="11" cy="4.6" r="2" fill="{fg}"/><path d="M11 8v6.5h5.2l2.3 5" fill="none" stroke="{fg}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/><path d="M11 11.2h4" stroke="{fg}" stroke-width="2" stroke-linecap="round"/><path d="M8.2 11.3a5 5 0 1 0 7.1 6.4" fill="none" stroke="{fg}" stroke-width="2" stroke-linecap="round"/>"#,
    },
    IconDef {
        name: "airport",
        label: "Airport",
        body: r#"<path d="M12 2.8c.9 0 1.4.8 1.4 1.7v5.2l7.3 4.4v2l-7.3-2.2v4.6l2.1 1.6v1.6L12 20.8l-3.5.9v-1.6l2.1-1.6v-4.6l-7.3 2.2v-2l7.3-4.4V4.5c0-.9.5-1.7 1.4-1.7z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "ferry",
        label: "Ferry",
        body: r#"<path d="M9 4h6v3H9z" fill="{fg}"/><path d="M6.5 8h11l1.5 5.2-7-2.6-7 2.6z" fill="{fg}"/><path d="M4 14l8-3 8 3-1.6 4.2H5.6z" fill="{fg}"/><path d="M3 20.5c1.5 0 1.5-1 3-1s1.5 1 3 1 1.5-1 3-1 1.5 1 3 1 1.5-1 3-1 1.5 1 3 1" fill="none" stroke="{fg}" stroke-width="1.4" stroke-linecap="round"/>"#,
    },
    IconDef {
        name: "rail",
        label: "National rail",
        body: r#"<path d="M3 9.2h18M3 14.8h18M13.5 3.5L8 9.2l6 5.6-5 5.7" fill="none" stroke="{fg}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    },
    IconDef {
        name: "metro",
        label: "Metro",
        body: r#"<path d="M5 19V5.5h2.6L12 12.6l4.4-7.1H19V19h-2.6v-8.6L12 17.3l-4.4-6.9V19z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "parking",
        label: "Parking",
        body: r#"<path fill-rule="evenodd" d="M7 4h6.2a5 5 0 0 1 0 10H10v6H7zm3 3v4h3.2a2 2 0 0 0 0-4z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "bus",
        label: "Bus",
        body: r#"<path fill-rule="evenodd" d="M6 3.5h12a1.5 1.5 0 0 1 1.5 1.5v12.5a1 1 0 0 1-1 1h-.5V20a1 1 0 0 1-1 1h-1a1 1 0 0 1-1-1v-1.5H9V20a1 1 0 0 1-1 1H7a1 1 0 0 1-1-1v-1.5h-.5a1 1 0 0 1-1-1V5A1.5 1.5 0 0 1 6 3.5zM7 6.5v5h10v-5zm.6 7.5a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4zm8.8 0a1.2 1.2 0 1 0 0 2.4 1.2 1.2 0 0 0 0-2.4z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "tram",
        label: "Tram",
        body: r#"<path d="M8 2.5h8M12 2.5v3" fill="none" stroke="{fg}" stroke-width="1.6" stroke-linecap="round"/><path fill-rule="evenodd" d="M8 5.5h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2v-9a2 2 0 0 1 2-2zm0 2.5v4.5h8V8zm1 7a1 1 0 1 0 0 2 1 1 0 0 0 0-2zm6 0a1 1 0 1 0 0 2 1 1 0 0 0 0-2z" fill="{fg}"/><path d="M8.5 18.5L6.5 21.5M15.5 18.5l2 3" stroke="{fg}" stroke-width="1.6" stroke-linecap="round"/>"#,
    },
    IconDef {
        name: "bike",
        label: "Bicycle",
        body: r#"<circle cx="6" cy="15.5" r="3.6" fill="none" stroke="{fg}" stroke-width="1.8"/><circle cx="18" cy="15.5" r="3.6" fill="none" stroke="{fg}" stroke-width="1.8"/><path d="M6 15.5l3.5-7h6.5L18 15.5M9.5 8.5l3 7H6M12.5 15.5l3.5-7M8 6.5h3" fill="none" stroke="{fg}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>"#,
    },
    IconDef {
        name: "taxi",
        label: "Taxi",
        body: r#"<path d="M9.5 4h5l.6 2.2h-6.2z" fill="{fg}"/><path fill-rule="evenodd" d="M7.3 7.5h9.4l2 5H20a1 1 0 0 1 1 1v4a1 1 0 0 1-1 1h-1V20a1 1 0 0 1-1 1h-1a1 1 0 0 1-1-1v-1.5H8V20a1 1 0 0 1-1 1H6a1 1 0 0 1-1-1v-1.5H4a1 1 0 0 1-1-1v-4a1 1 0 0 1 1-1h1.3zm1.2 2l-1.1 3h9.2l-1.1-3zM6.5 14.2a1.3 1.3 0 1 0 0 2.6 1.3 1.3 0 0 0 0-2.6zm11 0a1.3 1.3 0 1 0 0 2.6 1.3 1.3 0 0 0 0-2.6z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "info",
        label: "Information",
        body: r#"<circle cx="12" cy="5.5" r="2.1" fill="{fg}"/><path d="M9 9.5h4.5V18H15v2.2H9V18h1.6v-6.3H9z" fill="{fg}"/>"#,
    },
    IconDef {
        name: "interchange",
        label: "Interchange",
        body: r#"<path d="M4 8.5h13M13.5 4.5l4 4-4 4M20 15.5H7M10.5 11.5l-4 4 4 4" fill="none" stroke="{fg}" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>"#,
    },
];

pub fn find(name: &str) -> Option<&'static IconDef> {
    ICONS.iter().find(|i| i.name == name)
}

/// SVG markup for the icon body in its 24×24 box (background tile included).
pub fn markup(name: &str, fg: &str, bg: &str) -> String {
    let body = find(name).map(|d| d.body).unwrap_or(r#"<circle cx="12" cy="12" r="5" fill="{fg}"/>"#);
    let mut s = String::new();
    if !bg.is_empty() {
        s.push_str(&format!(r#"<rect width="24" height="24" rx="5" fill="{}"/>"#, crate::render::esc(bg)));
    }
    s.push_str(&body.replace("{fg}", &crate::render::esc(fg)));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_has_required_pictograms() {
        for name in ["wheelchair", "airport", "ferry", "rail", "parking"] {
            assert!(find(name).is_some(), "{name} missing");
        }
    }

    #[test]
    fn names_are_unique() {
        let mut names: Vec<_> = ICONS.iter().map(|i| i.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ICONS.len());
    }

    #[test]
    fn markup_substitutes_colors_and_is_well_formed() {
        for icon in ICONS {
            let m = markup(icon.name, "#fff", "#123456");
            assert!(!m.contains("{fg}"));
            let doc = format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{m}</svg>"#);
            roxmltree::Document::parse(&doc).unwrap_or_else(|e| panic!("{}: {e}", icon.name));
        }
    }

    #[test]
    fn unknown_icon_falls_back_to_placeholder() {
        assert!(markup("nope", "#000", "").contains("circle"));
    }
}
