//! Painter-drawn VS Code-style (codicon / lucide) vector icons.
//!
//! Every icon is rendered with egui painter primitives so that it is crisp
//! at any DPI and looks identical on macOS, Linux, and Windows.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind};

fn line(p: &egui::Painter, a: Pos2, b: Pos2, c: Color32, w: f32) {
    p.line_segment([a, b], Stroke::new(w, c));
}

pub fn rect_outline(p: &egui::Painter, r: Rect, c: Color32, w: f32, rounding: f32) {
    p.rect_stroke(r, rounding, Stroke::new(w, c), StrokeKind::Inside);
}

fn center(r: Rect) -> Pos2 {
    r.center()
}

/// Files/explorer: two overlapping sheets with folded corner.
pub fn files(p: &egui::Painter, r: Rect, c: Color32) {
    let back = Rect::from_min_size(
        Pos2::new(r.left() + 6.5, r.top() + 2.0),
        egui::vec2(12.0, 15.0),
    );
    let front = Rect::from_min_size(
        Pos2::new(r.left() + 3.0, r.top() + 5.5),
        egui::vec2(12.0, 15.0),
    );
    rect_outline(p, back, c, 1.5, 1.5);
    p.rect_filled(front, 1.5, Color32::from_rgb(0x0e, 0x0d, 0x12));
    rect_outline(p, front, c, 1.5, 1.5);
    line(
        p,
        Pos2::new(front.left() + 3.0, front.top() + 5.0),
        Pos2::new(front.right() - 3.0, front.top() + 5.0),
        c,
        1.2,
    );
    line(
        p,
        Pos2::new(front.left() + 3.0, front.top() + 8.5),
        Pos2::new(front.right() - 4.5, front.top() + 8.5),
        c,
        1.2,
    );
}

/// Magnifier (Search).
pub fn search(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = Pos2::new(r.left() + 9.5, r.top() + 9.5);
    p.circle_stroke(ctr, 5.5, Stroke::new(1.6, c));
    line(
        p,
        Pos2::new(ctr.x + 4.0, ctr.y + 4.0),
        Pos2::new(r.right() - 3.0, r.bottom() - 3.0),
        c,
        1.8,
    );
}

/// Source control: Git branch nodes.
pub fn git(p: &egui::Painter, r: Rect, c: Color32) {
    let top = Pos2::new(r.left() + 6.5, r.top() + 5.5);
    let bot = Pos2::new(r.left() + 6.5, r.bottom() - 5.5);
    let right = Pos2::new(r.right() - 6.5, r.top() + 7.5);
    line(p, Pos2::new(top.x, top.y + 2.5), Pos2::new(bot.x, bot.y - 2.5), c, 1.5);
    // Curve from bot up to right
    let mid = Pos2::new(right.x, r.center().y + 1.5);
    line(p, Pos2::new(bot.x + 2.0, bot.y - 1.5), mid, c, 1.5);
    line(p, mid, Pos2::new(right.x, right.y + 2.5), c, 1.5);
    p.circle_stroke(top, 2.5, Stroke::new(1.5, c));
    p.circle_stroke(bot, 2.5, Stroke::new(1.5, c));
    p.circle_stroke(right, 2.5, Stroke::new(1.5, c));
}

/// Run and Debug: play circle icon (circle with triangle inside).
pub fn play_circle(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let rad = (r.width().min(r.height()) * 0.42).max(5.0);
    p.circle_stroke(ctr, rad, Stroke::new(1.5, c));
    let pts = vec![
        Pos2::new(ctr.x - 2.2, ctr.y - 3.5),
        Pos2::new(ctr.x + 3.8, ctr.y),
        Pos2::new(ctr.x - 2.2, ctr.y + 3.5),
    ];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.5, c)));
}

/// Play triangle.
pub fn play(p: &egui::Painter, r: Rect, c: Color32) {
    let pts = vec![
        Pos2::new(r.left() + 4.0, r.top() + 3.0),
        Pos2::new(r.right() - 3.0, r.center().y),
        Pos2::new(r.left() + 4.0, r.bottom() - 3.0),
    ];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.5, c)));
}

