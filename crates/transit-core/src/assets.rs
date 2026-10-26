//! Fonts and SVG icons uploaded by the user. They are stored inside the
//! document so a `.transit` file (and its SVG export) is self-contained.
//!
//! Uploaded SVG is untrusted (files are shared), so it is reduced to an
//! allowlist of shape elements and presentation attributes before it is ever
//! rendered.

use crate::render::esc;
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;

/// Largest accepted SVG upload, in bytes.
pub const MAX_SVG_BYTES: usize = 512 * 1024;
/// Largest accepted font upload (as a data URL), in bytes.
pub const MAX_FONT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Assets {
    pub fonts: Vec<FontAsset>,
    pub icons: Vec<IconAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontAsset {
    /// Family name text layers refer to.
    pub family: String,
    /// The font file as a base64 `data:` URL. Shared, so undo snapshots don't copy it.
    pub data: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconAsset {
    /// Identifier used by `Icon::icon` (`custom-1`, `custom-2`, ...).
    pub name: String,
    pub label: String,
    /// viewBox of the uploaded drawing.
    pub view_box: String,
    /// Sanitized markup of the drawing's content.
    pub body: String,
}

impl Assets {
    pub fn icon(&self, name: &str) -> Option<&IconAsset> {
        self.icons.iter().find(|i| i.name == name)
    }

    pub fn font(&self, family: &str) -> Option<&FontAsset> {
        self.fonts.iter().find(|f| f.family == family)
    }

    /// Drop anything that is not valid (used when loading a file).
    pub fn sanitized(mut self) -> Self {
        self.fonts.retain(|f| clean_family(&f.family).as_deref() == Some(f.family.as_str()) && valid_font_data(&f.data));
        self.icons = self
            .icons
            .into_iter()
            .filter_map(|i| {
                let src = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">{}</svg>"#, esc(&i.view_box), i.body);
                let (view_box, body) = sanitize_svg(&src).ok()?;
                Some(IconAsset { view_box, body, ..i })
            })
            .collect();
        self
    }
}

/// The CSS `font-family` value text layers use for an uploaded font.
pub fn font_family_css(family: &str) -> String {
    format!("\"{family}\", sans-serif")
}

/// A usable family name: letters, digits, spaces, dashes and underscores.
pub fn clean_family(name: &str) -> Option<String> {
    let cleaned: String = name.chars().filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_')).take(64).collect();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

/// A base64 data URL of a TrueType, OpenType or WOFF font.
pub fn valid_font_data(data: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "data:font/ttf;base64,",
        "data:font/otf;base64,",
        "data:font/woff;base64,",
        "data:font/woff2;base64,",
        "data:font/sfnt;base64,",
        "data:application/font-woff;base64,",
        "data:application/x-font-ttf;base64,",
        "data:application/x-font-otf;base64,",
        "data:application/vnd.ms-opentype;base64,",
        "data:application/octet-stream;base64,",
    ];
    data.len() <= MAX_FONT_BYTES
        && PREFIXES.iter().any(|p| {
            data.strip_prefix(p)
                .is_some_and(|b64| !b64.is_empty() && b64.bytes().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'/' | b'=')))
        })
}

