use iced::{
    Color, Element, Event, Length, Pixels, Rectangle, Size,
    advanced::{
        Clipboard, Shell,
        layout::{self, Layout},
        renderer,
        text::{self, Text},
        widget::{Widget, tree::Tree},
    },
    alignment, keyboard, mouse,
};

use crate::puzzle::{BOX_DIMENSION, GRID_DIMENSION};

const CELL_BORDER_WIDTH: f32 = 1.0;
const BOX_BORDER_WIDTH: f32 = 3.0;

const BACKGROUND_COLOR: Color = Color::from_rgb8(255, 255, 255);
const SELECTED_CELL_COLOR: Color = Color::from_rgb8(191, 219, 254);
const PEER_CELL_COLOR: Color = Color::from_rgb8(239, 246, 255);
const MATCHING_DIGIT_CELL_COLOR: Color = Color::from_rgb8(219, 234, 254);
const CLUE_DIGIT_COLOR: Color = Color::from_rgb8(0, 0, 0);
const ENTRY_DIGIT_COLOR: Color = Color::from_rgb8(100, 100, 140);
const LINE_COLOR: Color = Color::from_rgb8(0, 0, 0);

const DIGIT_SIZE_RATIO: f32 = 0.5;

#[allow(missing_debug_implementations)]
pub struct Grid<'a, Message> {
    state: &'a State,
    on_action: Option<Box<dyn Fn(Action) -> Message + 'a>>,
}

#[derive(Debug, Clone)]
pub enum Action {
    SelectCell {
        row: usize,
        column: usize,
    },
    ClearSelection,
    EnterDigit {
        row: usize,
        column: usize,
        digit: u8,
    },
}

#[derive(Debug)]
pub struct State {
    cells: [[CellValue; GRID_DIMENSION]; GRID_DIMENSION],
    selected_cell: Option<(usize, usize)>,
}

#[derive(Debug)]
struct Cell {
    row: usize,
    column: usize,
    value: CellValue,
}

impl Cell {
    pub fn digit(&self) -> Option<u8> {
        self.value.digit()
    }

