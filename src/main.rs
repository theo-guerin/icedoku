mod puzzle;
mod widget;

use iced::{Element, Size, Theme, window};

use widget::grid;

use crate::{
    puzzle::{Difficulty, get_random_puzzle},
    widget::centered_square,
};

fn main() -> iced::Result {
    let window_settings = window::Settings {
        min_size: Some(IceDoku::MIN_WINDOW_SIZE),
        ..Default::default()
    };

    iced::application(IceDoku::new, IceDoku::update, IceDoku::view)
        .title(IceDoku::TITLE)
        .theme(IceDoku::THEME)
        .window(window_settings)
        .run()
}

#[derive(Debug)]
struct IceDoku {
    grid: grid::State,
}

#[derive(Debug, Clone)]
enum Message {
    GridEdited(grid::Action),
}

impl IceDoku {
    const TITLE: &'static str = "IceDoku";
    const THEME: Theme = Theme::Dark;
    const MIN_WINDOW_SIZE: Size = Size::new(400.0, 400.0);

    fn new() -> Self {
        let puzzle = get_random_puzzle(Difficulty::Easy);

        let mut cells = [[grid::CellValue::Empty; grid::SIZE]; grid::SIZE];
        for (x, row) in puzzle.clues.iter().enumerate() {
            for (y, &value) in row.iter().enumerate() {
                if let Some(value) = value {
                    cells[x][y] = grid::CellValue::Clue(value);
                }
            }
        }

        Self {
            grid: grid::State::new(cells),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GridEdited(action) => self.grid.perform(action),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        centered_square(grid(&self.grid).on_edit(Message::GridEdited)).into()
    }
}
