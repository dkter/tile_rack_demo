#![allow(unused_variables)]

use ggez::graphics::Color;
use ggez::graphics::Drawable;
use mint::Point2;
use itertools::Itertools;

const TILE_WIDTH: f32 = 50.0;
const TILE_HEIGHT: f32 = 50.0;
const TILE_SPACING: f32 = 10.0;
const TILE_COLOUR: Color = Color::new(0.9, 0.9, 0.9, 1.0);
const ANIMATION_STEPS: i32 = 100;

struct Tile {
    x: f32,
    y: f32,
    letter: char,
    blend_mode: Option<ggez::graphics::BlendMode>,
    dragging: bool,
    relative_x_click: Option<f32>,
    relative_y_click: Option<f32>,
    animation_progress: i32,
    x_animation_step: Option<f32>,
    y_animation_step: Option<f32>,
}

impl Tile {
    fn new(x: f32, y: f32, letter: char) -> Tile {
        Tile {
            x: x,
            y: y,
            letter: letter,
            blend_mode: None,
            dragging: false,
            relative_x_click: None,
            relative_y_click: None,
            animation_progress: 0,
            x_animation_step: None,
            y_animation_step: None,
        }
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

impl ggez::graphics::Drawable for Tile {
    fn draw(
        &self,
        ctx: &mut ggez::Context,
        param: ggez::graphics::DrawParam,
    ) -> ggez::GameResult {
        let rect = ggez::graphics::Rect::new(
            self.x, self.y,
            TILE_WIDTH, TILE_HEIGHT,
        );
        let rect_drawable = ggez::graphics::Mesh::new_rectangle(
            ctx,
            ggez::graphics::DrawMode::fill(),
            rect,
            TILE_COLOUR,
        )?;
        ggez::graphics::draw(ctx, &rect_drawable, ggez::graphics::DrawParam::default())?;

        let font = ggez::graphics::Font::default();
        let text = ggez::graphics::Text::new((self.letter, font, 24.0));
        let text_dimensions = text.dimensions(ctx);
        let point = Point2 {
            x: self.x + (TILE_WIDTH / 2.0) - (text_dimensions.w / 2.0),
            y: self.y + (TILE_HEIGHT / 2.0) - (text_dimensions.h / 2.0),
        };
        ggez::graphics::draw(ctx, &text, (point, Color::BLACK))?;
        Ok(())
    }

    fn dimensions(&self, ctx: &mut ggez::Context) -> Option<ggez::graphics::Rect> {
        Some(
            ggez::graphics::Rect::new(
                self.x,
                self.y,
                TILE_WIDTH,
                TILE_HEIGHT,
            )
        )
    }

    fn set_blend_mode(&mut self, mode: Option<ggez::graphics::BlendMode>) {
        self.blend_mode = mode;
    }

    fn blend_mode(&self) -> Option<ggez::graphics::BlendMode> {
        self.blend_mode
    }
}

struct TileRack {
    x: f32,
    y: f32,
    tiles: Vec<Tile>,
    size: usize,
    blend_mode: Option<ggez::graphics::BlendMode>,
}

impl TileRack {
    fn new(x: f32, y: f32, letters: &str) -> TileRack {
        let mut tiles: Vec<Tile> = Vec::with_capacity(letters.len());
        for (index, letter) in letters.chars().enumerate() {
            let tile_x = x + (index as f32) * (TILE_WIDTH + TILE_SPACING);
            let tile_y = y;
            tiles.push(Tile::new(tile_x, tile_y, letter));
        }

        TileRack {
            x: x,
            y: y,
            tiles: tiles,
            size: letters.len(),
            blend_mode: None,
        }
    }

    fn get_dragging_tile(&self) -> Option<(usize, &Tile)> {
        self.tiles.iter().enumerate().filter(
            |(index, tile)| tile.dragging
        ).next()
    }

    fn get_dragging_tile_mut(&mut self) -> Option<(usize, &mut Tile)> {
        self.tiles.iter_mut().enumerate().filter(
            |(index, tile)| tile.dragging
        ).next()
    }

    fn get_new_tile_index(&self, x: f32) -> usize {
        let tile_position = (x - self.x + (TILE_WIDTH / 2.0)) / (TILE_WIDTH + TILE_SPACING);
        if tile_position < 0.0 {
            0 as usize
        } else if tile_position > (self.size - 1) as f32 {
            self.size - 1
        } else {
            tile_position as usize
        }
    }

    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        // rust
        let maybe_dragging_index_x = if let Some((dragging_index, dragging_tile)) = self.get_dragging_tile() {
            Some((dragging_index, dragging_tile.x))
        } else {
            None
        };

        let new_tile_x_positions: Vec<f32> = (0..self.size).map(
            |index| {
                let mut tile_x = self.x + (index as f32) * (TILE_WIDTH + TILE_SPACING);
                if let Some((dragging_initial_index, dragging_x)) = maybe_dragging_index_x {
                    // new_index is the index that the tile would get if it were to be dropped now
                    let new_index = self.get_new_tile_index(dragging_x);
                    if new_index <= index && index <= dragging_initial_index {
                        tile_x += TILE_WIDTH + TILE_SPACING;
                    }
                    else if dragging_initial_index <= index && index <= new_index {
                        tile_x -= TILE_WIDTH + TILE_SPACING;
                    }
                }
                tile_x
            }
        ).collect();

        for (tile, new_x) in self.tiles.iter_mut().zip(new_tile_x_positions) {
            if !tile.dragging {
                let new_y = self.y;

                let (anim_x, anim_y) = if ANIMATION_STEPS != 0 {
                    if tile.x == new_x && tile.y == new_y || tile.animation_progress >= ANIMATION_STEPS {
                        tile.x_animation_step = None;
                        tile.y_animation_step = None;
                        tile.animation_progress = 0;
                        (new_x, new_y)
                    }
                    else {
                        let x_animation_step = match tile.x_animation_step {
                            Some(s) => s,
                            None => (new_x - tile.x) / ANIMATION_STEPS as f32,
                        };
                        let y_animation_step = match tile.y_animation_step {
                            Some(s) => s,
                            None => (new_y - tile.y) / ANIMATION_STEPS as f32,
                        };
                        tile.x_animation_step = Some(x_animation_step);
                        tile.y_animation_step = Some(y_animation_step);
                        tile.animation_progress += 1;
                        (tile.x + x_animation_step, tile.y + y_animation_step)
                    }
                } else {
                    (new_x, new_y)
                };

                tile.set_pos(anim_x, anim_y);
            }
        }
        Ok(())
    }
}

impl ggez::graphics::Drawable for TileRack {
    fn draw(
        &self,
        ctx: &mut ggez::Context,
        param: ggez::graphics::DrawParam,
    ) -> ggez::GameResult {
        // Sort by t.dragging to make sure the tile being dragged gets drawn last (i.e. on top)
        for tile in self.tiles.iter().sorted_by_key(|t| t.dragging) {
            ggez::graphics::draw(ctx, tile, ggez::graphics::DrawParam::default())?;
        }
        Ok(())
    }