    pub fn is_clue(&self) -> bool {
        self.value.is_clue()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellValue {
    Empty,
    Clue(u8),
    Entry(u8),
}

impl CellValue {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn digit(&self) -> Option<u8> {
        match self {
            CellValue::Empty => None,
            CellValue::Clue(value) | CellValue::Entry(value) => Some(*value),
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn is_clue(&self) -> bool {
        matches!(self, CellValue::Clue(_))
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    pub fn is_empty(&self) -> bool {
        matches!(self, CellValue::Empty)
    }
}

impl State {
    pub fn new(cells: [[CellValue; GRID_DIMENSION]; GRID_DIMENSION]) -> Self {
        Self {
            cells,
            selected_cell: None,
        }
    }

    pub fn perform(&mut self, action: Action) {
        match action {
            Action::SelectCell { row, column } => {
                if row < GRID_DIMENSION && column < GRID_DIMENSION {
                    self.selected_cell = Some((row, column));
                }
            }
            Action::ClearSelection => {
                self.selected_cell = None;
            }
            Action::EnterDigit { row, column, digit } => {
                if row < GRID_DIMENSION
                    && column < GRID_DIMENSION
                    && (1..=9).contains(&digit)
                    && !self.cells[row][column].is_clue()
                {
                    self.cells[row][column] = CellValue::Entry(digit);
                }
            }
        }
    }
}

impl<'a, Message> Grid<'a, Message> {
    pub fn new(state: &'a State) -> Self {
        Self {
            state,
            on_action: None,
        }
    }

    pub fn on_action(mut self, f: impl Fn(Action) -> Message + 'a) -> Self {
        self.on_action = Some(Box::new(f));
        self
    }
}

pub fn grid<Message>(state: &State) -> Grid<'_, Message> {
    Grid::new(state)
}

fn draw_cell_highlight(
    renderer: &mut impl renderer::Renderer,
    cell: &Cell,
    cell_bounds: Rectangle,
    selected_cell: Option<&Cell>,
) {
    let Some(selected_cell) = selected_cell else {
        return;
    };

    let highlight_color = if cell.row == selected_cell.row && cell.column == selected_cell.column {
        SELECTED_CELL_COLOR
    } else if !cell.is_empty() && cell.digit() == selected_cell.digit() {
        MATCHING_DIGIT_CELL_COLOR
    } else if (cell.row == selected_cell.row || cell.column == selected_cell.column)
        || (cell.row / BOX_DIMENSION == selected_cell.row / BOX_DIMENSION
            && cell.column / BOX_DIMENSION == selected_cell.column / BOX_DIMENSION)
    {
        PEER_CELL_COLOR
    } else {
        return;
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds: cell_bounds,
            ..renderer::Quad::default()
        },
        highlight_color,
    );
}

fn draw_cell_digit(renderer: &mut impl text::Renderer, cell: &Cell, bounds: Rectangle) {
    let Some(digit) = cell.digit() else {
        return;
    };

    let color = if cell.is_clue() {
        CLUE_DIGIT_COLOR
    } else {
        ENTRY_DIGIT_COLOR
    };

    renderer.fill_text(
        Text {
            content: digit.to_string(),
            bounds: bounds.size(),
            size: Pixels(bounds.width * DIGIT_SIZE_RATIO),
            line_height: text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        },
        bounds.center(),
        color,
        bounds,
    );
}

fn draw_grid_lines(renderer: &mut impl renderer::Renderer, bounds: Rectangle, cell_size: f32) {
    for line in 1..GRID_DIMENSION {
        let thickness = if line % BOX_DIMENSION == 0 {
            BOX_BORDER_WIDTH
        } else {
            CELL_BORDER_WIDTH
        };

        #[allow(clippy::cast_precision_loss)]
        let x = bounds.x + cell_size * line as f32;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: x - thickness / 2.0,
                    y: bounds.y,
                    width: thickness,
                    height: bounds.height,
                },
                snap: true,
                ..renderer::Quad::default()
            },
            LINE_COLOR,
        );

        #[allow(clippy::cast_precision_loss)]
        let y = bounds.y + cell_size * line as f32;
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: y - thickness / 2.0,
                    width: bounds.width,
                    height: thickness,
                },
                snap: true,
                ..renderer::Quad::default()
            },
            LINE_COLOR,
        );
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Grid<'_, Message>
where
    Renderer: renderer::Renderer + text::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn update(
        &mut self,
        _tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(on_action) = &self.on_action else {
            return;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(position) = cursor.position_in(layout.bounds()) else {
                    return;
                };

                #[allow(clippy::cast_precision_loss)]
                let cell_size = layout.bounds().width / GRID_DIMENSION as f32;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let column = (position.x / cell_size) as usize;
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let row = (position.y / cell_size) as usize;

                shell.publish(on_action(Action::SelectCell { row, column }));
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                shell.publish(on_action(Action::ClearSelection));
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Character(ch),
                ..
            }) => {
                if let Some((row, column)) = self.state.selected_cell
                    && !&(self.state.cells[row][column]).is_clue()
                    && let Ok(digit) = ch.parse::<u8>()
                    && (1..=9).contains(&digit)
                {
                    shell.publish(on_action(Action::EnterDigit { row, column, digit }));
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.position_in(layout.bounds()).is_some() {
            return mouse::Interaction::Pointer;
        }

        mouse::Interaction::default()
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        #[allow(clippy::cast_precision_loss)]
        let cell_size = bounds.width / GRID_DIMENSION as f32;

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            BACKGROUND_COLOR,
        );

        let selected_cell = self
            .state
            .selected_cell
            .map(|(selected_row, selected_column)| Cell {
                row: selected_row,
                column: selected_column,
                value: self.state.cells[selected_row][selected_column],
            });

        for (row_index, row) in self.state.cells.iter().enumerate() {
            for (column_index, &cell_value) in row.iter().enumerate() {
                let cell = Cell {
                    row: row_index,
                    column: column_index,
                    value: cell_value,
                };
                let cell_bounds = Rectangle {
                    #[allow(clippy::cast_precision_loss)]
                    x: bounds.x + cell_size * column_index as f32,
                    #[allow(clippy::cast_precision_loss)]
                    y: bounds.y + cell_size * row_index as f32,
                    width: cell_size,
                    height: cell_size,
                };

                draw_cell_highlight(renderer, &cell, cell_bounds, selected_cell.as_ref());
                draw_cell_digit(renderer, &cell, cell_bounds);
            }
        }

        draw_grid_lines(renderer, bounds, cell_size);
    }
}

impl<'a, Message, Theme, Renderer> From<Grid<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer + text::Renderer,
    Message: 'a,
{
    fn from(grid: Grid<'a, Message>) -> Self {
        Self::new(grid)
    }
}
