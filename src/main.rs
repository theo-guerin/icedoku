mod widget;

use iced::{Element, Size, Theme, window};

use widget::grid;

use crate::widget::centered_square;

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
        let mut cells = [[grid::CellValue::Empty; grid::SIZE]; grid::SIZE];
        for row in &mut cells {
            for cell in row {
                match rand::random::<f32>() {
                    0.0..0.4 => *cell = grid::CellValue::Clue(rand::random_range::<u8, _>(0..=9)),
                    0.4..0.6 => *cell = grid::CellValue::Filled(rand::random_range::<u8, _>(0..=9)),
                    _ => {}
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
