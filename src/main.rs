mod widget;

use iced::{
    Border, Color, Element, Length, Size, Subscription, Theme,
    keyboard::{self, Key, key::Named},
    widget::{Column, Grid, Row, button, container, responsive, text},
    window,
};

use crate::widget::centered_square::centered_square;

fn main() -> iced::Result {
    let window_settings = window::Settings {
        min_size: Some(IceDoku::MIN_WINDOW_SIZE),
        ..Default::default()
    };

    iced::application(IceDoku::new, IceDoku::update, IceDoku::view)
        .title(IceDoku::TITLE)
        .theme(IceDoku::THEME)
        .subscription(IceDoku::subscription)
        .window(window_settings)
        .run()
}

#[derive(Debug)]
struct IceDoku {
    cells: [[Cell; 9]; 9],
    selected_cell: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Copy)]
enum Cell {
    Empty,
    Fixed(u8),
    Filled(u8),
}

#[derive(Debug, Clone)]
enum Message {
    CellPressed { row: usize, column: usize },
    NumberPressed(u8),
    EscapePressed,
}

impl IceDoku {
    const TITLE: &'static str = "IceDoku";
    const THEME: Theme = Theme::Dark;
    const MIN_WINDOW_SIZE: Size = Size::new(400.0, 400.0);

    const GRID_SIZE: usize = 9;
    const SUBGRID_SIZE: usize = 3;

    fn new() -> Self {
        let mut cells = [[Cell::Empty; 9]; 9];
        for row in &mut cells {
            for cell in row {
                if rand::random::<f32>() < 0.2 {
                    *cell = Cell::Fixed(rand::random_range::<u8, _>(1..=9));
                }
            }
        }

        Self {
            cells,
            selected_cell: None,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::CellPressed { row, column } => {
                self.selected_cell = Some((row, column));
            }
            Message::NumberPressed(value) => {
                if let Some((row, column)) = self.selected_cell {
                    let cell = &mut self.cells[row][column];
                    if !matches!(cell, Cell::Fixed(_)) {
                        *cell = Cell::Filled(value);
                    }
                }
            }
            Message::EscapePressed => {
                self.selected_cell = None;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        centered_square(|| {
            let mut grid = Column::with_capacity(Self::GRID_SIZE);

            for (row_index, row_data) in self.cells.iter().enumerate() {
                let mut row = Row::with_capacity(Self::GRID_SIZE);

                for (column_index, &cell) in row_data.iter().enumerate() {
                    let is_fixed = matches!(cell, Cell::Fixed(_));

                    let background_color = match self.selected_cell {
                        Some((selected_row, selected_column))
                            if selected_row == row_index && selected_column == column_index =>
                        {
                            if is_fixed {
                                Color::from_rgb(0.35, 0.62, 0.95)
                            } else {
                                Color::from_rgb(0.52, 0.76, 0.98)
                            }
                        }
                        Some((selected_row, selected_column))
                            if selected_row == row_index || selected_column == column_index =>
                        {
                            if is_fixed {
                                Color::from_rgb(0.70, 0.82, 0.95)
                            } else {
                                Color::from_rgb(0.80, 0.90, 0.98)
                            }
                        }
                        Some((selected_row, selected_column))
                            if selected_row / Self::SUBGRID_SIZE
                                == row_index / Self::SUBGRID_SIZE
                                && selected_column / Self::SUBGRID_SIZE
                                    == column_index / Self::SUBGRID_SIZE =>
                        {
                            if is_fixed {
                                Color::from_rgb(0.70, 0.82, 0.95)
                            } else {
                                Color::from_rgb(0.80, 0.90, 0.98)
                            }
                        }
                        _ => {
                            if is_fixed {
                                Color::from_rgb(0.85, 0.88, 0.93)
                            } else {
                                Color::from_rgb(0.94, 0.96, 0.98)
                            }
                        }
                    };

                    row = row.push(
                        button(responsive(move |size| {
                            let font_size = (size.width.min(size.height) * 0.4).clamp(12.0, 96.0);

                            let text_el = text(match cell {
                                Cell::Fixed(value) | Cell::Filled(value) => value.to_string(),
                                Cell::Empty => String::new(),
                            })
                            .color(Color::from_rgb(0.2, 0.2, 0.2))
                            .size(font_size);

                            container(text_el)
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .center(Length::Fill)
                                .into()
                        }))
                        .on_press_with(move || Message::CellPressed {
                            row: row_index,
                            column: column_index,
                        })
                        .style(move |_theme, _state| button::Style {
                            background: Some(background_color.into()),
                            border: Border::default()
                                .color(Color::from_rgb(0.4, 0.4, 0.4))
                                .width(1),
                            ..Default::default()
                        }),
                    );
                }
                grid = grid.push(row);
            }
            grid.into()
        })
    }

    fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                key: Key::Named(Named::Escape),
                ..
            } => Some(Message::EscapePressed),
            keyboard::Event::KeyPressed {
                text: Some(text), ..
            } => match text.as_bytes() {
                [byte @ b'1'..=b'9'] => Some(Message::NumberPressed(byte - b'0')),
                _ => None,
            },
            _ => None,
        })
    }
}
