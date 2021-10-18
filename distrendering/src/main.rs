use std::{cmp::max};

use compactorsanim::compactors::Compactors;
use distanim::estimated_quantiles::{EstimatedQuantiles, QuantileElement};
use ggez::{
    conf::{NumSamples, WindowMode},
    event::EventHandler,
    graphics::{
        self, get_window_color_format, Canvas, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect,
        Transform,
    },
    mint,
    Context, GameError, GameResult,
};


use rand::{prelude::StdRng, SeedableRng};
use rand_distr::Distribution;
use renderingcommon::{DISTRIBUTION, DISTRIBUTION_MIN, DISTRIBUTION_PRACTICAL_RANGE, ELEMENT_WIDTH, Element, MAX_ELEMENT_HEIGHT, SPACE_BETWEEN_ELEMENTS, save_frame};

struct MainState<const BUCKETS: usize> {
    elements: Vec<QuantileElement<Element>>,
    estimated_element_count: usize,
    elements_seen: usize,
    buckets: [Vec<QuantileElement<Element>>; BUCKETS],
}

fn color(element: Element) -> Color {
    let value: f32 = (element as f32 - *DISTRIBUTION_MIN) / *DISTRIBUTION_PRACTICAL_RANGE;
    Color {
        r: value,
        g: value,
        b: value,
        a: 1.0,
    }
}

impl<const BUCKETS: usize> MainState<BUCKETS> {
    fn elements_size(&self) -> (u32, u32) {
        (
            self.elements.len() as u32 * (ELEMENT_WIDTH + SPACE_BETWEEN_ELEMENTS) as u32,
            MAX_ELEMENT_HEIGHT as u32,
        )
    }
    fn buckets_size(&self) -> (u32, u32) {
        (
            BUCKETS as u32 * (ELEMENT_WIDTH + SPACE_BETWEEN_ELEMENTS),
            self.elements.iter().map(|e| e.weight).sum::<usize>() as u32,
        )
    }
    fn step(&mut self) {
        if !self.elements.is_empty() {
            let current_element = self.elements.remove(0);
            let current_bucket = self.elements_seen / (self.estimated_element_count / BUCKETS);
            println!(
                "{}/({}/{}) = {}",
                self.elements_seen, self.estimated_element_count, BUCKETS, current_bucket
            );
            let current_bucket = if current_bucket == BUCKETS {
                current_bucket - 1
            } else {
                current_bucket
            };
            self.elements_seen += current_element.weight;
            self.buckets[current_bucket].push(current_element);
        }
        println!("{:?}", self.buckets);
        println!("{:?}", self.elements);
    }
    fn render_elements(&self, ctx: &mut Context) -> Result<Option<Mesh>, GameError> {
        if self.elements.is_empty() {
            Ok(None)
        } else {
            let mut mesh_builder = MeshBuilder::new();
            for (i, element) in self.elements.iter().enumerate() {
                mesh_builder.rectangle(
                    DrawMode::fill(),
                    Rect::new_i32(
                        (i as u32 * (ELEMENT_WIDTH + SPACE_BETWEEN_ELEMENTS)) as i32,
                        0,
                        ELEMENT_WIDTH as i32,
                        element.weight as i32,
                    ),
                    color(element.element),
                )?;
            }
            mesh_builder.build(ctx).map(Some)
        }
    }
    fn render_bucket(&self, ctx: &mut Context) -> Result<Option<Mesh>, GameError> {
        if self.buckets.iter().all(Vec::is_empty) {
            Ok(None)
        } else {
            let mut mesh_builder = MeshBuilder::new();
            for (i, bucket) in self.buckets.iter().enumerate() {
                let mut y = 0;
                for element in bucket {
                    mesh_builder.rectangle(
                        DrawMode::fill(),
                        Rect::new_i32(
                            (i as u32 * (ELEMENT_WIDTH + SPACE_BETWEEN_ELEMENTS)) as i32,
                            y as i32,
                            ELEMENT_WIDTH as i32,
                            element.weight as i32,
                        ),
                        color(element.element),
                    )?;
                    y += element.weight;
                }
            }
            mesh_builder.build(ctx).map(Some)
        }
    }
}