/// Activity / Profiler: pulse waveform polyline.
pub fn pulse(p: &egui::Painter, r: Rect, c: Color32) {
    let y = r.center().y;
    let pts = [
        Pos2::new(r.left() + 2.0, y),
        Pos2::new(r.left() + 6.5, y),
        Pos2::new(r.left() + 9.5, r.top() + 3.5),
        Pos2::new(r.left() + 13.5, r.bottom() - 3.5),
        Pos2::new(r.left() + 16.5, y),
        Pos2::new(r.right() - 2.0, y),
    ];
    for w in pts.windows(2) {
        line(p, w[0], w[1], c, 1.7);
    }
}

/// Extensions: 3D isometric cube / package icon (or 4-blocks).
pub fn extensions(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let w = 8.0;
    let h = 4.5;
    let top = Pos2::new(ctr.x, ctr.y - 7.5);
    let left = Pos2::new(ctr.x - w, ctr.y - 7.5 + h);
    let right = Pos2::new(ctr.x + w, ctr.y - 7.5 + h);
    let mid = Pos2::new(ctr.x, ctr.y - 7.5 + h * 2.0);
    let bot_l = Pos2::new(ctr.x - w, ctr.y + 7.5 - h);
    let bot_r = Pos2::new(ctr.x + w, ctr.y + 7.5 - h);
    let bot = Pos2::new(ctr.x, ctr.y + 7.5);

    line(p, top, left, c, 1.4);
    line(p, top, right, c, 1.4);
    line(p, left, mid, c, 1.4);
    line(p, right, mid, c, 1.4);
    line(p, left, bot_l, c, 1.4);
    line(p, right, bot_r, c, 1.4);
    line(p, mid, bot, c, 1.4);
    line(p, bot_l, bot, c, 1.4);
    line(p, bot_r, bot, c, 1.4);
}

/// Design Mockups: image frame with mountain + sun.
pub fn image_icon(p: &egui::Painter, r: Rect, c: Color32) {
    let box_r = r.shrink(3.0);
    rect_outline(p, box_r, c, 1.5, 2.5);
    p.circle_filled(
        Pos2::new(box_r.left() + 4.5, box_r.top() + 4.5),
        1.5,
        c,
    );
    let pts = [
        Pos2::new(box_r.left() + 2.0, box_r.bottom() - 2.0),
        Pos2::new(box_r.left() + 7.5, box_r.center().y),
        Pos2::new(box_r.left() + 11.0, box_r.bottom() - 4.0),
        Pos2::new(box_r.right() - 4.0, box_r.center().y - 1.0),
        Pos2::new(box_r.right() - 1.5, box_r.bottom() - 2.0),
    ];
    for w in pts.windows(2) {
        line(p, w[0], w[1], c, 1.4);
    }
}

/// Crab / Rust logo icon (drawn crisp vector crab/ferris).
pub fn rust_crab(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    // Shell oval
    p.circle_filled(Pos2::new(ctr.x, ctr.y + 1.0), 4.2, c);
    // Eyes
    p.circle_filled(Pos2::new(ctr.x - 1.6, ctr.y - 0.5), 0.9, Color32::BLACK);
    p.circle_filled(Pos2::new(ctr.x + 1.6, ctr.y - 0.5), 0.9, Color32::BLACK);
    // Claws
    line(
        p,
        Pos2::new(ctr.x - 3.5, ctr.y - 1.0),
        Pos2::new(ctr.x - 5.5, ctr.y - 3.2),
        c,
        1.5,
    );
    line(
        p,
        Pos2::new(ctr.x + 3.5, ctr.y - 1.0),
        Pos2::new(ctr.x + 5.5, ctr.y - 3.2),
        c,
        1.5,
    );
    // Legs
    for dy in [-0.5f32, 1.5, 3.2] {
        line(
            p,
            Pos2::new(ctr.x - 3.5, ctr.y + dy),
            Pos2::new(ctr.x - 6.0, ctr.y + dy + 1.0),
            c,
            1.2,
        );
        line(
            p,
            Pos2::new(ctr.x + 3.5, ctr.y + dy),
            Pos2::new(ctr.x + 6.0, ctr.y + dy + 1.0),
            c,
            1.2,
        );
    }
}

