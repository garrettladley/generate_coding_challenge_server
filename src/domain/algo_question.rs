use std::fmt::{Display, Formatter};

use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use strum::IntoEnumIterator;

pub struct Challenge {
    pub challenge: Vec<String>,
    pub solution: Vec<String>,
}

pub fn generate_challenge(n_random: usize, mandatory_cases: Vec<String>) -> Challenge {
    let mut rng = rand::thread_rng();
    let random_cases = (0..n_random)
        .map(|_| generate_random_case(&mut rng))
        .collect::<Vec<String>>();

    let mut challenge = mandatory_cases;

    challenge.extend(random_cases);

    challenge.shuffle(&mut rng);

    let solution = challenge
        .iter()
        .map(|case| parse_barcode(case))
        .collect::<Vec<String>>();

    Challenge {
        challenge,
        solution,
    }
}

#[derive(strum::EnumIter, Default, Debug)]
enum Instruction {
    #[default]
    BegEnd,
    Repeat,
    Reverse,
    Encrypt,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::BegEnd => write!(f, "#"),
            Instruction::Repeat => write!(f, "!"),
            Instruction::Reverse => write!(f, "^"),
            Instruction::Encrypt => write!(f, "%"),
        }
    }
}

impl Instruction {
    fn parse(value: &char) -> Option<Instruction> {
        match value {
            '#' => Some(Instruction::BegEnd),
            '!' => Some(Instruction::Repeat),
            '^' => Some(Instruction::Reverse),
            '%' => Some(Instruction::Encrypt),
            _ => None,
        }
    }
}

fn generate_random_case(rng: &mut impl Rng) -> String {
    let num_numeric = rng.gen_range(32..=64);
    let num_instructions = rng.gen_range(16..=32);

    let mut result = Instruction::BegEnd.to_string();

    result += &(0..num_numeric)
        .map(|_| rng.gen_range(0..=9).to_string())
        .collect::<String>();

    let mut instruction_positions: Vec<usize> = (1..num_numeric).collect();
    instruction_positions.shuffle(rng);
    instruction_positions.truncate(num_instructions);
    instruction_positions.sort_unstable();

    instruction_positions
        .iter()
        .enumerate()
        .for_each(|(i, &position)| {
            if let Some(instruction) = Instruction::iter().choose(rng) {
                let instruction_str = instruction.to_string();
                let adjusted_position = position + i * instruction_str.len();
                result.insert_str(adjusted_position, &instruction_str);
            }
        });

    result += &Instruction::BegEnd.to_string();

    result
}

pub fn parse_barcode(barcode: &str) -> String {
    let mut result: Vec<String> = Vec::new();
    let mut current_block = String::new();
    let mut previous_block = String::new();

    barcode.chars().for_each(|c| match Instruction::parse(&c) {
        Some(instrunction) => match instrunction {
            Instruction::BegEnd => {
                result.push(current_block.clone());
                previous_block = current_block.clone();
                current_block.clear();
            }
            Instruction::Repeat => {
                current_block += &previous_block;
            }
            Instruction::Reverse => {
                if let Some(last) = result.last_mut() {
                    *last = last.chars().rev().collect();
                }
            }
            Instruction::Encrypt => {
                if let Some(last) = result.last_mut() {
                    *last = last
                        .chars()
                        .map(|d| {
                            if d == '0' {
                                '0'
                            } else {
                                std::char::from_digit((d.to_digit(10).unwrap() * 2) % 10, 10)
                                    .unwrap()
                            }
                        })
                        .collect();
                }
            }
        },
        None => {
            current_block.push(c);
        }
    });

    result.push(current_block);
    result.concat()
}

#[cfg(test)]
mod tests {
    use crate::domain::algo_question::Instruction;

    use super::{generate_challenge, generate_random_case, parse_barcode};

    #[test]
    fn test_generate_challenge() {
        let challenge = generate_challenge(3, vec!["1234567890".to_string()]);
        assert_eq!(challenge.challenge.len(), 4);
        assert_eq!(challenge.solution.len(), 4);

        assert!(challenge.challenge.contains(&"1234567890".to_string()));
    }

    #[test]
    fn test_generated_random_case() {
        let generated_case = generate_random_case(&mut rand::thread_rng());

        assert_eq!(
            generated_case.chars().next().unwrap(),
            Instruction::BegEnd.to_string().chars().next().unwrap()
        );
        assert_eq!(
            generated_case.chars().last().unwrap(),
            Instruction::BegEnd.to_string().chars().next().unwrap()
        );
    }

    #[test]
    fn test_parse_barcode_example() {
        assert_eq!(parse_barcode("#12#34!#59^#67%#"), "1221430867");
    }

    #[test]
    fn test_parse_barcode_edge() {
        assert_eq!(parse_barcode("#12^!%%###34^#"), "1234");
    }
}
