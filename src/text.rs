use std::collections::HashMap;

use crate::util::Quad;

struct Text {
    x: f32,
    y: f32,
    z: f32,
    text: String,
}

impl Text {
    pub fn new(text: &str, pos: (f32, f32, f32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1,
            z: pos.2,
            text: text.to_owned(),
        }
    }

    pub fn to_vertices(&self, glyphs: &HashMap<u32, Glyph>) -> Vec<Quad> {
        let mut result = Vec::new();
        let mut chars = self.text.chars();

        let mut temp_right: f32;

        let glyph = glyphs.get(&(chars.next().unwrap() as u32)).unwrap();
        let x1 = self.x + glyph.plane_bounds.left;
        let y1 = self.y + glyph.plane_bounds.top;
        let x2 = self.x + glyph.plane_bounds.right;
        let y2 = self.y + glyph.plane_bounds.bottom;
        let tex_x1 = glyph.atlas_bounds.left;
        let tex_y1 = glyph.atlas_bounds.top;
        let tex_x2 = glyph.atlas_bounds.right;
        let tex_y2 = glyph.atlas_bounds.bottom;
        temp_right = x2;
        let vertex = Quad {
            top_left: [x1, y1, self.z],
            bottom_right: [x2, y2],
            tex_top_left: [tex_x1, tex_y1],
            tex_bottom_right: [tex_x2, tex_y2],
        };
        result.push(vertex);

        for n in 1..self.text.len() {
            let glyph = glyphs.get(&(chars.next().unwrap() as u32)).unwrap();
            let x1 = temp_right + glyph.advance_x;
            let y1 = self.y + glyph.plane_bounds.top;
            let x2 = temp_right + glyph.advance_x + glyph.plane_bounds.right;
            let y2 = self.y + glyph.plane_bounds.bottom;
            let tex_x1 = glyph.atlas_bounds.left;
            let tex_y1 = glyph.atlas_bounds.top;
            let tex_x2 = glyph.atlas_bounds.right;
            let tex_y2 = glyph.atlas_bounds.bottom;
            temp_right = x2;
            let vertex = Quad {
                top_left: [x1, y1, self.z],
                bottom_right: [x2, y2],
                tex_top_left: [tex_x1, tex_y1],
                tex_bottom_right: [tex_x2, tex_y2],
            };
            result.push(vertex);
        }

        result
    }
}