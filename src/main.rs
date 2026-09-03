mod game;
mod new_game;
mod puzzle;
mod widget;

use iced::{
    Center, Element, Fill, Size, Theme,
    widget::{button, column, container, row, text},
    window,
};

use crate::{
    game::{Game, MAX_MISTAKES, Status},
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
    game: Game,
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
            game: Game::new(puzzle),
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PuzzleGrid(action) => self.game.perform(action),
            Message::NewGameRequested => {
                self.mode = Mode::CreatingNewGame(new_game::NewGame::new(self.game.difficulty()));
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
        self.game = Game::new(puzzle);
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.mode {
            Mode::Playing => {
                let header = row![
                    container(self.header_title()).width(Fill),
                    container(self.mistake_counter()).center_x(Fill),
                    container(
                        button("New game")
                            .on_press(Message::NewGameRequested)
                            .padding([10, 16])
                    )
                    .align_right(Fill),
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
        let subtitle = match self.game.status() {
            Status::Playing => format!("{} puzzle", self.game.difficulty()),
            Status::Won => String::from("Puzzle solved"),
            Status::Lost => String::from("Game over"),
        };

        column![text("ICEDOKU").size(26), text(subtitle).size(13),]
            .spacing(2)
            .into()
    }

    fn mistake_counter(&self) -> Element<'_, Message> {
        column![
            text(format!("{} / {}", self.game.mistakes(), MAX_MISTAKES)).size(16),
            text("MISTAKES").size(10),
        ]
        .spacing(1)
        .align_x(Center)
        .into()
    }

    fn board(&self) -> Element<'_, Message> {
        let grid = puzzle_grid(self.game.grid(), self.game.solution());
        let grid = if self.game.status() == Status::Playing {
            grid.on_action(Message::PuzzleGrid)
        } else {
            grid
        };

        aspect_ratio(1.0, grid).into()
    }
}
