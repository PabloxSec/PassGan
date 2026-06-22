use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;
use serde::{Serialize, Deserialize};
use rand::Rng;

#[derive(Serialize, Deserialize)]
pub struct MarkovModel {
    pub order: usize,
    // transition matrix: key is the prefix string of length `order`,
    // value is a vector of (next_char, count)
    pub transitions: HashMap<String, Vec<(char, u32)>>,
    // starting sequences of length `order` with count
    pub starts: Vec<(String, u32)>,
}

impl MarkovModel {
    pub fn new(order: usize) -> Self {
        Self {
            order,
            transitions: HashMap::new(),
            starts: Vec::new(),
        }
    }
    
    pub fn train<P: AsRef<Path>>(&mut self, filepath: P) -> std::io::Result<()> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let mut starts_map: HashMap<String, u32> = HashMap::new();
        let mut transitions_map: HashMap<String, HashMap<char, u32>> = HashMap::new();

        let sop_char = '\x02'; // Start of password padding
        let eop_char = '\x03'; // End of password padding

        for line in reader.lines() {
            let line = line?;
            let pwd = line.trim();
            if pwd.is_empty() {
                continue;
            }
            
            // Pad start and end. SOP is padded `order` times.
            let padded_start = sop_char.to_string().repeat(self.order);
            let padded = format!("{}{}{}", padded_start, pwd, eop_char);
            let chars: Vec<char> = padded.chars().collect();
            
            // Count starts (the first n-gram)
            if chars.len() >= self.order {
                let start_ngram: String = chars[..self.order].iter().collect();
                *starts_map.entry(start_ngram).or_insert(0) += 1;
            }
            
            // Build transitions
            for i in 0..chars.len() - self.order {
                let ngram: String = chars[i..i+self.order].iter().collect();
                let next_char = chars[i + self.order];
                
                let state_entry = transitions_map.entry(ngram).or_insert_with(HashMap::new);
                *state_entry.entry(next_char).or_insert(0) += 1;
            }
        }
        
        // Convert starts map to a vector
        self.starts = starts_map.into_iter().collect();
        self.starts.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Convert transitions map to sorted vectors
        self.transitions = transitions_map
            .into_iter()
            .map(|(state, next_map)| {
                let mut v: Vec<(char, u32)> = next_map.into_iter().collect();
                v.sort_by(|a, b| b.1.cmp(&a.1));
                (state, v)
            })
            .collect();
            
        Ok(())
    }
    
    pub fn save<P: AsRef<Path>>(&self, filepath: P) -> std::io::Result<()> {
        let file = File::create(filepath)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, self)?;
        Ok(())
    }
    
    pub fn load<P: AsRef<Path>>(filepath: P) -> std::io::Result<Self> {
        let file = File::open(filepath)?;
        let reader = BufReader::new(file);
        let model = serde_json::from_reader(reader)?;
        Ok(model)
    }
    
    pub fn generate(&self, min_len: usize, max_len: usize) -> Option<String> {
        let mut rng = rand::thread_rng();
        
        // Choose a starting sequence based on frequencies
        let start_ngram = select_weighted(&self.starts, &mut rng)?;
        
        // Remove the start characters (SOP) to form the actual password string
        let mut result = start_ngram.replace('\x02', "");
        let mut current_state = start_ngram;
        
        while result.len() < max_len {
            if let Some(next_chars) = self.transitions.get(&current_state) {
                let next_char = select_weighted(next_chars, &mut rng)?;
                if next_char == '\x03' {
                    break;
                }
                result.push(next_char);
                
                // update current state
                let mut state_chars: Vec<char> = current_state.chars().collect();
                state_chars.remove(0);
                state_chars.push(next_char);
                current_state = state_chars.into_iter().collect();
            } else {
                break;
            }
        }
        
        if result.len() >= min_len {
            Some(result)
        } else {
            None
        }
    }
}

fn select_weighted<T: Clone, R: Rng>(choices: &[(T, u32)], rng: &mut R) -> Option<T> {
    if choices.is_empty() {
        return None;
    }
    let total_weight: u32 = choices.iter().map(|(_, w)| w).sum();
    if total_weight == 0 {
        return None;
    }
    let mut val = rng.gen_range(0..total_weight);
    for (choice, weight) in choices {
        if val < *weight {
            return Some(choice.clone());
        }
        val -= weight;
    }
    choices.first().map(|(c, _)| c.clone())
}