/// TOML / Config gear icon.
pub fn config_gear(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    p.circle_stroke(ctr, 3.5, Stroke::new(1.5, c));
    p.circle_filled(ctr, 1.3, c);
    for i in 0..6 {
        let a = (i as f32) * std::f32::consts::TAU / 6.0;
        let from = Pos2::new(ctr.x + a.cos() * 3.8, ctr.y + a.sin() * 3.8);
        let to = Pos2::new(ctr.x + a.cos() * 5.8, ctr.y + a.sin() * 5.8);
        line(p, from, to, c, 1.6);
    }
}

/// Markdown / Doc file icon.
pub fn doc_file(p: &egui::Painter, r: Rect, c: Color32) {
    let box_r = Rect::from_center_size(center(r), egui::vec2(10.0, 12.5));
    rect_outline(p, box_r, c, 1.3, 1.5);
    for dy in [-2.5f32, 0.5, 3.5] {
        line(
            p,
            Pos2::new(box_r.left() + 2.2, box_r.center().y + dy),
            Pos2::new(box_r.right() - 2.2, box_r.center().y + dy),
            c,
            1.1,
        );
    }
}

/// Account: head + shoulders.
pub fn account(p: &egui::Painter, r: Rect, c: Color32) {
    let head = Pos2::new(r.center().x, r.top() + 7.0);
    p.circle_stroke(head, 3.5, Stroke::new(1.5, c));
    let pts = vec![
        Pos2::new(r.left() + 4.5, r.bottom() - 3.5),
        Pos2::new(r.left() + 6.5, r.bottom() - 7.5),
        Pos2::new(r.right() - 6.5, r.bottom() - 7.5),
        Pos2::new(r.right() - 4.5, r.bottom() - 3.5),
    ];
    p.add(egui::Shape::line(pts, Stroke::new(1.5, c)));
}

/// Gear: hub + spokes.
pub fn gear(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    p.circle_stroke(ctr, 4.0, Stroke::new(1.5, c));
    p.circle_stroke(ctr, 1.5, Stroke::new(1.3, c));
    for i in 0..8 {
        let a = (i as f32) * std::f32::consts::TAU / 8.0;
        let from = Pos2::new(ctr.x + a.cos() * 5.0, ctr.y + a.sin() * 5.0);
        let to = Pos2::new(ctr.x + a.cos() * 7.2, ctr.y + a.sin() * 7.2);
        line(p, from, to, c, 1.5);
    }
}

/// Close X.
pub fn close_x(p: &egui::Painter, r: Rect, c: Color32) {
    let i = 3.5;
    line(
        p,
        Pos2::new(r.left() + i, r.top() + i),
        Pos2::new(r.right() - i, r.bottom() - i),
        c,
        1.4,
    );
    line(
        p,
        Pos2::new(r.right() - i, r.top() + i),
        Pos2::new(r.left() + i, r.bottom() - i),
        c,
        1.4,
    );
}

/// Chevron (points right when `open` is false, down when true).
pub fn chevron(p: &egui::Painter, r: Rect, c: Color32, open: bool) {
    let ctr = center(r);
    if open {
        line(
            p,
            Pos2::new(ctr.x - 3.5, ctr.y - 1.5),
            Pos2::new(ctr.x, ctr.y + 2.0),
            c,
            1.5,
        );
        line(
            p,
            Pos2::new(ctr.x, ctr.y + 2.0),
            Pos2::new(ctr.x + 3.5, ctr.y - 1.5),
            c,
            1.5,
        );
    } else {
        line(
            p,
            Pos2::new(ctr.x - 1.5, ctr.y - 3.5),
            Pos2::new(ctr.x + 2.0, ctr.y),
            c,
            1.5,
        );
        line(
            p,
            Pos2::new(ctr.x + 2.0, ctr.y),
            Pos2::new(ctr.x - 1.5, ctr.y + 3.5),
            c,
            1.5,
        );
    }
}

/// Plus sign.
pub fn plus(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    line(
        p,
        Pos2::new(ctr.x - 4.5, ctr.y),
        Pos2::new(ctr.x + 4.5, ctr.y),
        c,
        1.5,
    );
    line(
        p,
        Pos2::new(ctr.x, ctr.y - 4.5),
        Pos2::new(ctr.x, ctr.y + 4.5),
        c,
        1.5,
    );
}