    fn dimensions(&self, ctx: &mut ggez::Context) -> Option<ggez::graphics::Rect> {
        Some(
            ggez::graphics::Rect::new(
                self.x,
                self.y,
                (TILE_WIDTH + TILE_SPACING) * self.size as f32,
                TILE_HEIGHT,
            )
        )
    }

    fn set_blend_mode(&mut self, mode: Option<ggez::graphics::BlendMode>) {
        self.blend_mode = mode;
    }

    fn blend_mode(&self) -> Option<ggez::graphics::BlendMode> {
        self.blend_mode
    }
}

struct State {
    rack: TileRack,
}

impl State {
    fn new(rack_x: f32, rack_y: f32, letters: &str) -> State {
        State {
            rack: TileRack::new(rack_x, rack_y, &letters),
        }
    }
}

impl ggez::event::EventHandler<ggez::GameError> for State {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        while ggez::timer::check_update_time(ctx, 500) {
            self.rack.update(ctx)?;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        ggez::graphics::clear(ctx, Color::WHITE);
        ggez::graphics::draw(ctx, &self.rack, ggez::graphics::DrawParam::default())?;
        ggez::graphics::present(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::input::mouse::MouseButton,
        x: f32,
        y: f32,
    ) {
        if button == ggez::input::mouse::MouseButton::Left {
            let click_point = Point2{x, y};
            // Approximate tile position (doesn't take into account y position, spacing or
            // coordinates to the left/right of the tile rack)
            let tile_position = ((x - self.rack.x) / (TILE_WIDTH + TILE_SPACING)) as usize;
            if tile_position <= self.rack.size - 1 {
                let tile = &mut self.rack.tiles[tile_position];
                // Check if mouse event was actually within the bounds of the tile
                if let Some(tile_bounds) = tile.dimensions(ctx) {
                    if tile_bounds.contains(click_point) {
                        tile.dragging = true;
                        tile.relative_x_click = Some(x - tile.x);
                        tile.relative_y_click = Some(y - tile.y);
                    }
                }
            }
        }
    }

    fn mouse_motion_event(
        &mut self,
        ctx: &mut ggez::Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) {
        for (index, tile) in self.rack.tiles.iter_mut().enumerate() {
            if tile.dragging {
                let tile_x = x - tile.relative_x_click.unwrap();
                let tile_y = y - tile.relative_y_click.unwrap();
                tile.set_pos(tile_x, tile_y);
            }
        }
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut ggez::Context,
        button: ggez::input::mouse::MouseButton,
        x: f32,
        y: f32,
    ) {
        if button == ggez::input::mouse::MouseButton::Left {
            // assume there is only one tile being dragged
            let maybe_dragging_index_x = if let Some((index, tile)) = self.rack.get_dragging_tile_mut() {
                tile.dragging = false;
                Some((index, tile.x))
            } else {
                None
            };
            if let Some((index, tile_x)) = maybe_dragging_index_x {
                let new_index = self.rack.get_new_tile_index(tile_x);
                let tile_deref = self.rack.tiles.remove(index);
                self.rack.tiles.insert(new_index, tile_deref);
            }
        }
    }
}

fn main() {
    let conf = ggez::conf::Conf {
        window_setup: ggez::conf::WindowSetup {
            title: "Tile rack demo".to_owned(),
            samples: ggez::conf::NumSamples::One,
            vsync: true,
            icon: "".to_owned(),
            srgb: true,
        },
        window_mode: ggez::conf::WindowMode::default(),
        backend: ggez::conf::Backend::default(),
        modules: ggez::conf::ModuleConf::default(),
    };
    let window_width = conf.window_mode.width;
    let window_height = conf.window_mode.height;
    let (ctx, event_loop) = ggez::ContextBuilder::new("tile_rack_demo", "david")
        .default_conf(conf)
        .build()
        .unwrap();

    let rack_width = (TILE_WIDTH + TILE_SPACING) * 7.0 - TILE_SPACING;
    let rack_height = TILE_HEIGHT;

    let state = State::new(
        window_width / 2.0 - rack_width / 2.0,
        window_height / 2.0 - rack_height / 2.0,
        &"AEINRST",
    );
    ggez::event::run(ctx, event_loop, state);
}