impl<const BUCKETS: usize> EventHandler<GameError> for MainState<BUCKETS> {
    fn update(&mut self, _ctx: &mut ggez::Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> Result<(), GameError> {
        graphics::clear(ctx, Color::BLACK);
        if let Some(mesh) = self.render_elements(ctx)? {
            graphics::draw(
                ctx,
                &mesh,
                DrawParam::default().transform(
                    Transform::Values {
                        dest: mint::Point2 { x: 0.0, y: 0.0 },
                        rotation: 0.0,
                        scale: mint::Vector2 { x: 1.0, y: 1.0 },
                        offset: mint::Point2 {
                            x: -(SPACE_BETWEEN_ELEMENTS as f32),
                            y: -(SPACE_BETWEEN_ELEMENTS as f32),
                        },
                    }
                    .to_bare_matrix(),
                ),
            )?;
        }
        if let Some(mesh) = self.render_bucket(ctx)? {
            graphics::draw(
                ctx,
                &mesh,
                DrawParam::default().transform(
                    Transform::Values {
                        dest: mint::Point2 { x: 0.0, y: 0.0 },
                        rotation: 0.0,
                        scale: mint::Vector2 { x: 1.0, y: 1.0 },
                        offset: mint::Point2 {
                            x: -(SPACE_BETWEEN_ELEMENTS as f32),
                            y: -((SPACE_BETWEEN_ELEMENTS * 2 + MAX_ELEMENT_HEIGHT) as f32),
                        },
                    }
                    .to_bare_matrix(),
                ),
            )?;
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

fn make_est<D: Distribution<f32>, const LAZY: bool>(d: D) -> EstimatedQuantiles<Element> {
    let mut compactors: Compactors<Element, _, LAZY> = Compactors::new(10, |_| {});
    let mut r = StdRng::seed_from_u64(42);
    for _i in 1..100 {
        compactors.update(d.sample(&mut r) as u32);
    }
    compactors.into()
}

fn main() -> GameResult {
    let est = make_est::<_, true>(*DISTRIBUTION);
    println!("{:?},{}", est.elements, est.elements.len());
    let (mut ctx, _event_loop) = ggez::ContextBuilder::new("quantiles", "jedmonds").build()?;
    let mut state: MainState<7> = MainState {
        buckets: Default::default(),
        estimated_element_count: est.estimated_element_count(),
        elements: est.elements,
        elements_seen: 0,
    };
    let (elements_width, elements_height) = state.elements_size();
    let (bucket_width, bucket_height) = state.buckets_size();
    let (w, h) = (
        max(elements_width, bucket_width),
        elements_height + bucket_height,
    );

    let mut mode = WindowMode::default();
    mode.width = w as f32;
    mode.height = h as f32;
    println!("{} {}", elements_width, elements_height);
    graphics::set_mode(&mut ctx, mode)?;
    graphics::set_screen_coordinates(&mut ctx, Rect::new_i32(0, 0, w as i32, h as i32))?;
    // event::run(ctx, _event_loop, state)

    let color_format = get_window_color_format(&ctx);
    let canvas = Canvas::new(&mut ctx, w as u16, h as u16, NumSamples::One, color_format)?;
    graphics::set_canvas(&mut ctx, Some(&canvas));
    let mut frame_id = 0;
    for _i in 0..state.elements.len() {
        graphics::set_canvas(&mut ctx, Some(&canvas));
        state.draw(&mut ctx)?;
        save_frame(canvas.to_rgba8(&mut ctx)?, frame_id, w, h);
        frame_id += 1;
        state.step();
    }
    Ok(())
}