/// File-plus icon.
pub fn file_plus(p: &egui::Painter, r: Rect, c: Color32) {
    let box_r = Rect::from_min_size(
        Pos2::new(r.left() + 3.0, r.top() + 2.0),
        egui::vec2(10.0, 12.0),
    );
    rect_outline(p, box_r, c, 1.3, 1.5);
    let cx = box_r.center().x;
    let cy = box_r.center().y;
    line(p, Pos2::new(cx - 2.5, cy), Pos2::new(cx + 2.5, cy), c, 1.3);
    line(p, Pos2::new(cx, cy - 2.5), Pos2::new(cx, cy + 2.5), c, 1.3);
}

/// Trash can.
pub fn trash(p: &egui::Painter, r: Rect, c: Color32) {
    let body = Rect::from_min_size(
        Pos2::new(r.left() + 4.5, r.top() + 5.5),
        egui::vec2(r.width() - 9.0, r.height() - 8.0),
    );
    rect_outline(p, body, c, 1.3, 1.0);
    line(
        p,
        Pos2::new(r.left() + 3.0, r.top() + 5.5),
        Pos2::new(r.right() - 3.0, r.top() + 5.5),
        c,
        1.4,
    );
    line(
        p,
        Pos2::new(r.center().x - 2.5, r.top() + 3.5),
        Pos2::new(r.center().x + 2.5, r.top() + 3.5),
        c,
        1.4,
    );
}

/// Split-editor: two vertical columns.
pub fn split(p: &egui::Painter, r: Rect, c: Color32) {
    let outer = r.shrink(3.5);
    rect_outline(p, outer, c, 1.3, 1.5);
    line(
        p,
        Pos2::new(outer.center().x, outer.top()),
        Pos2::new(outer.center().x, outer.bottom()),
        c,
        1.3,
    );
}

/// Three horizontal dots (More).
pub fn dots_h(p: &egui::Painter, r: Rect, c: Color32) {
    let cy = r.center().y;
    let cx = r.center().x;
    for dx in [-4.0f32, 0.0, 4.0] {
        p.circle_filled(Pos2::new(cx + dx, cy), 1.3, c);
    }
}

/// Maximize / Restore diagonal arrows.
pub fn maximize(p: &egui::Painter, r: Rect, c: Color32) {
    let b = r.shrink(4.0);
    // Top-right arrow
    line(p, Pos2::new(b.center().x + 1.0, b.center().y - 1.0), b.right_top(), c, 1.3);
    line(p, b.right_top(), Pos2::new(b.right() - 4.0, b.top()), c, 1.3);
    line(p, b.right_top(), Pos2::new(b.right(), b.top() + 4.0), c, 1.3);
    // Bottom-left arrow
    line(p, Pos2::new(b.center().x - 1.0, b.center().y + 1.0), b.left_bottom(), c, 1.3);
    line(p, b.left_bottom(), Pos2::new(b.left() + 4.0, b.bottom()), c, 1.3);
    line(p, b.left_bottom(), Pos2::new(b.left(), b.bottom() - 4.0), c, 1.3);
}

/// Terminal prompt `>_` icon.
pub fn terminal(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    line(
        p,
        Pos2::new(ctr.x - 5.0, ctr.y - 3.5),
        Pos2::new(ctr.x - 1.5, ctr.y),
        c,
        1.5,
    );
    line(
        p,
        Pos2::new(ctr.x - 1.5, ctr.y),
        Pos2::new(ctr.x - 5.0, ctr.y + 3.5),
        c,
        1.5,
    );
    line(
        p,
        Pos2::new(ctr.x, ctr.y + 3.5),
        Pos2::new(ctr.x + 5.0, ctr.y + 3.5),
        c,
        1.5,
    );
}

/// Warning triangle.
pub fn warning_triangle(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let top = Pos2::new(ctr.x, ctr.y - 5.0);
    let bl = Pos2::new(ctr.x - 5.5, ctr.y + 4.5);
    let br = Pos2::new(ctr.x + 5.5, ctr.y + 4.5);
    p.add(egui::Shape::closed_line(vec![top, br, bl], Stroke::new(1.4, c)));
    line(p, Pos2::new(ctr.x, ctr.y - 1.5), Pos2::new(ctr.x, ctr.y + 1.5), c, 1.4);
    p.circle_filled(Pos2::new(ctr.x, ctr.y + 3.0), 0.8, c);
}

