use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::seq::IndexedRandom;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = Game::default().run(terminal);
    ratatui::restore();
    result
}

#[derive(Debug)]
pub struct Game {
    snake: Vec<(u16, u16)>,
    snake_direction: Direction,
    apple_position: (u16, u16),
    snake_move_time: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            snake: vec![(Self::BOARD_SIZE / 2, Self::BOARD_SIZE / 2)],
            snake_direction: Direction::Right,
            apple_position: (Self::BOARD_SIZE / 2, Self::BOARD_SIZE / 3),
            snake_move_time: 200,
        }
    }
}

impl Game {
    const BOARD_SIZE: u16 = 20;

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        let mut now = Instant::now();

        'render: loop {
            while event::poll(Duration::ZERO).is_ok_and(|available| available) {
                match event::read()? {
                    Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                        match key_event.code {
                            KeyCode::Up | KeyCode::Char('w') => {
                                if self.is_valid_turn(Direction::Up) {
                                    self.snake_direction = Direction::Up
                                }
                            }
                            KeyCode::Down | KeyCode::Char('s') => {
                                if self.is_valid_turn(Direction::Down) {
                                    self.snake_direction = Direction::Down
                                }
                            }
                            KeyCode::Left | KeyCode::Char('a') => {
                                if self.is_valid_turn(Direction::Left) {
                                    self.snake_direction = Direction::Left
                                }
                            }
                            KeyCode::Right | KeyCode::Char('d') => {
                                if self.is_valid_turn(Direction::Right) {
                                    self.snake_direction = Direction::Right
                                }
                            }
                            KeyCode::Char('q') => break 'render Ok(()),
                            _ => (),
                        }
                    }
                    _ => {}
                };
            }

            if now.elapsed() > Duration::from_millis(self.snake_move_time) {
                let direction = self.snake_direction.get_vec2();
                let head = self.snake[0];

                // Snake hit border
                if (head.0 == 0 && direction.0 < 0)
                    || (head.1 == 0 && direction.1 < 0)
                    || (head.0 == Self::BOARD_SIZE - 1 && direction.0 > 0)
                    || (head.1 == Self::BOARD_SIZE - 1 && direction.1 > 0)
                {
                    break Ok(());
                }

                self.snake.pop();

                let next_head = (
                    head.0.saturating_add_signed(direction.0),
                    head.1.saturating_add_signed(direction.1),
                );

                // Snake hit itself
                if self.snake.contains(&next_head) {
                    break Ok(());
                }

                self.snake.insert(0, next_head);

                now = Instant::now();
            }

            if self.snake[0] == self.apple_position {
                let tail_direction = if self.snake.len() > 1 {
                    let (x1, y1) = self.snake[self.snake.len() - 1];
                    let (x2, y2) = self.snake[self.snake.len() - 2];
                    (x1 as i16 - x2 as i16, y1 as i16 - y2 as i16)
                } else {
                    let (x, y) = self.snake_direction.get_vec2();
                    (-x, -y)
                };
                let tail = self.snake[self.snake.len() - 1];

                self.snake.push((
                    tail.0.saturating_add_signed(tail_direction.0),
                    tail.1.saturating_add_signed(tail_direction.1),
                ));

                let mut possible_positions =
                    Vec::with_capacity(Self::BOARD_SIZE as usize * Self::BOARD_SIZE as usize);

                for x in 0..Self::BOARD_SIZE {
                    for y in 0..Self::BOARD_SIZE {
                        if !self.snake.contains(&(x, y)) {
                            possible_positions.push((x, y));
                        }
                    }
                }

                self.apple_position = *possible_positions.choose(&mut rand::rng()).unwrap();
                self.snake_move_time = (self.snake_move_time - 10).max(50);
            }

            terminal.draw(|frame| self.draw(frame))?;
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn is_valid_turn(&self, direction: Direction) -> bool {
        if self.snake.len() > 1 {
            let direction = direction.get_vec2();
            let head = self.snake[0];

            (
                head.0.saturating_add_signed(direction.0),
                head.1.saturating_add_signed(direction.1),
            ) != self.snake[1]
        } else {
            self.snake_direction.opposite() != direction
        }
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        assert!(
            area.width > (Game::BOARD_SIZE + 2) * 2 && area.height > Game::BOARD_SIZE + 2,
            "The terminal window must be at least {}x{} characters big",
            (Game::BOARD_SIZE + 2) * 2,
            Game::BOARD_SIZE + 2
        );

        // Roughly 1x2 terminal character size
        let board_rect = Rect::new(
            area.x + ((area.width / 2).saturating_sub(Game::BOARD_SIZE)),
            area.y + ((area.height / 2).saturating_sub(Game::BOARD_SIZE / 2)),
            (Game::BOARD_SIZE - 1) * 2,
            Game::BOARD_SIZE,
        );

        let border_rect = Rect::new(
            board_rect.x - 1,
            board_rect.y - 1,
            board_rect.width + 4,
            board_rect.height + 2,
        );

        Block::bordered()
            .border_type(ratatui::widgets::BorderType::Plain)
            .border_set(border::THICK)
            .title(Line::from(format!(" Score: {} ", self.snake.len() - 1)).centered())
            .render(border_rect, buf);

        let (x, y) = self.apple_position;
        buf[((x * 2) + board_rect.x, y + board_rect.y)].set_symbol("##");

        for (x, y) in &self.snake {
            buf[((x * 2) + board_rect.x, y + board_rect.y)].set_symbol("██");
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn get_vec2(&self) -> (i16, i16) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
