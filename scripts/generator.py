#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import subprocess
from enum import StrEnum, auto
from functools import reduce
from itertools import batched
from pathlib import Path
from typing import NamedTuple, Sequence

PUZZLES_PER_DIFFICULTY = 1000
OUTPUT_PATH = Path(__file__).parent.parent / "data" / "sudokus.bin"

SOLUTION_ROW_BIT_COUNT = 29
SOLUTION_BIT_COUNT = 9 * SOLUTION_ROW_BIT_COUNT
PUZZLE_RECORD_SIZE = 43


class Difficulty(StrEnum):
    SIMPLE = auto()
    EASY = auto()
    INTERMEDIATE = auto()
    EXPERT = auto()


class Puzzle(NamedTuple):
    clues: list[int]
    solution: list[int]


def start_qqwing(count: int, difficulty: Difficulty) -> subprocess.Popen[str]:
    return subprocess.Popen(
        [
            "qqwing",
            "--generate",
            str(count),
            "--solution",
            "--difficulty",
            difficulty.value,
            "--one-line",
        ],
        stdout=subprocess.PIPE,
        text=True,
    )


def parse_qqwing_output(output_text: str) -> list[Puzzle]:
    return [
        Puzzle(
            clues=[int(x) for x in clues_line.replace(".", "0")],
            solution=[int(x) for x in solution_line.replace(".", "0")],
        )
        for clues_line, solution_line in batched(output_text.splitlines(), 2)
    ]


def encode_puzzles(puzzles_by_difficulty: dict[Difficulty, list[Puzzle]]) -> bytes:
    """
    Format per puzzle:
    - solution: 261 bits (9 row values, 29 bits each)
    - clue mask: 81 bits (row-major)
    - padding: 2 bits
    """

    def encode_solution_row(digits: Sequence[int]) -> int:
        return reduce(
            lambda accumulator, number: accumulator * 9 + number - 1,
            digits,
            0,
        )

    def encode_solution_rows(digits: Sequence[int]) -> int:
        row_values = [encode_solution_row(row) for row in batched(digits, 9)]
        return reduce(
            lambda accumulator, row_value: (
                (accumulator << SOLUTION_ROW_BIT_COUNT) | row_value
            ),
            row_values,
            0,
        )

    result = bytearray()

    for puzzles in puzzles_by_difficulty.values():
        for puzzle in puzzles:
            encoded_solution = encode_solution_rows(puzzle.solution)
            clue_mask = sum(
                1 << cell_index
                for cell_index, digit in enumerate(puzzle.clues)
                if digit != 0
            )
            packed_puzzle = encoded_solution + (clue_mask << SOLUTION_BIT_COUNT)
            result.extend(packed_puzzle.to_bytes(PUZZLE_RECORD_SIZE, "little"))

    return bytes(result)


def main() -> None:
    generator_processes = [
        (start_qqwing(PUZZLES_PER_DIFFICULTY, difficulty), difficulty)
        for difficulty in Difficulty
    ]

    puzzles_by_difficulty: dict[Difficulty, list[Puzzle]] = {}

    for process, difficulty in generator_processes:
        stdout, _stderr = process.communicate()
        puzzles_by_difficulty[difficulty] = parse_qqwing_output(stdout)

    encoded = encode_puzzles(puzzles_by_difficulty)
    OUTPUT_PATH.parent.mkdir(exist_ok=True)
    OUTPUT_PATH.write_bytes(encoded)


if __name__ == "__main__":
    main()
