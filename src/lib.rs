//!This library will convert English number names to their integer equivalent and vice versa.
//!
//! Uses Levenshtein distance algorithm to accept words a distance of 1 from a valid spelling.

pub mod to {
    use std::collections::HashMap;

    const MAX_STRING_DISTANCE: i32 = 1;

    ///This function takes an integer and returns its English name.
    ///
    /// ```
    /// # use to_int_and_back::to;
    ///
    /// assert_eq!(to::string(42), "forty two");
    ///
    /// assert_eq!(to::string(-42), "negative forty two");
    ///
    /// assert_eq!(to::int(&to::string(42)).unwrap(), 42);
    ///
    /// assert_eq!(to::string(to::int("forty two").unwrap()), "forty two");
    /// ```
    pub fn string(num: isize) -> String {
        let mut num_internal = num;
        if num_internal == 0 {
            return String::from("zero");
        }
        let word_lists = load_words_list("string");

        let mut is_neg = false;
        if num_internal < 0 {
            num_internal = num_internal * -1;
            is_neg = true;
        }
        let mut current_scale = 0;
        let mut words: String = String::new();
        while num_internal > 0 {
            if num_internal % 1000 != 0 {
                words = to_string_helper(num_internal % 1000, &word_lists)
                    + &word_lists.scales[current_scale]
                    + " "
                    + &words[..];
            }
            num_internal /= 1000;
            current_scale += 1;
        }
        if is_neg {
            words = String::from("negative ") + &words[..]
        }
        String::from(words.trim())
    }

    fn to_string_helper(num: isize, word_lists: &WordLists) -> String {
        if num == 0 {
            String::from("")
        } else if num < 20 {
            String::from(&word_lists.units[num as usize]) + " "
        } else if num < 100 {
            String::from(&word_lists.tens[(num / 10) as usize])
                + " "
                + to_string_helper(num % 10, word_lists).as_str()
        } else {
            String::from(&word_lists.units[(num / 100) as usize])
                + " hundred "
                + to_string_helper(num % 100, word_lists).as_str()
        }
    }

    ///This function takes an English name of an integer and returns its integer value as an isize.
    ///
    /// Will still convert for input errors a distance of 1 from the correct spelling of the English name.
    ///
    /// Will offer suggestion on possible word if input error is greater than a distance of 1 from the correct spelling of the English name.
    ///
    /// Returns
    /// Ok(isize),
    /// Err(String)
    ///
    /// ```
    /// # use to_int_and_back::to;
    ///
    /// assert_eq!(to::int("forty two").unwrap(), 42);
    ///
    /// assert_eq!(to::int("negative forty two").unwrap(), -42);
    ///
    /// assert_eq!(to::int("frty twoo").unwrap(), 42);
    ///
    /// assert_eq!(to::int("fty two").unwrap_err(), "Did you mean forty?");
    ///```
    pub fn int(text_num: &str) -> Result<isize, String> {
        let text_num_inner = &text_num.to_lowercase()[..];

        let mut num_words: HashMap<&str, (isize, isize)> = HashMap::new();

        let word_lists = load_words_list("int");

        let mut all_words: Vec<&str> = Vec::with_capacity(
            word_lists.units.len() + word_lists.tens.len() + word_lists.scales.len() + 1,
        );

        num_words.insert("and", (1, 0));
        all_words.push("and");

        for (index, word) in word_lists.units.iter().enumerate() {
            num_words.insert(word, (1, index as isize));
            all_words.push(word);
        }
        for (index, word) in word_lists.tens.iter().enumerate() {
            num_words.insert(word, (1, index as isize * 10));
            all_words.push(word);
        }
        let num: usize = 10;
        for (index, word) in word_lists.scales.iter().enumerate() {
            num_words.insert(word, (num.pow(get_power(index)) as isize, 0));
            all_words.push(word);
        }
        let mut current = 0;
        let mut result = 0;
        let mut multipler = 1;
        for (i, word) in text_num_inner.split_whitespace().enumerate() {
            if min_distance(word, "negative") < 3 {
                if multipler == 1 && i == 0 {
                    multipler = -1;
                    continue;
                } else {
                    return Err(String::from("Invalid input"));
                }
            }
            let word_next = if !all_words.contains(&word) {
                match find_matching_word(&word, &all_words) {
                    Ok(word) => word,
                    Err(e) => return Err(e),
                }
            } else {
                String::from(word)
            };

            let (scale, increment) = num_words[word_next.as_str()];
            current = current * scale + increment;
            if scale > 100 {
                result += current;
                current = 0;
            }
        }
        Ok((result + current) * multipler)
    }
    fn get_power(num: usize) -> u32 {
        if num == 0 {
            2
        } else {
            (num * 3) as u32
        }
    }

