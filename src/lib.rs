//!This library will convert English number names to their integer equivalent and vice versa.
//!
//! Support includes negative numbers and typos in the English name input.
//!
//! Uses Levenshtein distance algorithm to accept words a distance of 1 from a valid spelling.

pub mod to {
    use std::collections::HashMap;

    ///This function takes an integer and returns its English name.
    ///
    /// 42 -> "forty two"
    ///
    /// ```
    /// use to_int_and_back::to;
    ///
    /// assert_eq!(to::string(42), "forty two");
    ///
    /// assert_eq!(to::int(&to::string(42)), 42);
    ///
    /// assert_eq!(to::string(to::int("forty two")), "forty two");
    /// ```
    pub fn string(num: isize) -> String {
        let mut num_internal = num;
        if num_internal == 0 {
            return String::from("zero");
        }
        let units = [
            "",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];

        let tens = [
            "", "ten", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];

        let scales = [
            "",
            "thousand",
            "million",
            "billion",
            "trillion",
            "quadrillion",
        ];

        let mut is_neg = false;
        if num_internal < 0 {
            num_internal = num_internal * -1;
            is_neg = true;
        }
        let mut current_scale = 0;
        let mut words: String = String::from("");
        while num_internal > 0 {
            if num_internal % 1000 != 0 {
                words = to_string_helper(num_internal % 1000, &units, &tens)
                    + scales[current_scale]
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

    fn to_string_helper(num: isize, units: &[&str], tens: &[&str]) -> String {
        if num == 0 {
            String::from("")
        } else if num < 20 {
            String::from(units[num as usize]) + " "
        } else if num < 100 {
            String::from(tens[(num / 10) as usize])
                + " "
                + to_string_helper(num % 10, &units, &tens).as_str()
        } else {
            String::from(units[(num / 100) as usize])
                + " hundred "
                + to_string_helper(num % 100, &units, &tens).as_str()
        }
    }

    ///This function takes an English name of an integer and returns it as an isize.
    ///
    /// Will still convert for input errors a distance of 1 from the correct spelling of the English name.
    ///
    /// "forty two" -> 42
    ///
    /// ```
    /// use to_int_and_back::to;
    ///
    /// assert_eq!(to::int("forty two"), 42);
    ///
    /// assert_eq!(to::int("frty twoo"), 42);
    ///```
    /// ```should_panic
    /// use to_int_and_back::to;
    ///
    /// assert_eq!(to::int("fty twwoo"), 42);
    ///```
    pub fn int(text_num: &str) -> isize {
        let text_num_inner = &text_num.to_lowercase()[..];
        let mut num_words: HashMap<&str, (isize, isize)> = HashMap::new();
        let units = [
            "zero",
            "one",
            "two",
            "three",
            "four",
            "five",
            "six",
            "seven",
            "eight",
            "nine",
            "ten",
            "eleven",
            "twelve",
            "thirteen",
            "fourteen",
            "fifteen",
            "sixteen",
            "seventeen",
            "eighteen",
            "nineteen",
        ];
        let tens = [
            "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
        ];
        let scales = [
            "hundred",
            "thousand",
            "million",
            "billion",
            "trillion",
            "quadrillion",
        ];
        num_words.insert("and", (1, 0));

        for (index, word) in units.iter().enumerate() {
            num_words.insert(*word, (1, index as isize));
        }
        for (index, word) in tens.iter().enumerate() {
            num_words.insert(*word, (1, index as isize * 10));
        }
        let num: usize = 10;
        for (index, word) in scales.iter().enumerate() {
            num_words.insert(*word, (num.pow(get_power(index)) as isize, 0));
        }
        let mut current = 0;
        let mut result = 0;
        let mut multipler = 1;
        for word in text_num_inner.split_whitespace() {
            if min_distance(word, "negative") < 3 {
                if multipler == 1 {
                    multipler = -1;
                    continue;
                } else {
                    panic!("Invalid input");
                }
            }
            let word_next = if !num_words.contains_key(&word) {
                find_possible_matches(&word, &num_words)
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
        (result + current) * multipler
    }
    fn get_power(num: usize) -> u32 {
        if num == 0 {
            2
        } else {
            (num * 3) as u32
        }
    }

    fn find_possible_matches(word: &str, words: &HashMap<&str, (isize, isize)>) -> String {
        let mut min_dist = 9999;
        let mut final_string: String = String::from("");
        for (w, _) in words {
            let distance = min_distance(w, word);
            if distance < min_dist {
                min_dist = distance;
                final_string = String::from(*w);
            }
        }
        if min_dist > 1 {
            if min_distance(word, "negative") < 5 {
                println!("Did you mean negative?");
            } else {
                println!("Did you mean {}?", final_string);
            }
            panic!("Invalid input")
        }
        final_string
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
}

#[cfg(test)]
mod tests {
    use super::to;
    #[test]
    fn tests_should_not_panic() {
        assert_eq!(to::int("One Hundred And Forty Two"), 142);
        assert_eq!(
            to::int(
                "one million four hundred twenty seven thousand four hundred and seventy three"
            ),
            1_427_473
        );
        assert_eq!(
            to::int("negative seven thousand three hundred and ninety six"),
            -7396
        );
        assert_eq!(to::int("negativ three hundre and fifty fiv"), -355);
        assert_eq!(
            to::int(&to::string(-123_456_789_098_765_432)),
            -123_456_789_098_765_432
        );
    }

    #[test]
    fn tests_should_not_panic_int() {
        assert_eq!(to::string(142), "one hundred forty two");
        assert_eq!(
            to::string(1_427_473),
            "one million four hundred twenty seven thousand four hundred seventy three"
        );
        assert_eq!(
            to::string(-7396),
            "negative seven thousand three hundred ninety six"
        );
        assert_eq!(to::string(-355), "negative three hundred fifty five");
        assert_eq!(
            to::string(to::int(
                "negative twenty seven thousand eight hundred sixty nine"
            )),
            "negative twenty seven thousand eight hundred sixty nine"
        );
    }

    #[test]
    #[should_panic(expected = "Invalid input")]
    fn tests_should_panic() {
        assert_eq!(to::int("one hured and forty two"), 142);
    }
}
