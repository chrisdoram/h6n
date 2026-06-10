//! A visual smoke-test for the `h6n` hexagon library.
//!
//! Builds a small hexagonal map and renders a handful of SVG files, one per
//! library operation (neighbours, distance, reflections, rotations), into a
//! `renders/` directory. Open the `.svg` files in a browser to inspect them.

use std::fs;
use std::path::Path;

use h6n::{Hex, Point, Pointy, Vector};

/// The pixel size (center-to-corner) used for every rendered hex.
const SIZE: f64 = 32.0;

/// A hexagon to draw, paired with its fill colour and a text label.
struct Cell {
    hex: Hex<Pointy>,
    fill: &'static str,
    label: String,
}

impl Cell {
    fn new(hex: Hex<Pointy>, fill: &'static str, label: impl Into<String>) -> Self {
        Self {
            hex,
            fill,
            label: label.into(),
        }
    }
}

/// Accumulates SVG fragments while tracking the bounding box so the final
/// document can be given a tight `viewBox`.
struct Svg {
    elems: Vec<String>,
    min: (f64, f64),
    max: (f64, f64),
}

impl Svg {
    fn new() -> Self {
        Self {
            elems: Vec::new(),
            min: (f64::INFINITY, f64::INFINITY),
            max: (f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    fn grow(&mut self, (x, y): (f64, f64)) {
        self.min = (self.min.0.min(x), self.min.1.min(y));
        self.max = (self.max.0.max(x), self.max.1.max(y));
    }

    /// Draw a hexagon: filled polygon, outline, and centered coordinate label.
    fn cell(&mut self, cell: &Cell) {
        let corners = cell.hex.corners(SIZE);
        let points = corners
            .iter()
            .map(|(x, y)| format!("{x:.2},{y:.2}"))
            .collect::<Vec<_>>()
            .join(" ");
        for corner in corners {
            self.grow(corner);
        }

        self.elems.push(format!(
            r##"<polygon points="{points}" fill="{}" stroke="#333" stroke-width="1.5"/>"##,
            cell.fill
        ));

        let (cx, cy) = cell.hex.center(SIZE);
        self.elems.push(format!(
            r##"<text x="{cx:.2}" y="{cy:.2}" font-family="monospace" font-size="11" fill="#111" text-anchor="middle" dominant-baseline="central">{}</text>"##,
            cell.label
        ));
    }

    /// Draw a straight line between two pixel points (used for distance).
    fn line(&mut self, a: (f64, f64), b: (f64, f64), stroke: &str) {
        self.grow(a);
        self.grow(b);
        self.elems.push(format!(
            r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="{stroke}" stroke-width="3" stroke-dasharray="6 4"/>"##,
            a.0, a.1, b.0, b.1
        ));
    }

    /// Free-floating text at a pixel point.
    fn text(&mut self, (x, y): (f64, f64), s: &str) {
        self.grow((x, y));
        self.elems.push(format!(
            r##"<text x="{x:.2}" y="{y:.2}" font-family="monospace" font-size="15" font-weight="bold" fill="#b00" text-anchor="middle">{s}</text>"##,
        ));
    }

    fn finish(self) -> String {
        let pad = SIZE;
        let min_x = self.min.0 - pad;
        let min_y = self.min.1 - pad;
        let w = self.max.0 - self.min.0 + 2.0 * pad;
        let h = self.max.1 - self.min.1 + 2.0 * pad;
        let body = self.elems.join("\n  ");
        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x:.2} {min_y:.2} {w:.2} {h:.2}">
  <rect x="{min_x:.2}" y="{min_y:.2}" width="{w:.2}" height="{h:.2}" fill="#fafafa"/>
  {body}
</svg>
"##
        )
    }
}

/// Compact `q,r,s` label for a hex.
///
/// Deliberately not the `Display` impl: its `(q, r, s)` form is up to four
/// characters wider and overflows a `SIZE`-pixel hex at this font size.
fn label(hex: &Hex<Pointy>) -> String {
    let p = hex.coordinate();
    format!("{},{},{}", p.q(), p.r(), p.s())
}

/// All hexes within `radius` rings of the origin.
fn hex_map(radius: i32) -> Vec<Hex<Pointy>> {
    let mut hexes = Vec::new();
    for q in -radius..=radius {
        let lo = (-radius).max(-q - radius);
        let hi = radius.min(-q + radius);
        for r in lo..=hi {
            hexes.push((q, r).into());
        }
    }
    hexes
}

/// Render the base map as faint background cells, returning the populated `Svg`.
fn base(radius: i32) -> Svg {
    let mut svg = Svg::new();
    for hex in hex_map(radius) {
        svg.cell(&Cell::new(hex, "#ffffff", label(&hex)));
    }
    svg
}

fn write(dir: &Path, name: &str, svg: Svg) {
    let path = dir.join(name);
    fs::write(&path, svg.finish()).expect("failed to write svg");
    println!("wrote {}", path.display());
}

fn neighbours(dir: &Path) {
    let mut svg = base(3);
    let center: Hex<Pointy> = (0, 0).into();
    svg.cell(&Cell::new(center, "#ffd166", label(&center)));
    for n in center.neighbors() {
        svg.cell(&Cell::new(n, "#90caf9", label(&n)));
    }
    write(dir, "neighbours.svg", svg);
}

fn distance(dir: &Path) {
    let mut svg = base(3);
    let a: Hex<Pointy> = (-2, 0).into();
    let b: Hex<Pointy> = (2, -1).into();
    svg.cell(&Cell::new(a, "#a5d6a7", label(&a)));
    svg.cell(&Cell::new(b, "#a5d6a7", label(&b)));
    svg.line(a.center(SIZE), b.center(SIZE), "#b00");

    let d = a.coordinate().distance(b.coordinate());
    let mid = {
        let (ax, ay) = a.center(SIZE);
        let (bx, by) = b.center(SIZE);
        ((ax + bx) / 2.0, (ay + by) / 2.0 - 12.0)
    };
    svg.text(mid, &format!("d = {d}"));
    write(dir, "distance.svg", svg);
}

fn reflections(dir: &Path) {
    let mut svg = base(3);
    let h: Hex<Pointy> = (3, -1).into();
    svg.cell(&Cell::new(h, "#ffd166", label(&h)));
    svg.cell(&Cell::new(h.reflect_q(), "#ef9a9a", label(&h.reflect_q())));
    svg.cell(&Cell::new(h.reflect_r(), "#a5d6a7", label(&h.reflect_r())));
    svg.cell(&Cell::new(h.reflect_s(), "#90caf9", label(&h.reflect_s())));
    write(dir, "reflections.svg", svg);
}

fn rotations(dir: &Path) {
    let mut svg = base(3);
    let origin: Point = (0, 0).into();
    svg.cell(&Cell::new(origin.into(), "#cccccc", "0,0,0"));

    // Rotate the spoke vector to the starting hex clockwise around the origin,
    // stepping through all six 60-degree positions.
    let start: Hex<Pointy> = (2, -1).into();
    let mut spoke: Vector = start.coordinate() - origin;
    let shades = [
        "#ffd166", "#f4a261", "#e76f51", "#90caf9", "#64b5f6", "#42a5f5",
    ];
    for shade in shades {
        let hex: Hex<Pointy> = (origin + spoke).into();
        svg.cell(&Cell::new(hex, shade, label(&hex)));
        spoke = spoke.rotate_clockwise();
    }
    write(dir, "rotations.svg", svg);
}

fn main() {
    let dir = Path::new("renders");
    fs::create_dir_all(dir).expect("failed to create output directory");

    neighbours(dir);
    distance(dir);
    reflections(dir);
    rotations(dir);

    println!("done — open the files in renders/ in a browser");
}
