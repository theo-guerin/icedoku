use iced::{
    Border, Color, Element, Event, Length, Pixels, Rectangle, Size,
    advanced::{
        self, Clipboard, Shell,
        layout::{self, Layout},
        renderer,
        text::{self, Text},
        widget::{Widget, tree::Tree},
    },
    alignment, keyboard, mouse,
};

use crate::puzzle::{BOX_DIMENSION, GRID_DIMENSION, Puzzle};

const CELL_LINE_WIDTH: f32 = 1.0;
const BOX_LINE_WIDTH: f32 = 3.0;

const BACKGROUND_COLOR: Color = Color::from_rgb8(255, 255, 255);
const SELECTED_CELL_COLOR: Color = Color::from_rgba8(0, 79, 227, 104.0 / 255.0);
const PEER_CELL_COLOR: Color = Color::from_rgba8(101, 155, 255, 68.0 / 255.0);
const MATCHING_DIGIT_CELL_COLOR: Color = Color::from_rgb8(139, 179, 255);
const CLUE_DIGIT_COLOR: Color = Color::from_rgb8(51, 51, 51);
const ENTRY_DIGIT_COLOR: Color = Color::from_rgb8(68, 68, 221);
const INCORRECT_ENTRY_DIGIT_COLOR: Color = Color::from_rgb8(211, 47, 47);
const CELL_LINE_COLOR: Color = Color::from_rgb8(119, 119, 119);
const BOX_LINE_COLOR: Color = Color::from_rgb8(51, 51, 51);

const DIGIT_SIZE_RATIO: f32 = 0.5;
const CANDIDATE_DIGIT_SIZE_RATIO: f32 = 0.18;

pub fn puzzle_grid<'a, Message>(
    state: &'a State,
    solution: &'a [[u8; GRID_DIMENSION]; GRID_DIMENSION],
) -> PuzzleGrid<'a, Message> {
    PuzzleGrid::new(state, solution)
}

#[allow(missing_debug_implementations)]
pub struct PuzzleGrid<'a, Message> {
    state: &'a State,
    solution: &'a [[u8; GRID_DIMENSION]; GRID_DIMENSION],
    on_action: Option<Box<dyn Fn(Action) -> Message + 'a>>,
}

impl<'a, Message> PuzzleGrid<'a, Message> {
    pub fn new(state: &'a State, solution: &'a [[u8; GRID_DIMENSION]; GRID_DIMENSION]) -> Self {
        Self {
            state,
            solution,
            on_action: None,
        }
    }

    pub fn on_action(mut self, f: impl Fn(Action) -> Message + 'a) -> Self {
        self.on_action = Some(Box::new(f));
        self
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    SelectCell { row: usize, column: usize },
    ClearSelection,
    EnterDigit(u8),
    ClearCell,
    ToggleNotes,
}

#[derive(Debug, Clone, Copy)]
pub struct CellEdit {
    pub row: usize,
    pub column: usize,
    pub digit: Option<u8>,
}

#[derive(Debug)]
pub struct State {
    cells: [[CellValue; GRID_DIMENSION]; GRID_DIMENSION],
    candidates: [[CandidateSet; GRID_DIMENSION]; GRID_DIMENSION],
    selected_cell: Option<(usize, usize)>,
    notes_enabled: bool,
}

impl State {
    pub fn perform(&mut self, action: Action) -> Option<CellEdit> {
        match action {
            Action::SelectCell { row, column } => {
                if row >= GRID_DIMENSION || column >= GRID_DIMENSION {
                    return None;
                }

                self.selected_cell = Some((row, column));
            }
            Action::ClearSelection => {
                self.selected_cell = None;
            }
            Action::EnterDigit(digit) => {
                let (row, column) = self.selected_cell?;

                if self.notes_enabled {
                    if self.cells[row][column].is_empty() {
                        self.candidates[row][column].toggle(digit);
                    }

                    return None;
                }

                let cell = &mut self.cells[row][column];
                let entry = CellValue::Entry(digit);

                if (1..=9).contains(&digit) && !cell.is_clue() && *cell != entry {
                    *cell = entry;
                    self.candidates[row][column].clear();

                    return Some(CellEdit {
                        row,
                        column,
                        digit: Some(digit),
                    });
                }
            }
            Action::ClearCell => {
                let (row, column) = self.selected_cell?;

                let cell = &mut self.cells[row][column];
                if !cell.is_clue() && !cell.is_empty() {
                    *cell = CellValue::Empty;

                    return Some(CellEdit {
                        row,
                        column,
                        digit: None,
                    });
                }

                self.candidates[row][column].clear();
            }
            Action::ToggleNotes => {
                self.notes_enabled = !self.notes_enabled;
            }
        }

        None
    }

    pub fn matches_solution(&self, solution: &[[u8; GRID_DIMENSION]; GRID_DIMENSION]) -> bool {
        self.cells.iter().zip(solution).all(|(cells, solution)| {
            cells
                .iter()
                .zip(solution)
                .all(|(cell, solution)| cell.digit() == Some(*solution))
        })
    }

