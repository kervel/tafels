use rand::prelude::*;

use crate::difficulty::Difficulty;
use crate::mode::GameMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Operation {
    Multiply,
    Divide,
    Add,
    Subtract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ExerciseState {
    Pending,
    Active,
    Answered,
    TimedOut,
}

#[derive(Debug, Clone)]
pub struct ExerciseData {
    pub operation: Operation,
    pub operand_a: u32,
    pub operand_b: u32,
    pub correct_answer: u32,
    pub choices: [u32; 4],
    pub time_limit: f32,
}

pub fn generate_exercise(mode: GameMode, difficulty: &Difficulty) -> ExerciseData {
    match mode {
        GameMode::Tables => generate_tables(difficulty),
        GameMode::Arithmetic => generate_arithmetic(difficulty),
    }
}

fn generate_tables(difficulty: &Difficulty) -> ExerciseData {
    let mut rng = rand::rng();
    let (table_range, timer) = difficulty.config();

    let a = rng.random_range(table_range.clone());
    let b = rng.random_range(table_range.clone());

    let (operation, operand_a, operand_b, correct_answer) = if rng.random::<bool>() {
        (Operation::Multiply, a, b, a * b)
    } else {
        let product = a * b;
        (Operation::Divide, product, a, b)
    };

    let candidates = [
        correct_answer.wrapping_add(1),
        correct_answer.wrapping_sub(1),
        correct_answer.wrapping_add(a),
        correct_answer.wrapping_sub(a),
        correct_answer.wrapping_add(b),
        correct_answer.wrapping_sub(b),
        a.wrapping_mul(b.wrapping_add(1)),
        a.wrapping_mul(b.wrapping_sub(1)),
        correct_answer.wrapping_add(2),
    ];

    ExerciseData {
        operation,
        operand_a,
        operand_b,
        correct_answer,
        choices: build_choices(&mut rng, correct_answer, &candidates),
        time_limit: timer,
    }
}

/// Addition/subtraction that always crosses a ten boundary (the carry/borrow
/// step children struggle with). Ranges per difficulty:
/// - Easy: one-digit + one-digit, result <= 20 (8+6, 12-7)
/// - Medium: two-digit +/- one-digit, result <= 100 (47+8, 63-7)
/// - Hard: two-digit +/- two-digit, result <= 100 (47+38, 82-35)
fn generate_arithmetic(difficulty: &Difficulty) -> ExerciseData {
    let mut rng = rand::rng();
    let (_, timer) = difficulty.config();

    // Pick an addition a + b = sum whose units digits carry, then randomly
    // present it as the inverse subtraction sum - b = a.
    let (a, b): (u32, u32) = loop {
        let (a, b) = match difficulty {
            Difficulty::Easy => (rng.random_range(2..=9), rng.random_range(2..=9)),
            Difficulty::Medium => (rng.random_range(11..=91), rng.random_range(2..=9)),
            Difficulty::Hard => (rng.random_range(11..=89), rng.random_range(11..=89)),
        };
        if (a % 10) + (b % 10) > 10 && a + b <= 100 {
            break (a, b);
        }
    };
    let sum = a + b;

    let (operation, operand_a, operand_b, correct_answer) = if rng.random::<bool>() {
        (Operation::Add, a, b, sum)
    } else {
        (Operation::Subtract, sum, b, a)
    };

    // Distractors sit right next to the answer (8+4 -> 11, 12, 13) so the
    // child can't pick by magnitude alone.
    let candidates = [
        correct_answer.wrapping_add(1),
        correct_answer.wrapping_sub(1),
        correct_answer.wrapping_add(2),
        correct_answer.wrapping_sub(2),
        correct_answer.wrapping_add(3),
        correct_answer.wrapping_sub(3),
    ];

    ExerciseData {
        operation,
        operand_a,
        operand_b,
        correct_answer,
        choices: build_choices(&mut rng, correct_answer, &candidates),
        time_limit: timer,
    }
}

/// Pick 3 distinct positive distractors from `candidates` (in priority order,
/// falling back to answer+3, +4, ...), then shuffle them with the answer.
fn build_choices(rng: &mut impl Rng, correct_answer: u32, candidates: &[u32]) -> [u32; 4] {
    let mut distractors = Vec::new();
    for &c in candidates {
        if c > 0 && c != correct_answer && !distractors.contains(&c) {
            distractors.push(c);
            if distractors.len() == 3 {
                break;
            }
        }
    }

    let mut fallback = correct_answer + 3;
    while distractors.len() < 3 {
        if fallback != correct_answer && !distractors.contains(&fallback) {
            distractors.push(fallback);
        }
        fallback += 1;
    }

    let mut choices = [
        correct_answer,
        distractors[0],
        distractors[1],
        distractors[2],
    ];
    for i in (1..4).rev() {
        let j = rng.random_range(0..=i);
        choices.swap(i, j);
    }
    choices
}

impl ExerciseData {
    pub fn question_text(&self) -> String {
        match self.operation {
            Operation::Multiply => format!("{} x {} = ?", self.operand_a, self.operand_b),
            Operation::Divide => format!("{} / {} = ?", self.operand_a, self.operand_b),
            Operation::Add => format!("{} + {} = ?", self.operand_a, self.operand_b),
            Operation::Subtract => format!("{} - {} = ?", self.operand_a, self.operand_b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::GameMode;

    fn crosses_ten(op: Operation, a: u32, b: u32) -> bool {
        match op {
            Operation::Add => (a % 10) + (b % 10) > 10,
            Operation::Subtract => (a % 10) < (b % 10),
            _ => false,
        }
    }

    fn check_arithmetic(difficulty: Difficulty, max_result: u32, both_two_digit: bool) {
        for _ in 0..500 {
            let ex = generate_exercise(GameMode::Arithmetic, &difficulty);
            assert!(
                matches!(ex.operation, Operation::Add | Operation::Subtract),
                "arithmetic mode must only produce +/-"
            );
            let (a, b) = (ex.operand_a, ex.operand_b);
            match ex.operation {
                Operation::Add => assert_eq!(ex.correct_answer, a + b),
                Operation::Subtract => assert_eq!(ex.correct_answer, a - b),
                _ => unreachable!(),
            }
            assert!(a <= max_result && ex.correct_answer <= max_result, "{:?}", ex);
            assert!(crosses_ten(ex.operation, a, b), "must cross a ten: {:?}", ex);
            if both_two_digit {
                assert!(b >= 10, "hard: both operands two-digit: {:?}", ex);
            } else if difficulty == Difficulty::Medium {
                assert!(b < 10, "medium: second operand one-digit: {:?}", ex);
            }
            assert!(ex.choices.contains(&ex.correct_answer));
            for &c in &ex.choices {
                assert!(c > 0, "choices must be positive: {:?}", ex);
                let d = c.abs_diff(ex.correct_answer);
                assert!(d <= 3, "distractors must be close to answer: {:?}", ex);
            }
            let mut sorted = ex.choices.to_vec();
            sorted.sort();
            sorted.dedup();
            assert_eq!(sorted.len(), 4, "choices must be distinct: {:?}", ex);
        }
    }

    #[test]
    fn arithmetic_easy_sums_to_20_crossing_ten() {
        check_arithmetic(Difficulty::Easy, 20, false);
    }

    #[test]
    fn arithmetic_medium_one_digit_operand_crossing_ten_within_100() {
        check_arithmetic(Difficulty::Medium, 100, false);
    }

    #[test]
    fn arithmetic_hard_two_digit_operands_crossing_ten_within_100() {
        check_arithmetic(Difficulty::Hard, 100, true);
    }

    #[test]
    fn tables_mode_only_multiplies_or_divides() {
        for _ in 0..200 {
            let ex = generate_exercise(GameMode::Tables, &Difficulty::Easy);
            assert!(matches!(ex.operation, Operation::Multiply | Operation::Divide));
        }
    }

    #[test]
    fn question_text_for_add_and_subtract() {
        let mk = |operation, a, b| ExerciseData {
            operation,
            operand_a: a,
            operand_b: b,
            correct_answer: 0,
            choices: [1, 2, 3, 4],
            time_limit: 1.0,
        };
        assert_eq!(mk(Operation::Add, 8, 6).question_text(), "8 + 6 = ?");
        assert_eq!(mk(Operation::Subtract, 12, 7).question_text(), "12 - 7 = ?");
    }
}