/// `@font-face` rules for the given fonts (for an SVG `<style>` element).
pub fn font_face_css(fonts: &[&FontAsset]) -> String {
    let mut css = String::new();
    for f in fonts {
        let _ = write!(css, r#"@font-face{{font-family:"{}";src:url({})}}"#, f.family, f.data);
    }
    css
}

const ELEMENTS: &[&str] = &[
    "g", "path", "circle", "ellipse", "rect", "line", "polyline", "polygon", "defs", "linearGradient", "radialGradient",
    "stop", "clipPath", "mask", "title", "desc", "text", "tspan",
];

const ATTRIBUTES: &[&str] = &[
    "id", "d", "points", "x", "y", "x1", "y1", "x2", "y2", "cx", "cy", "r", "rx", "ry", "fx", "fy", "width", "height",
    "transform", "fill", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin", "stroke-miterlimit",
    "stroke-dasharray", "stroke-dashoffset", "fill-rule", "clip-rule", "opacity", "fill-opacity", "stroke-opacity",
    "offset", "stop-color", "stop-opacity", "gradientUnits", "gradientTransform", "spreadMethod", "clipPathUnits",
    "maskUnits", "maskContentUnits", "clip-path", "mask", "font-size", "font-family", "font-weight", "font-style",
    "text-anchor", "dominant-baseline", "dx", "dy", "display", "visibility", "color",
];

/// Values may only reference fragments inside the icon itself.
fn safe_value(v: &str) -> bool {
    let lower = v.to_ascii_lowercase();
    if lower.contains("javascript:") || lower.contains("expression") || lower.contains('\\') || lower.contains('@') {
        return false;
    }
    let mut rest = lower.as_str();
    while let Some(i) = rest.find("url(") {
        let after = rest[i + 4..].trim_start().trim_start_matches(['\'', '"']);
        if !after.starts_with('#') {
            return false;
        }
        rest = &rest[i + 4..];
    }
    true
}

fn safe_style(style: &str) -> String {
    style
        .split(';')
        .filter_map(|decl| {
            let (prop, value) = decl.split_once(':')?;
            let (prop, value) = (prop.trim(), value.trim());
            (ATTRIBUTES.contains(&prop) && safe_value(value)).then(|| format!("{prop}:{value}"))
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn write_children(node: roxmltree::Node, out: &mut String) {
    for child in node.children() {
        if child.is_text() {
            if matches!(node.tag_name().name(), "text" | "tspan" | "title" | "desc") {
                out.push_str(&esc(child.text().unwrap_or("")));
            }
            continue;
        }
        if !child.is_element() {
            continue;
        }
        let tag = child.tag_name();
        if !matches!(tag.namespace(), None | Some("http://www.w3.org/2000/svg")) || !ELEMENTS.contains(&tag.name()) {
            continue;
        }
        let _ = write!(out, "<{}", tag.name());
        for a in child.attributes() {
            if a.namespace().is_some() {
                continue;
            }
            let value = if a.name() == "style" {
                safe_style(a.value())
            } else if ATTRIBUTES.contains(&a.name()) && safe_value(a.value()) {
                a.value().to_string()
            } else {
                continue;
            };
            if !value.is_empty() {
                let _ = write!(out, r#" {}="{}""#, a.name(), esc(&value));
            }
        }
        out.push('>');
        write_children(child, out);
        let _ = write!(out, "</{}>", tag.name());
    }
}

fn parse_len(v: Option<&str>) -> Option<f64> {
    v?.trim().trim_end_matches("px").parse::<f64>().ok().filter(|n| n.is_finite() && *n > 0.0)
}

/// Reduce an uploaded SVG file to its viewBox and a safe body: only shape,
/// gradient and text elements with presentation attributes are kept; scripts,
/// event handlers, styles, links and external references are dropped.
pub fn sanitize_svg(text: &str) -> Result<(String, String), String> {
    if text.len() > MAX_SVG_BYTES {
        return Err(format!("the SVG file is larger than {} kilobytes", MAX_SVG_BYTES / 1024));
    }
    let doc = roxmltree::Document::parse(text).map_err(|e| format!("not a valid SVG file: {e}"))?;
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        return Err("not an SVG file".into());
    }
    let view_box = root
        .attribute("viewBox")
        .map(|v| v.split(|c: char| c == ',' || c.is_whitespace()).filter(|s| !s.is_empty()).map(str::parse::<f64>).collect::<Result<Vec<_>, _>>())
        .and_then(|r| r.ok())
        .filter(|v| v.len() == 4 && v.iter().all(|n| n.is_finite()) && v[2] > 0.0 && v[3] > 0.0)
        .map(|v| v.iter().map(|n| crate::geom::fmt_num(*n)).collect::<Vec<_>>().join(" "))
        .or_else(|| {
            let w = parse_len(root.attribute("width"))?;
            let h = parse_len(root.attribute("height"))?;
            Some(format!("0 0 {} {}", crate::geom::fmt_num(w), crate::geom::fmt_num(h)))
        })
        .unwrap_or_else(|| "0 0 24 24".into());
    let mut body = String::new();
    write_children(root, &mut body);
    if body.is_empty() {
        return Err("the SVG file has no drawable content".into());
    }
    Ok((view_box, body))
}

/// Markup for an uploaded icon in the 24×24 pictogram box: the background tile
/// and the drawing fitted inside it. Drawings using `currentColor` take the
/// icon color.
pub fn icon_markup(icon: &IconAsset, fg: &str, bg: &str) -> String {
    let mut s = String::new();
    if !bg.is_empty() {
        let _ = write!(s, r#"<rect width="24" height="24" rx="5" fill="{}"/>"#, esc(bg));
    }
    let _ = write!(
        s,
        r#"<svg x="3" y="3" width="18" height="18" viewBox="{}" color="{}" preserveAspectRatio="xMidYMid meet">{}</svg>"#,
        esc(&icon.view_box),
        esc(fg),
        icon.body
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizer_keeps_shapes_and_drops_active_content() {
        let src = r##"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="48px" height="32">
  <script>alert(1)</script>
  <style>path{fill:red}</style>
  <defs><linearGradient id="g"><stop offset="0" stop-color="#f00"/></linearGradient></defs>
  <path d="M0 0L10 10" fill="url(#g)" onclick="alert(1)" style="stroke:#000;behavior:url(x.htc);fill:url(http://evil/x)"/>
  <a xlink:href="javascript:alert(1)"><circle r="3"/></a>
  <image href="http://evil/x.png"/>
  <foreignObject><div>hi</div></foreignObject>
  <rect width="5" height="5" fill="url(https://evil/x#y)"/>
  <text x="1">A &amp; B</text>
</svg>"##;
        let (vb, body) = sanitize_svg(src).unwrap();
        assert_eq!(vb, "0 0 48 32");
        for bad in ["script", "alert", "style>", "onclick", "evil", "href", "image", "foreignObject", "behavior"] {
            assert!(!body.contains(bad), "{bad} survived: {body}");
        }
        assert!(body.contains(r#"<path d="M0 0L10 10" fill="url(#g)" style="stroke:#000">"#), "{body}");
        assert!(body.contains("<linearGradient id=\"g\">"), "{body}");
        assert!(body.contains("<rect width=\"5\" height=\"5\">"), "{body}");
        assert!(body.contains("A &amp; B"), "{body}");
        let wrapped = format!(r#"<svg xmlns="http://www.w3.org/2000/svg">{body}</svg>"#);
        roxmltree::Document::parse(&wrapped).unwrap();
    }

    #[test]
    fn sanitizer_rejects_non_svg_and_empty_drawings() {
        assert!(sanitize_svg("<html/>").is_err());
        assert!(sanitize_svg("not xml").is_err());
        assert!(sanitize_svg(r#"<svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#).is_err());
        assert!(sanitize_svg(&format!("<svg>{}</svg>", " ".repeat(MAX_SVG_BYTES))).is_err());
        let (vb, _) = sanitize_svg(r#"<svg viewBox="0,0, 10 20"><path d="M0 0"/></svg>"#).unwrap();
        assert_eq!(vb, "0 0 10 20");
    }

    #[test]
    fn font_names_and_data_are_validated() {
        assert_eq!(clean_family("  My \"Font\"; } x  ").as_deref(), Some("My Font x"));
        assert_eq!(clean_family("<>"), None);
        assert!(valid_font_data("data:font/woff2;base64,d09GMgABAAA="));
        assert!(!valid_font_data("data:font/woff2;base64,abc)}"));
        assert!(!valid_font_data("data:text/css;base64,abcd"));
        assert!(!valid_font_data("https://evil/font.woff"));
    }

    #[test]
    fn loading_drops_invalid_assets() {
        let a = Assets {
            fonts: vec![
                FontAsset { family: "Good".into(), data: "data:font/ttf;base64,AAAA".into() },
                FontAsset { family: "Bad\"}".into(), data: "data:font/ttf;base64,AAAA".into() },
            ],
            icons: vec![IconAsset {
                name: "custom-1".into(),
                label: "x".into(),
                view_box: "0 0 24 24".into(),
                body: r#"<path d="M0 0" onload="alert(1)"/><script>x</script>"#.into(),
            }],
        }
        .sanitized();
        assert_eq!(a.fonts.len(), 1);
        assert_eq!(a.icons[0].body, r#"<path d="M0 0"></path>"#);
    }
}