    pub fn remove_candidate_from_peers(&mut self, row: usize, column: usize, digit: u8) {
        for candidate_row in 0..GRID_DIMENSION {
            for candidate_column in 0..GRID_DIMENSION {
                let shares_row = candidate_row == row;
                let shares_column = candidate_column == column;
                let shares_box = candidate_row / BOX_DIMENSION == row / BOX_DIMENSION
                    && candidate_column / BOX_DIMENSION == column / BOX_DIMENSION;

                if shares_row || shares_column || shares_box {
                    self.candidates[candidate_row][candidate_column].remove(digit);
                }
            }
        }
    }

    pub fn notes_enabled(&self) -> bool {
        self.notes_enabled
    }
}

impl From<&Puzzle> for State {
    fn from(puzzle: &Puzzle) -> Self {
        Self {
            cells: puzzle
                .clues
                .map(|row| row.map(|clue| clue.map_or(CellValue::Empty, CellValue::Clue))),
            candidates: [[CandidateSet::default(); GRID_DIMENSION]; GRID_DIMENSION],
            selected_cell: None,
            notes_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellValue {
    Empty,
    Clue(u8),
    Entry(u8),
}

impl CellValue {
    pub fn digit(self) -> Option<u8> {
        match self {
            Self::Empty => None,
            Self::Clue(value) | Self::Entry(value) => Some(value),
        }
    }

    pub fn is_clue(self) -> bool {
        matches!(self, Self::Clue(_))
    }

    pub fn is_empty(self) -> bool {
        matches!(self, Self::Empty)
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct CandidateSet(u16);

impl CandidateSet {
    fn toggle(&mut self, digit: u8) {
        if let Some(bit) = candidate_bit(digit) {
            self.0 ^= bit;
        }
    }

    fn remove(&mut self, digit: u8) {
        if let Some(bit) = candidate_bit(digit) {
            self.0 &= !bit;
        }
    }

    fn contains(self, digit: u8) -> bool {
        candidate_bit(digit).is_some_and(|bit| self.0 & bit != 0)
    }

    fn clear(&mut self) {
        self.0 = 0;
    }
}

fn candidate_bit(digit: u8) -> Option<u16> {
    (1..=9)
        .contains(&digit)
        .then(|| 1_u16 << u32::from(digit - 1))
}

#[derive(Debug)]
struct PositionedCell {
    row: usize,
    column: usize,
    value: CellValue,
}

impl PositionedCell {
    fn digit(&self) -> Option<u8> {
        self.value.digit()
    }

    fn is_clue(&self) -> bool {
        self.value.is_clue()
    }

    fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    fn has_same_position(&self, other: &Self) -> bool {
        self.row == other.row && self.column == other.column
    }

    fn is_peer_of(&self, other: &Self) -> bool {
        (self.row == other.row || self.column == other.column)
            || (self.row / BOX_DIMENSION == other.row / BOX_DIMENSION
                && self.column / BOX_DIMENSION == other.column / BOX_DIMENSION)
    }
}

fn draw_cell_highlight(
    renderer: &mut impl advanced::Renderer,
    cell: &PositionedCell,
    cell_bounds: Rectangle,
    selected_cell: Option<&PositionedCell>,
) {
    let Some(selected_cell) = selected_cell else {
        return;
    };

    let highlight_color = if cell.has_same_position(selected_cell) {
        SELECTED_CELL_COLOR
    } else if !cell.is_empty() && cell.digit() == selected_cell.digit() {
        MATCHING_DIGIT_CELL_COLOR
    } else if cell.is_peer_of(selected_cell) {
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

fn draw_cell_digit(
    renderer: &mut impl text::Renderer,
    cell: &PositionedCell,
    cell_bounds: Rectangle,
    is_incorrect: bool,
) {
    let Some(digit) = cell.digit() else {
        return;
    };

    let color = if is_incorrect {
        INCORRECT_ENTRY_DIGIT_COLOR
    } else if cell.is_clue() {
        CLUE_DIGIT_COLOR
    } else {
        ENTRY_DIGIT_COLOR
    };

    renderer.fill_text(
        Text {
            content: digit.to_string(),
            bounds: cell_bounds.size(),
            size: Pixels(cell_bounds.width * DIGIT_SIZE_RATIO),
            line_height: text::LineHeight::default(),
            font: renderer.default_font(),
            align_x: text::Alignment::Center,
            align_y: alignment::Vertical::Center,
            shaping: text::Shaping::Basic,
            wrapping: text::Wrapping::None,
        },
        cell_bounds.center(),
        color,
        cell_bounds,
    );
}

fn draw_candidates(
    renderer: &mut impl text::Renderer,
    candidates: CandidateSet,
    cell_bounds: Rectangle,
) {
    #[allow(clippy::cast_precision_loss)]
    let candidate_cell_size = cell_bounds.width / BOX_DIMENSION as f32;

    for digit in 1..=9 {
        if !candidates.contains(digit) {
            continue;
        }

        let index = usize::from(digit - 1);
        let row = index / BOX_DIMENSION;
        let column = index % BOX_DIMENSION;
        let candidate_bounds = Rectangle {
            #[allow(clippy::cast_precision_loss)]
            x: cell_bounds.x + candidate_cell_size * column as f32,
            #[allow(clippy::cast_precision_loss)]
            y: cell_bounds.y + candidate_cell_size * row as f32,
            width: candidate_cell_size,
            height: candidate_cell_size,
        };

        renderer.fill_text(
            Text {
                content: digit.to_string(),
                bounds: candidate_bounds.size(),
                size: Pixels(cell_bounds.width * CANDIDATE_DIGIT_SIZE_RATIO),
                line_height: text::LineHeight::default(),
                font: renderer.default_font(),
                align_x: text::Alignment::Center,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
            },
            candidate_bounds.center(),
            ENTRY_DIGIT_COLOR,
            candidate_bounds,
        );
    }
}

fn draw_grid_lines(renderer: &mut impl advanced::Renderer, bounds: Rectangle, cell_size: f32) {
    for line in 1..GRID_DIMENSION {
        let (thickness, color) = if line % BOX_DIMENSION == 0 {
            (BOX_LINE_WIDTH, BOX_LINE_COLOR)
        } else {
            (CELL_LINE_WIDTH, CELL_LINE_COLOR)
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
                ..renderer::Quad::default()
            },
            color,
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
                ..renderer::Quad::default()
            },
            color,
        );
    }

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border {
                color: BOX_LINE_COLOR,
                width: BOX_LINE_WIDTH,
                ..Border::default()
            },
            ..renderer::Quad::default()
        },
        Color::TRANSPARENT,
    );
}

fn digit_from_key(key: &keyboard::Key, physical_key: keyboard::key::Physical) -> Option<u8> {
    use keyboard::key::{Code, Physical};

    if let keyboard::Key::Character(character) = key
        && let Ok(digit @ 1..=9) = character.parse()
    {
        return Some(digit);
    }

    match physical_key {
        Physical::Code(Code::Numpad1) => Some(1),
        Physical::Code(Code::Numpad2) => Some(2),
        Physical::Code(Code::Numpad3) => Some(3),
        Physical::Code(Code::Numpad4) => Some(4),
        Physical::Code(Code::Numpad5) => Some(5),
        Physical::Code(Code::Numpad6) => Some(6),
        Physical::Code(Code::Numpad7) => Some(7),
        Physical::Code(Code::Numpad8) => Some(8),
        Physical::Code(Code::Numpad9) => Some(9),
        _ => None,
    }
}

fn is_notes_toggle_key(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
    matches!(key, keyboard::Key::Character(character) if character.eq_ignore_ascii_case("n"))
        && !modifiers.control()
        && !modifiers.alt()
        && !modifiers.logo()
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for PuzzleGrid<'_, Message>
where
    Renderer: advanced::Renderer + text::Renderer,
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
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
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
        if shell.is_event_captured() {
            return;
        }

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
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) if self.state.selected_cell.is_some() => {
                shell.publish(on_action(Action::ClearSelection));
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Backspace),
                ..
            }) if self.state.selected_cell.is_some() => {
                shell.publish(on_action(Action::ClearCell));
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                repeat: false,
                ..
            }) if is_notes_toggle_key(key, *modifiers) => {
                shell.publish(on_action(Action::ToggleNotes));
                shell.capture_event();
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key, physical_key, ..
            }) if self.state.selected_cell.is_some() => {
                if let Some(digit) = digit_from_key(key, *physical_key) {
                    shell.publish(on_action(Action::EnterDigit(digit)));
                    shell.capture_event();
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
        if self.on_action.is_some() && cursor.position_in(layout.bounds()).is_some() {
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
            .map(|(selected_row, selected_column)| PositionedCell {
                row: selected_row,
                column: selected_column,
                value: self.state.cells[selected_row][selected_column],
            });

        for (row_index, row) in self.state.cells.iter().enumerate() {
            for (column_index, &cell_value) in row.iter().enumerate() {
                let cell = PositionedCell {
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
                let is_incorrect = !cell.is_clue()
                    && !cell.is_empty()
                    && cell.digit() != Some(self.solution[row_index][column_index]);

                if cell.is_empty() {
                    draw_candidates(
                        renderer,
                        self.state.candidates[row_index][column_index],
                        cell_bounds,
                    );
                } else {
                    draw_cell_digit(renderer, &cell, cell_bounds, is_incorrect);
                }
            }
        }

        draw_grid_lines(renderer, bounds, cell_size);
    }
}

impl<'a, Message, Theme, Renderer> From<PuzzleGrid<'a, Message>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + text::Renderer,
    Message: 'a,
{
    fn from(puzzle_grid: PuzzleGrid<'a, Message>) -> Self {
        Self::new(puzzle_grid)
    }
}
