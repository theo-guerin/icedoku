mod puzzle;
mod widget;

use iced::{Element, Length, Size, Theme, widget::container, window};

use crate::{
    puzzle::{Difficulty, get_random_puzzle},
    widget::{aspect_ratio, grid},
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
        let puzzle = get_random_puzzle(Difficulty::Simple);

        let mut cells = [[grid::CellValue::Empty; puzzle::GRID_DIMENSION]; puzzle::GRID_DIMENSION];
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
        container(aspect_ratio(
            1.0,
            grid(&self.grid).on_edit(Message::GridEdited),
        ))
        .center(Length::Fill)
        .into()
    }
}
