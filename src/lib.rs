//!This library will convert English number names to their integer equivalent and vice versa.
//!
//! Uses Levenshtein distance algorithm to accept words a distance of 1 from a valid spelling.

pub mod to {
    use std::collections::HashMap;

    const MAX_STRING_DISTANCE: i32 = 1;

    /// This function takes a floating point number and returns its English name
    ///
    /// Takes two arguments, number to convert (num) and precision of decimal (precision)
    ///
    /// Experimental!
    /// ```
    ///# use to_int_and_back::to;
    ///
    /// assert_eq!(to::string_f(3.14, 2), "three point one four");
    ///
    /// assert_eq!(to::string_f(-42.53, 2), "negative forty two point five three");
    ///
    /// assert_eq!(to::string_f(1.539323, 2), "one point five four");
    ///```
    pub fn string_f(num: f32, precision: u8) -> String {
        let mut head = num.floor();
        let mut tail = num % 1_f32;

        return if tail == 0_f32 {
            string(head as isize)
        } else if precision == 0 {
            string(num.round() as isize)
        }
        else {
            if num < 0_f32 {
                head += 1_f32;
                tail = tail.abs();
            }

            tail = (tail * 10_f32.powf(precision as f32)).round();
            tail = tail / (10_f32.powf(precision as f32));

            let mut tail_string = String::from(" point");
            for c in tail.to_string().trim().chars().skip(2) {
                tail_string += match c {
                    '0' => " zero",
                    '1' => " one",
                    '2' => " two",
                    '3' => " three",
                    '4' => " four",
                    '5' => " five",
                    '6' => " six",
                    '7' => " seven",
                    '8' => " eight",
                    '9' => " nine",
                    _ => panic!("unreachable"),
                };
            }
            string(head as isize) + &*tail_string
        }
    }

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

    ///This function takes an English name of a floating point number and returns its value as an f32.
    ///
    /// Returns
    /// Ok(f32),
    /// Err(String)
    ///
    /// ```
    /// # use to_int_and_back::to;
    ///
    /// assert_eq!(to::float("forty two point seven").unwrap(), 42.7 as f32);
    ///
    /// assert_eq!(to::float("negative forty two point nine").unwrap(), -42.9 as f32);
    ///
    /// assert_eq!(to::float("frty twoo point zero").unwrap(), 42.0 as f32);
    ///
    /// assert_eq!(to::float("fty two").unwrap_err(), "Did you mean forty?");
    ///```
    pub fn float(text_num: &str) -> Result<f32, String> {
        let text_num_inner = &text_num.to_lowercase()[..];

        for w in text_num_inner.split_whitespace() {
            if min_distance(w, "point") == 1 {
                return Err(String::from("Did you mean point?"));
            }
        }

        return if text_num_inner.contains("point") {
            let head_string = text_num_inner.split("point").collect::<Vec<&str>>()[0].trim();
            let tail_string = text_num_inner.split("point").collect::<Vec<&str>>()[1].trim();
            let mut tail_string_digs = String::from("0.");
            for c in tail_string.split_whitespace() {
                let dig = match int(&c) {
                    Ok(num) => num as u32,
                    Err(e) => {
                        return Err(e);
                    }
                };
                let ch = match char::from_digit(dig, 10) {
                    None => {
                        return Err(String::from("Invalid value in tail string."));
                    }
                    Some(v) => v
                };
                tail_string_digs.push(ch);
            }
            let h = match int(head_string) {
                Ok(num) => num as f32,
                Err(e) => {
                    return Err(e);
                }
            };
            let t = tail_string_digs.trim().parse::<f32>().unwrap();
            return if h < 0_f32 {
                Ok(h - t)
            } else {
                Ok(h + t)
            };
        } else {
            match int(text_num) {
                Ok(num) => Ok(num as f32),
                Err(e) => Err(e)
            }
        }
    }

    ///This function takes an English name of an integer and returns its value as an isize.
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
        fn float_test() {
            assert_eq!(float("negative three point five").unwrap(), -3.5 as f32);
            assert_eq!(float("forty too point four two").unwrap(), 42.42 as f32);
            assert_eq!(float("zero point four two").unwrap(), 0.42 as f32);
            assert_eq!(float("point four two").unwrap(), 0.42 as f32);
            assert_eq!(float("three point five").unwrap(), 3.5 as f32);
            assert_eq!(float("three").unwrap(), 3 as f32);

            assert_eq!(
                float(
                    "one million four hundred twenty seven thousand four hundred and seventy three
                    point seven six nine nine eight"
                )
                    .unwrap(),
                1_427_473.769_98 as f32
            );
            assert_eq!(float("three thosad point four two").unwrap_err(), "Did you mean thousand?");
            assert_eq!(float("three point sixty two").unwrap_err(), "Invalid value in tail string.");
            assert_eq!(float("three poin sixty two").unwrap_err(), "Did you mean point?");
        }

        #[test]
        fn string_f_test() {
            assert_eq!(string_f(3_f32, 0), "three");
            assert_eq!(string_f(-3_f32, 0), "negative three");
            assert_eq!(string_f(-4.000, 0), "negative four");
            assert_eq!(string_f(-4.000, 3), "negative four");
            assert_eq!(string_f(-33.53, 2), "negative thirty three point five three");
            assert_eq!(string_f(3.44, 1), "three point four");
            assert_eq!(string_f(3.45, 1), "three point five");
            assert_eq!(string_f(3.4, 0), "three");
            assert_eq!(string_f(3.5, 0), "four");
            assert_eq!(string_f(-3.44, 1), "negative three point four");
            assert_eq!(string_f(-3.45, 1), "negative three point five");
            assert_eq!(string_f(-3.4, 0), "negative three");
            assert_eq!(string_f(-3.5, 0), "negative four");
            assert_eq!(string_f(1427473.75 as f32, 2),
            "one million four hundred twenty seven thousand four hundred seventy three point seven five"
            );
            assert_eq!(string_f(14.274737 as f32, 2),
                       "fourteen point two seven"
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
