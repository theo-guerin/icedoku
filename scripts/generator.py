#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import subprocess
from collections.abc import Sequence
from enum import StrEnum, auto
from functools import reduce
from itertools import batched
from pathlib import Path
from typing import NamedTuple

PROJECT_ROOT_PATH = Path(__file__).parent.parent
OUTPUT_DIRECTORY_PATH = PROJECT_ROOT_PATH / "data"
OUTPUT_FILE_PATH = OUTPUT_DIRECTORY_PATH / "sudokus.bin"

PUZZLES_PER_DIFFICULTY = 1000
SUDOKU_GRID_SIZE = 9

SOLUTION_ROW_BIT_COUNT = 29
SOLUTION_BIT_COUNT = SUDOKU_GRID_SIZE * SOLUTION_ROW_BIT_COUNT
PUZZLE_RECORD_BYTE_COUNT = 43


class Difficulty(StrEnum):
    SIMPLE = auto()
    EASY = auto()
    INTERMEDIATE = auto()
    EXPERT = auto()


class Puzzle(NamedTuple):
    clues: list[int]
    solution: list[int]


def start_qqwing(puzzle_count: int, difficulty: Difficulty) -> subprocess.Popen[str]:
    return subprocess.Popen(
        [
            "qqwing",
            "--generate",
            str(puzzle_count),
            "--solution",
            "--difficulty",
            difficulty.value,
            "--one-line",
        ],
        stdout=subprocess.PIPE,
        text=True,
    )


def parse_qqwing_puzzles(qqwing_output: str) -> list[Puzzle]:
    return [
        Puzzle(clues=list(map(int, clues_line)), solution=list(map(int, solution_line)))
        for clues_line, solution_line in batched(
            qqwing_output.replace(".", "0").splitlines(), 2
        )
    ]


def encode_puzzles(puzzles_by_difficulty: dict[Difficulty, list[Puzzle]]) -> bytes:
    """Encode puzzles as fixed-size little-endian records.

    Each record stores, from least to most significant bits:
    - solution: 261 bits (nine 29-bit row values)
    - clue bit mask: 81 row-major bits
    - padding: 2 bits

    Records follow the input mapping's iteration order.
    """

    def encode_solution_row(row_digits: Sequence[int]) -> int:
        """Encode a solution row as a base-9 integer, mapping digits 1-9 to 0-8."""
        return reduce(
            lambda encoded_value, digit: encoded_value * SUDOKU_GRID_SIZE + digit - 1,
            row_digits,
            0,
        )

    def pack_solution(solution_digits: Sequence[int]) -> int:
        """Pack a solution's encoded rows into a 261-bit integer."""
        encoded_row_values = [
            encode_solution_row(row)
            for row in batched(solution_digits, SUDOKU_GRID_SIZE)
        ]
        return reduce(
            lambda packed_value, encoded_row_value: (
                (packed_value << SOLUTION_ROW_BIT_COUNT) | encoded_row_value
            ),
            encoded_row_values,
            0,
        )

    def build_clue_bit_mask(clues: Sequence[int]) -> int:
        """Build a row-major bit mask for non-empty clue cells."""
        return sum(
            (1 << cell_index) for cell_index, digit in enumerate(clues) if digit != 0
        )

    encoded_puzzles = bytearray()

    for difficulty_puzzles in puzzles_by_difficulty.values():
        for puzzle in difficulty_puzzles:
            packed_solution = pack_solution(puzzle.solution)
            clue_bit_mask = build_clue_bit_mask(puzzle.clues)
            packed_puzzle_record = packed_solution + (
                clue_bit_mask << SOLUTION_BIT_COUNT
            )
            encoded_puzzles.extend(
                packed_puzzle_record.to_bytes(PUZZLE_RECORD_BYTE_COUNT, "little")
            )

    return bytes(encoded_puzzles)


def generate_puzzles_by_difficulty(
    puzzle_count: int,
) -> dict[Difficulty, list[Puzzle]]:
    qqwing_processes = [
        (start_qqwing(puzzle_count, difficulty), difficulty)
        for difficulty in Difficulty
    ]

    puzzles_by_difficulty: dict[Difficulty, list[Puzzle]] = {}

    for qqwing_process, difficulty in qqwing_processes:
        qqwing_output, _ = qqwing_process.communicate()
        puzzles_by_difficulty[difficulty] = parse_qqwing_puzzles(qqwing_output)

    return puzzles_by_difficulty


def main() -> None:
    puzzles_by_difficulty = generate_puzzles_by_difficulty(PUZZLES_PER_DIFFICULTY)
    encoded_puzzles = encode_puzzles(puzzles_by_difficulty)
    OUTPUT_DIRECTORY_PATH.mkdir(exist_ok=True)
    _ = OUTPUT_FILE_PATH.write_bytes(encoded_puzzles)


if __name__ == "__main__":
    main()
