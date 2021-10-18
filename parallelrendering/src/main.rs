use std::cell::RefCell;

use compactorsanim::{compactor::Compactor, compactors::Compactors};
use compactorsrenderingcommon::{compactor_mesh, max_frame_size};
use ggez::{
    conf::{NumSamples, WindowMode},
    event::{self, EventHandler},
    graphics::{self, get_window_color_format, Canvas, Color, DrawParam, Rect, Transform},
    mint, GameError, GameResult,
};
use itertools::Itertools;
use rand::{prelude::StdRng, Rng, SeedableRng};
use rand_distr::Distribution;
use renderingcommon::{save_frame, Element};

struct CompactorFrames<I: Iterator<Item = Vec<Compactor<Element>>>> {
    frames: I,
    current_frame: Option<Vec<Compactor<Element>>>,
    w: usize,
    h: usize,
}

struct MainState<I: Iterator<Item = Vec<Compactor<Element>>>> {
    compactors: Vec<CompactorFrames<I>>,
}

impl<I: Iterator<Item = Vec<Compactor<Element>>>> MainState<I> {
    fn step(&mut self) {
        for compactor in self.compactors.iter_mut() {
            compactor.current_frame = compactor.frames.next();
        }
    }
}

impl<I: Iterator<Item = Vec<Compactor<Element>>>> EventHandler<GameError> for MainState<I> {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, Color::BLACK);
        let mut x = 0;
        for compactor in &self.compactors {
            if let Some(current_frame) = &compactor.current_frame {
                let mesh = compactor_mesh(ctx, &current_frame)?;
                let transform = Transform::Values {
                    dest: mint::Point2 {
                        x: x as f32,
                        y: 0.0,
                    },
                    rotation: 0.0,
                    scale: mint::Vector2 { x: 1.0, y: 1.0 },
                    offset: mint::Point2 { x: 0.0, y: 0.0 },
                };
                x += compactor.w as u32 + SPACE_BETWEEN_COMPACTORS;
                graphics::draw(
                    ctx,
                    &mesh,
                    DrawParam::default().transform(transform.to_bare_matrix()),
                )?;
            }
        }
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
        self.step();
    }
}

fn make_frames<R: Rng, D: Distribution<f32>, const LAZY: bool>(
    d: D,
    mut r: R,
    parallelism: usize,
) -> Vec<RefCell<Vec<Vec<Compactor<Element>>>>> {
    let frames: Vec<RefCell<Vec<Vec<Compactor<Element>>>>> =
        (0..parallelism).map(|_| RefCell::new(Vec::new())).collect();
    let mut compactors: Vec<Compactors<Element, _, LAZY>> = Vec::new();
    for i in 0..parallelism {
        let frames = &frames;
        let mut compactor = Compactors::new(10, move |frame| frames[i].borrow_mut().push(frame));
        for _i in 1..100 {
            compactor.update(d.sample(&mut r) as u32);
        }
        compactors.push(compactor);
    }
    while compactors.len() > 1 {
        compactors = compactors
            .into_iter()
            .chunks(2)
            .into_iter()
            .map(|mut chunk| {
                let mut first = chunk.next().unwrap();
                if let Some(second) = chunk.next() {
                    first.merge(second);
                    first
                } else {
                    first
                }
            })
            .collect();
    }

    frames
}

const SPACE_BETWEEN_COMPACTORS: u32 = 10;

fn main() -> GameResult {
    let frames =
        make_frames::<_, _, true>(*renderingcommon::DISTRIBUTION, StdRng::seed_from_u64(42), 4);
    let (mut ctx, _event_loop) = ggez::ContextBuilder::new("quantiles", "jedmonds").build()?;
    let mut state = MainState {
        compactors: frames
            .into_iter()
            .map(|frames| {
                let (w, h) = max_frame_size(frames.borrow().iter());
                let mut frames_iter = frames.into_inner().into_iter();

                CompactorFrames {
                    h: h as usize,
                    w: w as usize,
                    current_frame: frames_iter.next(),
                    frames: frames_iter,
                }
            })
            .collect(),
    };
    println!(
        "{} {} {} {}",
        state.compactors[0].w, state.compactors[1].w, state.compactors[2].w, state.compactors[3].w
    );
    let w: u32 = state
        .compactors
        .iter()
        .map(|f| f.w as u32 + SPACE_BETWEEN_COMPACTORS)
        .sum();
    let h = state.compactors.iter().map(|f| f.h).max().unwrap() as u32;
    let mut mode = WindowMode::default();
    mode.width = w as f32;
    mode.height = h as f32;
    graphics::set_mode(&mut ctx, mode)?;
    graphics::set_screen_coordinates(&mut ctx, Rect::new_i32(0, 0, w as i32, h as i32))?;
    event::run(ctx, _event_loop, state)
    // let color_format = get_window_color_format(&ctx);
    // let canvas = Canvas::new(&mut ctx, w as u16, h as u16, NumSamples::One, color_format)?;
    // graphics::set_canvas(&mut ctx, Some(&canvas));

    // let mut frame_id = 0;
    // while state.compactors.iter().any(|c| c.current_frame.is_some()) {
    //     graphics::set_canvas(&mut ctx, Some(&canvas));
    //     state.draw(&mut ctx)?;
    //     save_frame(canvas.to_rgba8(&mut ctx)?, frame_id, w, h);
    //     frame_id += 1;
    //     state.step();
    // }

    // Ok(())
}
