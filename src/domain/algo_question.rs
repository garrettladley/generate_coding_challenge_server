use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng,
};

use strum::IntoEnumIterator;

use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use crate::domain::Color;

#[derive(strum::EnumIter, Debug)]
enum EditType {
    Insertion,
    Deletion,
    Substitution,
}

pub fn generate_challenge(
    nuid: &str,
    n_random: usize,
    mandatory_cases: Vec<String>,
) -> (Vec<String>, Vec<String>) {
    let mut rng: Pcg64 = Seeder::from(nuid).make_rng();
    let random_cases: Vec<String> = (0..n_random)
        .map(|_| {
            let color = Color::iter().choose(&mut rng).unwrap().to_string();
            let len = color.len();
            let random_count = rng.gen_range(0..=len);
            if random_count == 0 {
                return color;
            }
            match EditType::iter().choose(&mut rng).unwrap() {
                EditType::Deletion => color.chars().skip(random_count).collect(),
                EditType::Insertion => {
                    let alphabet: Vec<char> = ('a'..='z').collect();
                    let mut color_chars: Vec<char> = color.chars().collect();
                    let random_chars = alphabet
                        .choose_multiple(&mut rng, random_count)
                        .cloned()
                        .collect::<Vec<char>>();
                    let random_indices = (0..random_count)
                        .map(|_| rng.gen_range(0..=color_chars.len()))
                        .collect::<Vec<usize>>();
                    for (index, random_char) in random_indices.into_iter().zip(random_chars) {
                        color_chars.insert(index, random_char);
                    }
                    color_chars.into_iter().collect()
                }
                EditType::Substitution => {
                    let changed_indices: Vec<_> =
                        (0..random_count).map(|_| rng.gen_range(0..len)).collect();
                    let alphabet: Vec<char> = ('a'..='z').collect();
                    let mut color_chars: Vec<char> = color.chars().collect();

                    for index in changed_indices {
                        let original_char = color_chars[index];
                        let mut new_char;
                        loop {
                            new_char = *alphabet.choose(&mut rng).unwrap();
                            if new_char != original_char {
                                break;
                            }
                        }
                        color_chars[index] = new_char;
                    }
                    color_chars.into_iter().collect()
                }
            }
        })
        .collect();

    let mut all_cases = mandatory_cases;
    all_cases.extend(random_cases);

    let answers: Vec<String> = all_cases
        .iter()
        .filter_map(|case| one_edit_away(case))
        .map(|color| color.to_string())
        .collect::<Vec<String>>();

    (all_cases, answers)
}

fn n_edits_away(str1: &str, str2: &str, n: isize) -> bool {
    if (str1.len() as isize - str2.len() as isize).abs() > n {
        return false;
    }

    let (shorter, longer) = if str1.len() > str2.len() {
        (str2, str1)
    } else {
        (str1, str2)
    };

    let mut short_pointer = 0;
    let mut long_pointer = 0;
    let mut edit_count = 0;

    while short_pointer < shorter.len() && long_pointer < longer.len() {
        if shorter.chars().nth(short_pointer) != longer.chars().nth(long_pointer) {
            edit_count += 1;
            if edit_count > n {
                return false;
            }
            if shorter.len() == longer.len() {
                short_pointer += 1;
            }
        } else {
            short_pointer += 1;
        }
        long_pointer += 1;
    }
    edit_count <= n
}

pub fn one_edit_away(str: &str) -> Option<Color> {
    Color::iter().find(|&color| n_edits_away(str, color.to_string().as_str(), 1))
}
#[cfg(test)]
mod tests {

    use super::generate_challenge;
    use super::one_edit_away;
    use super::Color;

    #[test]
    fn test_generate_challenge() {
        let mandatory_cases: Vec<String> = vec![
            String::from(""),
            Color::Red.to_string(),
            Color::Orange.to_string(),
            Color::Yellow.to_string(),
            Color::Green.to_string(),
            Color::Blue.to_string(),
            Color::Violet.to_string(),
        ];
        let n_mandatory = mandatory_cases.len();
        let n_random = 10;
        let (cases, answers) =
            generate_challenge(&String::from("001234567"), n_random, mandatory_cases);

        assert_eq!(cases.len(), n_mandatory + n_random);

        assert!(answers.iter().all(|answer| one_edit_away(answer).is_some()));
    }

    #[test]
    fn test_one_edit_away_example() {
        assert_eq!(one_edit_away("red").unwrap(), Color::Red);
        assert_eq!(one_edit_away("lue").unwrap(), Color::Blue);
        assert!(one_edit_away("ooran").is_none());
        assert!(one_edit_away("abc").is_none());
        assert_eq!(one_edit_away("greene").unwrap(), Color::Green);
    }
}