/// Error / Alert circle.
pub fn alert_circle(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    p.circle_stroke(ctr, 5.0, Stroke::new(1.4, c));
    line(p, Pos2::new(ctr.x, ctr.y - 2.2), Pos2::new(ctr.x, ctr.y + 0.8), c, 1.4);
    p.circle_filled(Pos2::new(ctr.x, ctr.y + 2.6), 0.8, c);
}

/// Bug icon (Debug console).
pub fn bug(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    p.circle_stroke(Pos2::new(ctr.x, ctr.y + 1.0), 3.8, Stroke::new(1.4, c));
    line(p, Pos2::new(ctr.x - 5.5, ctr.y + 1.0), Pos2::new(ctr.x + 5.5, ctr.y + 1.0), c, 1.2);
    line(p, Pos2::new(ctr.x - 4.5, ctr.y - 2.0), Pos2::new(ctr.x - 2.0, ctr.y - 0.5), c, 1.2);
    line(p, Pos2::new(ctr.x + 4.5, ctr.y - 2.0), Pos2::new(ctr.x + 2.0, ctr.y - 0.5), c, 1.2);
}

/// Lightning / Zap icon.
pub fn zap(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let pts = vec![
        Pos2::new(ctr.x + 1.0, ctr.y - 5.5),
        Pos2::new(ctr.x - 4.0, ctr.y + 0.5),
        Pos2::new(ctr.x, ctr.y + 0.5),
        Pos2::new(ctr.x - 1.0, ctr.y + 5.5),
        Pos2::new(ctr.x + 4.0, ctr.y - 0.5),
        Pos2::new(ctr.x, ctr.y - 0.5),
    ];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.4, c)));
}

/// CPU / Chip icon.
pub fn cpu(p: &egui::Painter, r: Rect, c: Color32) {
    let box_r = Rect::from_center_size(center(r), egui::vec2(9.0, 9.0));
    rect_outline(p, box_r, c, 1.4, 1.5);
    rect_outline(p, box_r.shrink(2.5), c, 1.1, 0.5);
    for d in [-2.2f32, 2.2] {
        line(p, Pos2::new(box_r.center().x + d, box_r.top() - 2.0), Pos2::new(box_r.center().x + d, box_r.top()), c, 1.2);
        line(p, Pos2::new(box_r.center().x + d, box_r.bottom()), Pos2::new(box_r.center().x + d, box_r.bottom() + 2.0), c, 1.2);
        line(p, Pos2::new(box_r.left() - 2.0, box_r.center().y + d), Pos2::new(box_r.left(), box_r.center().y + d), c, 1.2);
        line(p, Pos2::new(box_r.right(), box_r.center().y + d), Pos2::new(box_r.right() + 2.0, box_r.center().y + d), c, 1.2);
    }
}

/// Layers / Stack icon.
pub fn layers(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let top = vec![
        Pos2::new(ctr.x, ctr.y - 5.5),
        Pos2::new(ctr.x + 6.0, ctr.y - 2.0),
        Pos2::new(ctr.x, ctr.y + 1.5),
        Pos2::new(ctr.x - 6.0, ctr.y - 2.0),
    ];
    p.add(egui::Shape::closed_line(top, Stroke::new(1.4, c)));
    line(p, Pos2::new(ctr.x - 6.0, ctr.y + 1.5), Pos2::new(ctr.x, ctr.y + 4.5), c, 1.4);
    line(p, Pos2::new(ctr.x, ctr.y + 4.5), Pos2::new(ctr.x + 6.0, ctr.y + 1.5), c, 1.4);
}

/// Shield check icon.
pub fn shield(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let pts = vec![
        Pos2::new(ctr.x, ctr.y - 6.0),
        Pos2::new(ctr.x + 5.0, ctr.y - 3.5),
        Pos2::new(ctr.x + 4.5, ctr.y + 2.0),
        Pos2::new(ctr.x, ctr.y + 6.0),
        Pos2::new(ctr.x - 4.5, ctr.y + 2.0),
        Pos2::new(ctr.x - 5.0, ctr.y - 3.5),
    ];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.4, c)));
    line(p, Pos2::new(ctr.x - 2.2, ctr.y), Pos2::new(ctr.x - 0.5, ctr.y + 2.0), c, 1.4);
    line(p, Pos2::new(ctr.x - 0.5, ctr.y + 2.0), Pos2::new(ctr.x + 2.5, ctr.y - 1.5), c, 1.4);
}

