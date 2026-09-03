use crate::{
    puzzle::{Difficulty, GRID_DIMENSION, Puzzle},
    widget::puzzle_grid,
};

pub const MAX_MISTAKES: u8 = 3;

#[derive(Debug)]
pub struct Game {
    puzzle: Puzzle,
    grid: puzzle_grid::State,
    mistakes: u8,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Lost,
}

impl Game {
    pub fn new(puzzle: Puzzle) -> Self {
        let grid = puzzle_grid::State::from(&puzzle);

        Self {
            puzzle,
            grid,
            mistakes: 0,
            status: Status::Playing,
        }
    }

    pub fn perform(&mut self, action: puzzle_grid::Action) {
        if self.status != Status::Playing {
            return;
        }

        let Some(edit) = self.grid.perform(action) else {
            return;
        };
        let Some(digit) = edit.digit else {
            return;
        };

        if digit != self.puzzle.solution[edit.row][edit.column] {
            self.mistakes += 1;

            if self.mistakes == MAX_MISTAKES {
                self.status = Status::Lost;
            }
        } else if self.grid.matches_solution(&self.puzzle.solution) {
            self.status = Status::Won;
        }
    }

    pub fn restart(&mut self) {
        self.grid = puzzle_grid::State::from(&self.puzzle);
        self.mistakes = 0;
        self.status = Status::Playing;
    }

    pub fn difficulty(&self) -> Difficulty {
        self.puzzle.difficulty
    }

    pub fn grid(&self) -> &puzzle_grid::State {
        &self.grid
    }

    pub fn solution(&self) -> &[[u8; GRID_DIMENSION]; GRID_DIMENSION] {
        &self.puzzle.solution
    }

    pub fn mistakes(&self) -> u8 {
        self.mistakes
    }

    pub fn status(&self) -> Status {
        self.status
    }
}
