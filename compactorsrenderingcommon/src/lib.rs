use std::cmp::max;

use compactorsanim::compactor::Compactor;
use ggez::{
    graphics::{Color, DrawMode, Mesh, MeshBuilder, Rect},
    GameError,
};
use renderingcommon::{
    Element, CAPACITY_RECT_COLOR, ELEMENT_WIDTH, MAX_ELEMENT_HEIGHT, SPACE_BETWEEN_ELEMENTS,
    USED_CAPACITY_RECT_COLOR,
};

fn capacity_rects(level: usize, capacity: usize) -> impl Iterator<Item = Rect> {
    (0..capacity).map(move |element_index| {
        compactor_element_to_rect(level, element_index, MAX_ELEMENT_HEIGHT)
    })
}

fn compactor_element_to_rect(level: usize, element_index: usize, element: Element) -> Rect {
    Rect::new_i32(
        (SPACE_BETWEEN_ELEMENTS + element_index as u32 * (SPACE_BETWEEN_ELEMENTS + ELEMENT_WIDTH))
            as i32,
        (SPACE_BETWEEN_ELEMENTS + level as u32 * (MAX_ELEMENT_HEIGHT + SPACE_BETWEEN_ELEMENTS))
            as i32,
        ELEMENT_WIDTH as i32,
        element as i32,
    )
}

pub fn compactor_mesh(
    ctx: &mut ggez::Context,
    current_frame: &Vec<Compactor<Element>>,
) -> Result<Mesh, GameError> {
    let mut mesh_builder = MeshBuilder::new();
    let mut used_capacity: i32 = current_frame.iter().map(|c| c.data.len() as i32).sum();
    for (level, compactor) in current_frame.iter().enumerate() {
        for (i, capacity_rect) in capacity_rects(level, compactor.capacity).enumerate() {
            let color = if (i as i32) < used_capacity {
                USED_CAPACITY_RECT_COLOR
            } else {
                CAPACITY_RECT_COLOR
            };
            mesh_builder.rectangle(DrawMode::fill(), capacity_rect, color)?;
        }
        used_capacity -= compactor.capacity as i32;
        for (element_index, &element) in compactor.data.iter().enumerate() {
            mesh_builder.rectangle(
                DrawMode::fill(),
                compactor_element_to_rect(level, element_index, element),
                Color::RED,
            )?;
        }
    }
    mesh_builder.build(ctx)
}

pub fn frame_size(compactors: &Vec<Compactor<Element>>) -> (u32, u32) {
    return (
        SPACE_BETWEEN_ELEMENTS
            + compactors
                .iter()
                .map(|c| c.data.len() as u32 * (ELEMENT_WIDTH + SPACE_BETWEEN_ELEMENTS))
                .max()
                .unwrap_or(0),
        SPACE_BETWEEN_ELEMENTS
            + compactors.len() as u32 * (MAX_ELEMENT_HEIGHT + SPACE_BETWEEN_ELEMENTS),
    );
}

pub fn max_frame_size<'a>(frames: impl Iterator<Item = &'a Vec<Compactor<Element>>>) -> (u32, u32) {
    frames.map(frame_size).fold((0, 0), |(w1, h1), (w2, h2)| {
        (max(w1 as u32, w2 as u32), max(h1 as u32, h2 as u32))
    })
}
