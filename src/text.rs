use std::collections::HashMap;

use artery_font::Rect;
use wgpu::util::DeviceExt;

use crate::{util::Quad, Graphics};

pub struct Text {
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

    pub fn create_buffer(
        &self,
        gfx: &Graphics,
        glyphs: &HashMap<u32, Glyph>,
    ) -> (wgpu::Buffer, u32) {
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

        for _ in 1..self.text.len() {
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

        let buffer =
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Text Buffer"),
                    contents: bytemuck::cast_slice(&result),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        (buffer, result.len() as u32)
    }
}

pub struct Glyph {
    pub advance_x: f32,
    pub plane_bounds: Rect,
    pub atlas_bounds: Rect,
}