/// Checkmark icon.
pub fn check(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    line(p, Pos2::new(ctr.x - 4.0, ctr.y + 0.5), Pos2::new(ctr.x - 1.0, ctr.y + 3.5), c, 1.6);
    line(p, Pos2::new(ctr.x - 1.0, ctr.y + 3.5), Pos2::new(ctr.x + 4.5, ctr.y - 3.0), c, 1.6);
}

/// Sparkles / Copilot star icon.
pub fn sparkles(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    line(p, Pos2::new(ctr.x, ctr.y - 4.5), Pos2::new(ctr.x, ctr.y + 4.5), c, 1.4);
    line(p, Pos2::new(ctr.x - 4.5, ctr.y), Pos2::new(ctr.x + 4.5, ctr.y), c, 1.4);
    line(p, Pos2::new(ctr.x - 2.5, ctr.y - 2.5), Pos2::new(ctr.x + 2.5, ctr.y + 2.5), c, 1.1);
    line(p, Pos2::new(ctr.x + 2.5, ctr.y - 2.5), Pos2::new(ctr.x - 2.5, ctr.y + 2.5), c, 1.1);
}

/// Lightbulb (code action).
pub fn bulb(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = Pos2::new(r.center().x, r.top() + 6.5);
    p.circle_stroke(ctr, 3.8, Stroke::new(1.4, c));
    line(
        p,
        Pos2::new(ctr.x - 2.0, r.bottom() - 4.5),
        Pos2::new(ctr.x + 2.0, r.bottom() - 4.5),
        c,
        1.4,
    );
    line(
        p,
        Pos2::new(ctr.x - 1.5, r.bottom() - 2.5),
        Pos2::new(ctr.x + 1.5, r.bottom() - 2.5),
        c,
        1.4,
    );
}

/// Bell (notifications).
pub fn bell(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let pts = vec![
        Pos2::new(ctr.x - 4.0, ctr.y + 3.0),
        Pos2::new(ctr.x - 3.0, ctr.y - 2.5),
        Pos2::new(ctr.x, ctr.y - 4.5),
        Pos2::new(ctr.x + 3.0, ctr.y - 2.5),
        Pos2::new(ctr.x + 4.0, ctr.y + 3.0),
    ];
    p.add(egui::Shape::line(pts, Stroke::new(1.3, c)));
    line(
        p,
        Pos2::new(ctr.x - 5.0, ctr.y + 3.0),
        Pos2::new(ctr.x + 5.0, ctr.y + 3.0),
        c,
        1.4,
    );
    p.circle_filled(Pos2::new(ctr.x, ctr.y + 4.8), 1.2, c);
}

/// Refresh: circular arc + arrowhead.
pub fn refresh(p: &egui::Painter, r: Rect, c: Color32) {
    let ctr = center(r);
    let rad = 5.2f32;
    let mut prev: Option<Pos2> = None;
    for i in 0..=10 {
        let a: f32 = -0.4 + (i as f32) * 0.48;
        let pt = Pos2::new(ctr.x + a.cos() * rad, ctr.y + a.sin() * rad);
        if let Some(pr) = prev {
            line(p, pr, pt, c, 1.4);
        }
        prev = Some(pt);
    }
    let end_a: f32 = -0.4 + 10.0 * 0.48;
    let end = Pos2::new(ctr.x + end_a.cos() * rad, ctr.y + end_a.sin() * rad);
    line(p, end, Pos2::new(end.x - 3.5, end.y - 0.5), c, 1.4);
    line(p, end, Pos2::new(end.x - 0.5, end.y + 3.2), c, 1.4);
}

/// Numeric or text badge (e.g. `2` or `Rust`) drawn at a rect's top-right.
pub fn text_badge(p: &egui::Painter, r: Rect, label: &str, bg: Color32, fg: Color32) {
    if label.is_empty() {
        return;
    }
    let w = (label.len() as f32 * 5.2 + 7.0).max(14.0);
    let h = 13.0;
    let badge_rect = Rect::from_center_size(
        Pos2::new(r.right() - w * 0.5 - 3.0, r.top() + 11.0),
        egui::vec2(w, h),
    );
    p.rect_filled(badge_rect, 6.5, bg);
    p.text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(8.5),
        fg,
    );
}

