use std::ops::Range;

use rand::random_range;
use strum::EnumCount;
use strum_macros::EnumCount;

// See scripts/generator.py for the encoding details
const ENCODED_PUZZLE_RECORDS: &[u8] = include_bytes!("../data/sudokus.bin");

pub const GRID_DIMENSION: usize = 9;
pub const BOX_DIMENSION: usize = 3;

const PUZZLE_RECORD_BYTE_COUNT: usize = 43;
const PUZZLE_RECORD_COUNT: usize = ENCODED_PUZZLE_RECORDS.len() / PUZZLE_RECORD_BYTE_COUNT;
const RECORDS_PER_DIFFICULTY: usize = PUZZLE_RECORD_COUNT / Difficulty::COUNT;

const SOLUTION_BIT_COUNT: usize = 261;
const SOLUTION_ROW_BIT_COUNT: usize = 29;

#[derive(Debug)]
pub struct Puzzle {
    pub difficulty: Difficulty,
    pub clues: [[Option<u8>; GRID_DIMENSION]; GRID_DIMENSION],
    pub solution: [[u8; GRID_DIMENSION]; GRID_DIMENSION],
}

#[derive(Debug, Clone, Copy, EnumCount)]
pub enum Difficulty {
    Simple,
    Easy,
    Intermediate,
    Expert,
}

pub fn get_random_puzzle(difficulty: Difficulty) -> Puzzle {
    let record = random_record(difficulty);

    let solution = decode_solution(record);
    let clues = build_clues(record, solution);

    Puzzle {
        difficulty,
        clues,
        solution,
    }
}

fn random_record(difficulty: Difficulty) -> &'static [u8] {
    let record_index = random_range(difficulty_record_range(difficulty));

    let start = record_index * PUZZLE_RECORD_BYTE_COUNT;
    &ENCODED_PUZZLE_RECORDS[start..start + PUZZLE_RECORD_BYTE_COUNT]
}

fn difficulty_record_range(difficulty: Difficulty) -> Range<usize> {
    let start = difficulty as usize * RECORDS_PER_DIFFICULTY;
    start..start + RECORDS_PER_DIFFICULTY
}

fn decode_solution(record: &[u8]) -> [[u8; GRID_DIMENSION]; GRID_DIMENSION] {
    let mut rows = [[0; GRID_DIMENSION]; GRID_DIMENSION];

    for (index, row) in rows.iter_mut().enumerate() {
        let start_bit = (GRID_DIMENSION - 1 - index) * SOLUTION_ROW_BIT_COUNT;
        let packed_row = extract_bits::<SOLUTION_ROW_BIT_COUNT>(record, start_bit);
        *row = decode_packed_solution_row(packed_row);
    }

    rows
}

fn extract_bits<const BIT_COUNT: usize>(record: &[u8], start_bit: usize) -> u32 {
    const { assert!(BIT_COUNT <= 32) };

    let mut value = 0;

    for index in 0..BIT_COUNT {
        let bit_index = start_bit + index;
        let byte = record[bit_index / 8];
        let bit_in_byte = bit_index % 8;
        let bit = (byte >> bit_in_byte) & 1;
        value |= u32::from(bit) << index;
    }

    value
}

fn decode_packed_solution_row(packed_row: u32) -> [u8; GRID_DIMENSION] {
    let mut row = [0; GRID_DIMENSION];

    let mut packed_row = packed_row;
    for column in row.iter_mut().rev() {
        *column = (packed_row % 9) as u8 + 1;
        packed_row /= 9;
    }

    row
}

fn build_clues(
    record: &[u8],
    solution: [[u8; GRID_DIMENSION]; GRID_DIMENSION],
) -> [[Option<u8>; GRID_DIMENSION]; GRID_DIMENSION] {
    let mut clues = [[None; GRID_DIMENSION]; GRID_DIMENSION];

    for row in 0..GRID_DIMENSION {
        for column in 0..GRID_DIMENSION {
            let bit_index = SOLUTION_BIT_COUNT + row * GRID_DIMENSION + column;
            let byte_index = bit_index / 8;
            let bit_in_byte = bit_index % 8;
            let bit = (record[byte_index] >> bit_in_byte) & 1;

            if bit == 1 {
                clues[row][column] = Some(solution[row][column]);
            }
        }
    }

    clues
}