    fn find_matching_word(word: &str, words: &Vec<&str>) -> Result<String, String> {
        let mut min_dist = 9999;
        let mut final_string = String::new();
        for w in words {
            let distance = min_distance(w, word);
            if distance < min_dist {
                min_dist = distance;
                final_string = String::from(*w);
            }
        }
        if min_dist > MAX_STRING_DISTANCE {
            return if min_distance(word, "negative") < 5 {
                Err(String::from("Did you mean negative?"))
            } else {
                Err(String::from(format!("Did you mean {}?", final_string)))
            }
        }
        Ok(final_string)
    }

    fn min_distance(word1: &str, word2: &str) -> i32 {
        let (word1, word2) = (word1.as_bytes(), word2.as_bytes());
        let mut dist = Vec::with_capacity(word2.len() + 1);
        for j in 0..=word2.len() {
            dist.push(j)
        }
        let mut prev_dist = dist.clone();
        for i in 1..=word1.len() {
            for j in 0..=word2.len() {
                if j == 0 {
                    dist[j] += 1;
                } else if word1[i - 1] == word2[j - 1] {
                    dist[j] = prev_dist[j - 1];
                } else {
                    dist[j] = dist[j].min(dist[j - 1]).min(prev_dist[j - 1]) + 1;
                }
            }
            prev_dist.copy_from_slice(&dist);
        }
        dist[word2.len()] as i32
    }

    use serde::Deserialize;

    use std::fs::File;
    use std::io::Read;
    use std::path::Path;

    #[derive(Deserialize)]
    struct WordLists {
        units: Vec<String>,
        tens: Vec<String>,
        scales: Vec<String>,
    }

    fn load_words_list(name: &str) -> WordLists {
        let path_name = format!("{}_words.toml", name);
        let path = Path::new(&path_name);
        let display = path.display();
        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(why) => panic!("couldn't open {}: {}", display, why),
        };
        let mut file_content = String::new();
        file.read_to_string(&mut file_content).unwrap();
        toml::from_str(&file_content).unwrap()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn tests_should_not_panic_to_int() {
            assert_eq!(int("One Hundred And Forty Two").unwrap(), 142);
            assert_eq!(
                int(
                    "one million four hundred twenty seven thousand four hundred and seventy three"
                )
                .unwrap(),
                1_427_473
            );
            assert_eq!(
                int("negative seven thousand three hundred and ninety six").unwrap(),
                -7396
            );
            assert_eq!(int("negativ three hundre and fifty fiv").unwrap(), -355);
            assert_eq!(
                int(&string(-123_456_789_098_765_432)).unwrap(),
                -123_456_789_098_765_432
            );
        }

        #[test]
        fn tests_should_not_panic_to_string() {
            assert_eq!(string(142), "one hundred forty two");
            assert_eq!(
                string(1_427_473),
                "one million four hundred twenty seven thousand four hundred seventy three"
            );
            assert_eq!(
                string(-7396),
                "negative seven thousand three hundred ninety six"
            );
            assert_eq!(string(-355), "negative three hundred fifty five");
            assert_eq!(
                string(int("negative twenty seven thousand eight hundred sixty nine").unwrap()),
                "negative twenty seven thousand eight hundred sixty nine"
            );
        }

        #[test]
        fn min_distance_tests() {
            assert_eq!(min_distance("big fish", "big wish"), 1);
            assert_eq!(min_distance("negative", "negativ"), 1);
            assert_eq!(min_distance("negative", "ngtiv"), 3);
            assert_eq!(min_distance("hello", "goodbye"), 7);
        }

        #[test]
        fn find_matching_word_tests() {
            let words = vec!["gourami", "tetra", "snail"];
            assert_eq!(find_matching_word("gorami", &words).unwrap(), "gourami");
            assert_eq!(
                find_matching_word("gormi", &words).unwrap_err(),
                "Did you mean gourami?"
            );
            assert_eq!(find_matching_word("teta", &words).unwrap(), "tetra");
            assert_eq!(
                find_matching_word("tet", &words).unwrap_err(),
                "Did you mean tetra?"
            );
            assert_eq!(find_matching_word("sail", &words).unwrap(), "snail");
            assert_eq!(
                find_matching_word("sal", &words).unwrap_err(),
                "Did you mean snail?"
            );
        }

        #[test]
        fn test_error_messages() {
            assert_eq!(
                int("one hured and forty two").unwrap_err(),
                "Did you mean hundred?"
            );
            assert_eq!(
                int("ngtiv one hundred forty two").unwrap_err(),
                "Did you mean negative?"
            );
            assert_eq!(
                int("negative negative one hundred forty two").unwrap_err(),
                "Invalid input"
            );
            assert_eq!(int("ten negative").unwrap_err(), "Invalid input");
        }

        #[test]
        fn load_words_list_tests() {
            load_words_list("string");
            load_words_list("int");
        }

        #[test]
        #[should_panic]
        fn load_words_list_should_panic() {
            load_words_list("foo");
            load_words_list("bar");
        }
    }
}