/// Draw a file icon based on filename/language, matching the .aistudio screenshots
/// (orange crab for `.rs`, amber gear for `.toml`, blue doc for `.md`, etc.).
pub fn draw_file_icon(p: &egui::Painter, r: Rect, filename: &str) {
    let lower = filename.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        rust_crab(p, r, Color32::from_rgb(0xf9, 0x73, 0x16));
    } else if lower.ends_with(".toml") {
        config_gear(p, r, Color32::from_rgb(0xfb, 0xbf, 0x24));
    } else if lower.ends_with(".md") {
        doc_file(p, r, Color32::from_rgb(0x60, 0xa5, 0xfa));
    } else {
        let lang = crate::languages::Language::from_path(filename);
        let (bg, fg) = language_color(lang);
        file_chip(p, r, bg, lang.badge(), fg);
    }
}

/// Colored file-type chip used for other languages.
pub fn file_chip(p: &egui::Painter, r: Rect, bg: Color32, label: &str, fg: Color32) {
    p.rect_filled(r, 3.0, bg);
    p.text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::monospace(8.5),
        fg,
    );
}

/// Per-language chip colors (dark bg, bright fg).
pub fn language_color(lang: crate::languages::Language) -> (Color32, Color32) {
    use crate::languages::Language as L;
    match lang {
        L::Rust => (Color32::from_rgb(84, 44, 34), Color32::from_rgb(255, 150, 110)),
        L::Python => (Color32::from_rgb(38, 62, 92), Color32::from_rgb(140, 190, 255)),
        L::JavaScript => (Color32::from_rgb(92, 82, 30), Color32::from_rgb(255, 225, 110)),
        L::TypeScript => (Color32::from_rgb(30, 62, 104), Color32::from_rgb(120, 180, 255)),
        L::Go => (Color32::from_rgb(28, 84, 96), Color32::from_rgb(110, 220, 240)),
        L::C | L::Cpp => (Color32::from_rgb(44, 62, 104), Color32::from_rgb(150, 190, 255)),
        L::CSharp => (Color32::from_rgb(70, 44, 104), Color32::from_rgb(200, 160, 255)),
        L::Java => (Color32::from_rgb(96, 58, 30), Color32::from_rgb(255, 180, 110)),
        L::Kotlin => (Color32::from_rgb(84, 48, 110), Color32::from_rgb(210, 160, 255)),
        L::Swift => (Color32::from_rgb(100, 56, 32), Color32::from_rgb(255, 170, 110)),
        L::Ruby => (Color32::from_rgb(96, 34, 40), Color32::from_rgb(255, 140, 140)),
        L::Php => (Color32::from_rgb(56, 56, 100), Color32::from_rgb(170, 170, 255)),
        L::Html => (Color32::from_rgb(100, 54, 30), Color32::from_rgb(255, 170, 110)),
        L::Css => (Color32::from_rgb(30, 74, 110), Color32::from_rgb(120, 200, 255)),
        L::Json => (Color32::from_rgb(90, 78, 34), Color32::from_rgb(240, 210, 120)),
        L::Toml => (Color32::from_rgb(88, 60, 40), Color32::from_rgb(255, 190, 130)),
        L::Yaml => (Color32::from_rgb(92, 40, 50), Color32::from_rgb(255, 150, 160)),
        L::Markdown => (Color32::from_rgb(40, 70, 110), Color32::from_rgb(140, 200, 255)),
        L::Shell => (Color32::from_rgb(40, 84, 54), Color32::from_rgb(140, 240, 170)),
        L::Sql => (Color32::from_rgb(44, 78, 98), Color32::from_rgb(140, 210, 250)),
        L::Xml => (Color32::from_rgb(96, 64, 34), Color32::from_rgb(255, 190, 120)),
        L::Dockerfile => (Color32::from_rgb(30, 78, 104), Color32::from_rgb(120, 200, 255)),
        L::Plain => (Color32::from_rgb(66, 70, 80), Color32::from_rgb(190, 195, 208)),
    }
}
