mod new_game;
mod puzzle;
mod widget;

use iced::{
    Center, Element, Fill, Size, Theme,
    widget::{button, column, container, row, space, text},
    window,
};

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
    mode: Mode,
    puzzle_grid: puzzle_grid::State,
    difficulty: Difficulty,
}

#[derive(Debug)]
enum Mode {
    Playing,
    CreatingNewGame(new_game::NewGame),
}

#[derive(Debug, Clone)]
enum Message {
    PuzzleGrid(puzzle_grid::Action),
    NewGame(new_game::Message),
    NewGameRequested,
}

impl IceDoku {
    const TITLE: &'static str = "IceDoku";
    const THEME: Theme = Theme::Dark;
    const MIN_WINDOW_SIZE: Size = Size::new(400.0, 400.0);

    fn new() -> Self {
        let difficulty = INITIAL_DIFFICULTY;
        let puzzle = get_random_puzzle(difficulty);

        Self {
            mode: Mode::Playing,
            puzzle_grid: puzzle_grid::State::from(&puzzle),
            difficulty,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PuzzleGrid(action) => self.puzzle_grid.perform(action),
            Message::NewGameRequested => {
                self.mode = Mode::CreatingNewGame(new_game::NewGame::new(self.difficulty));
            }
            Message::NewGame(message) => {
                let Mode::CreatingNewGame(new_game) = &mut self.mode else {
                    return;
                };

                let Some(action) = new_game.update(message) else {
                    return;
                };

                match action {
                    new_game::Action::Close => {
                        self.mode = Mode::Playing;
                    }
                    new_game::Action::Start(difficulty) => {
                        self.start_new_game(difficulty);
                        self.mode = Mode::Playing;
                    }
                }
            }
        }
    }

    fn start_new_game(&mut self, difficulty: Difficulty) {
        let puzzle = get_random_puzzle(difficulty);
        self.puzzle_grid = puzzle_grid::State::from(&puzzle);
        self.difficulty = difficulty;
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.mode {
            Mode::Playing => {
                let header = row![
                    self.header_title(),
                    space::horizontal(),
                    button("New game")
                        .on_press(Message::NewGameRequested)
                        .padding([10, 16]),
                ]
                .align_y(Center);

                container(column![header, container(self.board()).center(Fill)].spacing(20))
                    .padding(20)
                    .center(Fill)
                    .into()
            }
            Mode::CreatingNewGame(new_game) => {
                let setup = container(new_game.view().map(Message::NewGame))
                    .max_width(520)
                    .width(Fill);

                container(setup).padding(24).center(Fill).into()
            }
        }
    }

    fn header_title(&self) -> Element<'_, Message> {
        column![
            text("ICEDOKU").size(26),
            text(format!("{} puzzle", self.difficulty)).size(13),
        ]
        .spacing(2)
        .into()
    }

    fn board(&self) -> Element<'_, Message> {
        aspect_ratio(
            1.0,
            puzzle_grid(&self.puzzle_grid).on_action(Message::PuzzleGrid),
        )
        .into()
    }
}
