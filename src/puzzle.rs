use std::ops::Range;

use rand::random_range;

const ENCODED_PUZZLES: &[u8] = include_bytes!("../data/sudokus.bin");

const PUZZLE_RECORD_BYTES: usize = 43;
const PUZZLE_RECORD_COUNT: usize = ENCODED_PUZZLES.len() / PUZZLE_RECORD_BYTES;
const PUZZLE_RECORD_COUNT_PER_DIFFICULTY: usize = PUZZLE_RECORD_COUNT / 4;

const GRID_SIDE: usize = 9;

const CLUE_MASK_OFFSET_BITS: usize = SOLUTION_OFFSET_BITS + SOLUTION_BITS;

const SOLUTION_BITS: usize = 261;
const SOLUTION_OFFSET_BITS: usize = 0;
const SOLUTION_ROW_BIT_COUNT: usize = 29;

#[derive(Debug)]
pub struct Puzzle {
    pub difficulty: Difficulty,
    pub clues: [[Option<u8>; GRID_SIDE]; GRID_SIDE],
    pub solution: [[u8; GRID_SIDE]; GRID_SIDE],
}

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

pub fn get_random_puzzle(difficulty: Difficulty) -> Puzzle {
    let record = random_record(difficulty);

    let solution = decode_solution_grid(record);
    let clues = build_clue_grid(record, solution);

    Puzzle {
        difficulty,
        clues,
        solution,
    }
}

fn random_record(difficulty: Difficulty) -> &'static [u8] {
    let record_index = random_range(difficulty_record_range(difficulty));

    let start = record_index * PUZZLE_RECORD_BYTES;
    &ENCODED_PUZZLES[start..start + PUZZLE_RECORD_BYTES]
}

fn difficulty_record_range(difficulty: Difficulty) -> Range<usize> {
    let start = difficulty as usize * PUZZLE_RECORD_COUNT_PER_DIFFICULTY;
    start..start + PUZZLE_RECORD_COUNT_PER_DIFFICULTY
}

fn decode_solution_grid(record: &[u8]) -> [[u8; GRID_SIDE]; GRID_SIDE] {
    let mut rows = [[0; GRID_SIDE]; GRID_SIDE];

    for (row_index, row) in rows.iter_mut().enumerate() {
        let row_start_bit = (GRID_SIDE - 1 - row_index) * SOLUTION_ROW_BIT_COUNT;
        let packed_row = read_bits::<SOLUTION_ROW_BIT_COUNT>(record, row_start_bit);
        *row = decode_packed_solution_row(packed_row);
    }

    rows
}

fn read_bits<const BIT_COUNT: usize>(record: &[u8], start_bit: usize) -> u32 {
    const { assert!(BIT_COUNT <= 32) };

    let mut value = 0;

    #[allow(clippy::cast_lossless)]
    for i in 0..BIT_COUNT {
        let bit_index = start_bit + i;
        let byte = record[bit_index / 8];
        let bit_in_byte = bit_index % 8;
        let bit = (byte >> bit_in_byte) & 1;
        value |= u32::from(bit) << i;
    }

    value
}

fn decode_packed_solution_row(mut packed_row: u32) -> [u8; GRID_SIDE] {
    let mut row = [0; GRID_SIDE];

    for column in row.iter_mut().rev() {
        *column = (packed_row % 9) as u8 + 1;
        packed_row /= 9;
    }

    row
}

fn build_clue_grid(
    record: &[u8],
    solution: [[u8; GRID_SIDE]; GRID_SIDE],
) -> [[Option<u8>; GRID_SIDE]; GRID_SIDE] {
    let mut clues = [[None; GRID_SIDE]; GRID_SIDE];

    for row in 0..GRID_SIDE {
        for column in 0..GRID_SIDE {
            let mask_bit_index = CLUE_MASK_OFFSET_BITS + row * GRID_SIDE + column;
            let byte_index = mask_bit_index / 8;
            let bit_in_byte = mask_bit_index % 8;
            let bit = (record[byte_index] >> bit_in_byte) & 1;

            if bit == 1 {
                clues[row][column] = Some(solution[row][column]);
            }
        }
    }

    clues
}
