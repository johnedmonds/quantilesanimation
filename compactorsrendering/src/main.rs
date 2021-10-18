use compactorsrenderingcommon::{compactor_mesh, max_frame_size};
use rand::{prelude::StdRng, Rng, SeedableRng};
use rand_distr::Distribution;
use renderingcommon::{save_frame, Element};

use ggez::{
    conf::{NumSamples, WindowMode},
    event::EventHandler,
    graphics::{self, get_window_color_format, Canvas, Color, DrawParam, Rect},
    GameError, GameResult,
};

use compactorsanim::{compactor::Compactor, compactors::Compactors};

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
        let mesh = compactor_mesh(ctx, &self.current_frame)?;
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

fn make_frames<R: Rng, D: Distribution<f32>, const LAZY: bool>(
    d: D,
    mut r: R,
) -> Vec<Vec<Compactor<Element>>> {
    let mut frames: Vec<Vec<Compactor<Element>>> = Vec::new();
    let mut compactors: Compactors<Element, _, LAZY> =
        Compactors::new(10, |frame| frames.push(frame));
    for _i in 1..100 {
        compactors.update(d.sample(&mut r) as u32);
    }
    frames
}

fn main() -> GameResult {
    let frames =
        make_frames::<_, _, true>(*renderingcommon::DISTRIBUTION, StdRng::seed_from_u64(42));
    let (mut ctx, _event_loop) = ggez::ContextBuilder::new("quantiles", "jedmonds").build()?;
    let _mode = WindowMode::default();
    let (w, h) = max_frame_size(frames.iter());
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
