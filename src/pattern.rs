use rand::Rng;

pub fn generate_real_world_password(min_len: usize, max_len: usize) -> String {
    let mut rng = rand::thread_rng();
    
    // Syllables to make it look like a pronounceable human-like word
    let consonants = ["b", "c", "d", "f", "g", "h", "j", "k", "l", "m", "n", "p", "r", "s", "t", "v", "w", "x", "y", "z", "ch", "sh", "th", "br", "cr", "dr", "fr", "gr", "pr", "tr", "st"];
    let vowels = ["a", "e", "i", "o", "u", "ea", "ee", "oo", "ou", "ai", "ay"];
    
    let special_chars = ["!", "@", "#", "$", "%", "*", "?", "_", "."];
    let common_years = ["2020", "2021", "2022", "2023", "2024", "2025", "1990", "1995", "1998", "1999", "2000", "2001", "2005", "2010"];
    
    // Choose a target length
    let target_len = if min_len == max_len {
        min_len
    } else {
        rng.gen_range(min_len..=max_len)
    };
    
    // Safeguard minimum length
    let target_len = target_len.max(4);
    
    // We will generate the password step-by-step
    // Templates:
    // 0: Word + Digits
    // 1: Word + Symbol
    // 2: Word + Digits + Symbol
    // 3: Word + Symbol + Digits
    // 4: Capitalized Word + lowercase Word (Dual word)
    let template = rng.gen_range(0..=4);
    
    let mut password = String::new();
    
    match template {
        0 => {
            // Word + Digits
            let digit_len = rng.gen_range(1..=4).min(target_len - 3);
            let word_len = target_len - digit_len;
            
            password.push_str(&generate_pronounceable_word(word_len, &consonants, &vowels, &mut rng));
            password.push_str(&generate_digits(digit_len, &common_years, &mut rng));
        }
        1 => {
            // Word + Symbol
            let symbol_len = rng.gen_range(1..=2).min(target_len - 3);
            let word_len = target_len - symbol_len;
            
            password.push_str(&generate_pronounceable_word(word_len, &consonants, &vowels, &mut rng));
            for _ in 0..symbol_len {
                password.push_str(special_chars[rng.gen_range(0..special_chars.len())]);
            }
        }
        2 => {
            // Word + Digits + Symbol
            let symbol_len = 1;
            let digit_len = rng.gen_range(1..=3).min(target_len - 4);
            let word_len = target_len - digit_len - symbol_len;
            
            password.push_str(&generate_pronounceable_word(word_len, &consonants, &vowels, &mut rng));
            password.push_str(&generate_digits(digit_len, &common_years, &mut rng));
            password.push_str(special_chars[rng.gen_range(0..special_chars.len())]);
        }
        3 => {
            // Word + Symbol + Digits
            let symbol_len = 1;
            let digit_len = rng.gen_range(1..=3).min(target_len - 4);
            let word_len = target_len - digit_len - symbol_len;
            
            password.push_str(&generate_pronounceable_word(word_len, &consonants, &vowels, &mut rng));
            password.push_str(special_chars[rng.gen_range(0..special_chars.len())]);
            password.push_str(&generate_digits(digit_len, &common_years, &mut rng));
        }
        _ => {
            // Dual word, capitalized separation (e.g. BlueSky, HappyDog)
            let first_word_len = target_len / 2;
            let second_word_len = target_len - first_word_len;
            
            let mut w1 = generate_pronounceable_word(first_word_len, &consonants, &vowels, &mut rng);
            let mut w2 = generate_pronounceable_word(second_word_len, &consonants, &vowels, &mut rng);
            
            capitalize_first(&mut w1);
            capitalize_first(&mut w2);
            
            password = format!("{}{}", w1, w2);
        }
    }
    
    // Capitalize the first letter for templates 0, 1, 2, 3 with high probability (80%)
    if template <= 3 && rng.gen_bool(0.8) {
        capitalize_first(&mut password);
    }
    
    // Leetspeak substitutions occasionally (15% chance)
    if rng.gen_bool(0.15) {
        password = apply_leetspeak(password, &mut rng);
    }
    
    // Ensure exact length matching in case of syllable length rounding
    if password.len() > target_len {
        password.truncate(target_len);
    } else if password.len() < target_len {
        // pad with random digit or symbol
        let padding_needed = target_len - password.len();
        for _ in 0..padding_needed {
            if rng.gen_bool(0.5) {
                password.push_str(&rng.gen_range(0..10).to_string());
            } else {
                password.push_str(special_chars[rng.gen_range(0..special_chars.len())]);
            }
        }
    }
    
    password
}

fn generate_pronounceable_word(length: usize, consonants: &[&str], vowels: &[&str], rng: &mut impl Rng) -> String {
    let mut word = String::new();
    let mut use_consonant = rng.gen_bool(0.6); // 60% chance to start with consonant
    
    while word.len() < length {
        let piece = if use_consonant {
            consonants[rng.gen_range(0..consonants.len())]
        } else {
            vowels[rng.gen_range(0..vowels.len())]
        };
        
        // Prevent overshoot
        if word.len() + piece.len() <= length {
            word.push_str(piece);
            use_consonant = !use_consonant;
        } else {
            // just append single random vowel or consonant
            if use_consonant {
                word.push_str(consonants[rng.gen_range(0..20)]); // simple single letter consonants
            } else {
                word.push_str(vowels[rng.gen_range(0..5)]); // simple single letter vowels
            }
            break;
        }
    }
    word
}

fn generate_digits(length: usize, common_years: &[&str], rng: &mut impl Rng) -> String {
    if length == 4 && rng.gen_bool(0.5) {
        // Use a common year
        return common_years[rng.gen_range(0..common_years.len())].to_string();
    }
    
    let mut digits = String::new();
    if length == 3 && rng.gen_bool(0.6) {
        return "123".to_string();
    }
    if length == 2 && rng.gen_bool(0.6) {
        return "12".to_string();
    }
    
    for _ in 0..length {
        digits.push_str(&rng.gen_range(0..10).to_string());
    }
    digits
}

fn capitalize_first(s: &mut String) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

fn apply_leetspeak(s: String, rng: &mut impl Rng) -> String {
    s.chars().map(|c| {
        match c {
            'a' | 'A' if rng.gen_bool(0.5) => '@',
            'o' | 'O' if rng.gen_bool(0.5) => '0',
            'e' | 'E' if rng.gen_bool(0.5) => '3',
            'i' | 'I' if rng.gen_bool(0.5) => '1',
            's' | 'S' if rng.gen_bool(0.5) => '$',
            't' | 'T' if rng.gen_bool(0.5) => '7',
            other => other
        }
    }).collect()
}
