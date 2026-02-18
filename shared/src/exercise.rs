use rand::prelude::*;

use crate::difficulty::Difficulty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Operation {
    Multiply,
    Divide,
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

pub fn generate_exercise(difficulty: &Difficulty) -> ExerciseData {
    let mut rng = rand::thread_rng();
    let (table_range, timer) = difficulty.config();

    let a = rng.r#gen_range(table_range.clone());
    let b = rng.r#gen_range(table_range.clone());

    let (operation, operand_a, operand_b, correct_answer) = if rng.r#gen::<bool>() {
        (Operation::Multiply, a, b, a * b)
    } else {
        let product = a * b;
        (Operation::Divide, product, a, b)
    };

    let mut distractors = Vec::new();
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

    for &c in &candidates {
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
        let j = rng.r#gen_range(0..=i);
        choices.swap(i, j);
    }

    ExerciseData {
        operation,
        operand_a,
        operand_b,
        correct_answer,
        choices,
        time_limit: timer,
    }
}

impl ExerciseData {
    pub fn question_text(&self) -> String {
        match self.operation {
            Operation::Multiply => format!("{} x {} = ?", self.operand_a, self.operand_b),
            Operation::Divide => format!("{} / {} = ?", self.operand_a, self.operand_b),
        }
    }
}
