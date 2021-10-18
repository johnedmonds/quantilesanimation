use renderingcommon::{CAPACITY_RECT_COLOR, ELEMENT_WIDTH, Element, MAX_ELEMENT_HEIGHT, SPACE_BETWEEN_ELEMENTS, USED_CAPACITY_RECT_COLOR, save_frame};
use std::{cmp::max, fs::File};

use ggez::{
    conf::{NumSamples, WindowMode},
    event::EventHandler,
    graphics::{
        self, get_window_color_format, Canvas, Color, DrawMode, DrawParam, MeshBuilder, Rect,
    },
    GameError, GameResult,
};
use png::{BitDepth, ColorType};
use rand::{
    prelude::{Distribution, StdRng},
    SeedableRng,
};

use compactorsanim::compactor::Compactor;
use rand_distr::Normal;

use compactorsanim::compactors::Compactors;

struct MainState<I: Iterator<Item = Vec<Compactor<Element>>>> {
    frames: I,
    current_frame: Vec<Compactor<Element>>,
}

impl<I: Iterator<Item = Vec<Compactor<Element>>>> EventHandler<GameError> for MainState<I> {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, Color::BLACK);
        let mut mesh_builder = MeshBuilder::new();
        let mut used_capacity: i32 = self.current_frame.iter().map(|c| c.data.len() as i32).sum();
        for (level, compactor) in self.current_frame.iter().enumerate() {
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
        let mesh = mesh_builder.build(ctx)?;
        graphics::draw(ctx, &mesh, DrawParam::default())?;
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: ggez::event::KeyCode,
        _keymods: ggez::event::KeyMods,
        _repeat: bool,
    ) {
        if keycode == ggez::event::KeyCode::Escape {
            return ggez::event::quit(ctx);
        }
        if let Some(frame) = self.frames.next() {
            self.current_frame = frame;
        }
    }
}

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

fn frame_size(compactors: &Vec<Compactor<Element>>) -> (u32, u32) {
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

fn make_frames<D: Distribution<f32>, const LAZY: bool>(d: D) -> Vec<Vec<Compactor<Element>>> {
    let mut frames: Vec<Vec<Compactor<Element>>> = Vec::new();
    let mut compactors: Compactors<Element, _, LAZY> =
        Compactors::new(10, |frame| frames.push(frame));
    let mut r = StdRng::seed_from_u64(42);
    for _i in 1..100 {
        compactors.update(d.sample(&mut r) as u32);
    }
    frames
}

fn main() -> GameResult {
    let frames = make_frames::<_, true>(*renderingcommon::DISTRIBUTION);
    let (mut ctx, _event_loop) = ggez::ContextBuilder::new("quantiles", "jedmonds").build()?;
    let _mode = WindowMode::default();
    let (w, h) = frames
        .iter()
        .map(frame_size)
        .fold((0, 0), |(w1, h1), (w2, h2)| {
            (max(w1 as u32, w2 as u32), max(h1 as u32, h2 as u32))
        });
    let mut frames = frames.into_iter();
    let color_format = get_window_color_format(&ctx);
    let canvas = Canvas::new(&mut ctx, w as u16, h as u16, NumSamples::One, color_format)?;
    graphics::set_screen_coordinates(&mut ctx, Rect::new_i32(0, 0, w as i32, h as i32))?;
    graphics::set_canvas(&mut ctx, Some(&canvas));
    let mut state = MainState {
        current_frame: frames.next().unwrap_or(Vec::new()),
        frames,
    };
    state.draw(&mut ctx)?;
    let mut frame_id = 0;
    save_frame(canvas.to_rgba8(&mut ctx)?, frame_id, w, h);
    frame_id += 1;
    while let Some(frame) = state.frames.next() {
        state.current_frame = frame;
        graphics::set_canvas(&mut ctx, Some(&canvas));
        state.draw(&mut ctx)?;
        save_frame(canvas.to_rgba8(&mut ctx)?, frame_id, w, h);
        frame_id += 1;
    }

    Ok(())
}
