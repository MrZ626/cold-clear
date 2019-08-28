use ggez::{ Context, GameResult };
use ggez::graphics::{ self, Color, DrawParam, Rect, Image };
use libtetris::*;
use arrayvec::ArrayVec;

pub struct BoardDrawState {
    board: ArrayVec<[ColoredRow; 40]>,
    state: State,
    pub statistics: Statistics
}

enum State {
    Falling(FallingPiece, FallingPiece),
    LineClearAnimation(ArrayVec<[i32; 4]>, i32),
    Delay
}

impl BoardDrawState {
    pub fn new() -> Self {
        BoardDrawState {
            board: ArrayVec::from([*ColoredRow::EMPTY; 40]),
            state: State::Delay,
            statistics: Statistics::default()
        }
    }

    pub fn update(&mut self, events: &[Event]) {
        if let State::LineClearAnimation(_, ref mut frames) = self.state {
            *frames += 1;
        }
        for event in events {
            match event {
                Event::PiecePlaced { piece, locked, .. } => {
                    for (x, y) in piece.cells() {
                        self.board[y as usize].set(x as usize, piece.kind.0.color());
                    }
                    if locked.cleared_lines.is_empty() {
                        self.state = State::Delay;
                    } else {
                        self.state = State::LineClearAnimation(locked.cleared_lines.clone(), 0);
                    }
                }
                Event::PieceFalling(piece, ghost) => {
                    self.state = State::Falling(*piece, *ghost);
                }
                Event::EndOfLineClearDelay => {
                    self.state = State::Delay;
                    self.board.retain(|row| !row.is_full());
                    while !self.board.is_full() {
                        self.board.push(*ColoredRow::EMPTY);
                    }
                }
                Event::GarbageAdded(columns) => {
                    self.board.truncate(40 - columns.len());
                    for &col in columns {
                        let mut row = *ColoredRow::EMPTY;
                        for x in 0..10 {
                            if x != col {
                                row.set(x, CellColor::Garbage);
                            }
                        }
                        self.board.insert(0, row);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn draw(&self, ctx: &mut Context, img: &Image) -> GameResult {
        for y in 0..21 {
            for x in 0..10 {
                graphics::draw(ctx, img, DrawParam::new()
                    .dest([x as f32, (20-y) as f32 - 0.75])
                    .src(if self.board[y].cell_color(x) == CellColor::Empty
                        { tile(0, 0) } else { tile(1, 0) })
                    .color(cell_color_to_color(self.board[y].cell_color(x)))
                    .scale([SPRITE_SCALE, SPRITE_SCALE]))?;
            }
        }
        match self.state {
            State::Falling(piece, ghost) => {
                for (x,y) in ghost.cells() {
                    graphics::draw(ctx, img, draw_tile(
                        x, y, 2, 0, cell_color_to_color(piece.kind.0.color())
                    ))?;
                }
                for (x,y) in piece.cells() {
                    graphics::draw(ctx, img, draw_tile(
                        x, y, 1, 0, cell_color_to_color(piece.kind.0.color())
                    ))?;
                }
            }
            State::LineClearAnimation(ref lines, frame) => {
                let frame_x = frame.min(35) / 12;
                let frame_y = frame.min(35) % 12;
                for &y in lines {
                    graphics::draw(ctx, img, draw_tile(
                        0, y, frame_x*3+3, frame_y, graphics::WHITE
                    ))?;
                    graphics::draw(ctx, img, draw_tile(
                        9, y, frame_x*3+5, frame_y, graphics::WHITE
                    ))?;
                    for x in 1..9 {
                        graphics::draw(ctx, img, draw_tile(
                            x, y, frame_x*3+4, frame_y, graphics::WHITE
                        ))?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn cell_color_to_color(cell_color: CellColor) -> Color {
    match cell_color {
        CellColor::Empty => graphics::WHITE,
        CellColor::Garbage => Color::from_rgb(160, 160, 160),
        CellColor::Unclearable => Color::from_rgb(64, 64, 64),
        CellColor::Z => Color::from_rgb(255, 32, 32),
        CellColor::S => Color::from_rgb(32, 255, 32),
        CellColor::O => Color::from_rgb(255, 255, 32),
        CellColor::L => Color::from_rgb(255, 143, 32),
        CellColor::J => Color::from_rgb(96, 96, 255),
        CellColor::I => Color::from_rgb(32, 255, 255),
        CellColor::T => Color::from_rgb(143, 32, 255)
    }
}

fn tile(x: i32, y: i32) -> Rect {
    Rect {
        x: x as f32 * (85.0/1024.0) + 1.0/1024.0,
        y: y as f32 * (85.0/1024.0) + 1.0/1024.0,
        h: 83.0/1024.0,
        w: 83.0/1024.0
    }
}

fn draw_tile(x: i32, y: i32, tx: i32, ty: i32, color: Color) -> DrawParam {
    DrawParam::new()
        .dest([x as f32, (20-y) as f32 - 0.75])
        .src(tile(tx, ty))
        .color(color)
        .scale([SPRITE_SCALE, SPRITE_SCALE])
}

const SPRITE_SCALE: f32 = 1.0/83.0;
