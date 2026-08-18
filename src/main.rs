mod puzzle;
mod widget;

use iced::{
    Alignment, Element, Length, Size, Theme,
    widget::{column, container, pick_list},
    window,
};
use strum::VariantArray;

use crate::{
    puzzle::{Difficulty, get_random_puzzle},
    widget::{aspect_ratio, puzzle_grid},
};

const INITIAL_DIFFICULTY: Difficulty = Difficulty::Simple;

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
    puzzle_grid: puzzle_grid::State,
    difficulty: Option<Difficulty>,
}

#[derive(Debug, Clone)]
enum Message {
    GridEdited(puzzle_grid::Action),
    DifficultySelected(Difficulty),
}

impl IceDoku {
    const TITLE: &'static str = "IceDoku";
    const THEME: Theme = Theme::Dark;
    const MIN_WINDOW_SIZE: Size = Size::new(400.0, 400.0);

    fn new() -> Self {
        let difficulty = INITIAL_DIFFICULTY;
        let puzzle = get_random_puzzle(difficulty);

        Self {
            puzzle_grid: puzzle_grid::State::from(&puzzle),
            difficulty: Some(difficulty),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::GridEdited(action) => self.puzzle_grid.perform(action),
            Message::DifficultySelected(difficulty) => {
                self.difficulty = Some(difficulty);
                let new_puzzle = get_random_puzzle(difficulty);
                self.puzzle_grid = puzzle_grid::State::from(&new_puzzle);
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            pick_list(
                Difficulty::VARIANTS,
                self.difficulty,
                Message::DifficultySelected
            ),
            container(aspect_ratio(
                1.0,
                puzzle_grid(&self.puzzle_grid).on_action(Message::GridEdited),
            ))
            .center(Length::Fill)
        ]
        .align_x(Alignment::Center)
        .into()
    }
}
