mod markov;
mod pattern;

use std::fs::File;
use std::io::{self, Write, BufWriter};
use std::process::Command;
use std::path::Path;
use dialoguer::{theme::ColorfulTheme, Select, Input, Confirm};
use indicatif::{ProgressBar, ProgressStyle};
use markov::MarkovModel;
use dialoguer::console::Term;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Check if CLI flags were passed
    if args.len() > 1 {
        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
            print_help();
            return Ok(());
        }

        // Parse CLI arguments
        let mut mode = None;
        let mut input = None;
        let mut output = None;
        let mut model = None;
        let mut count = 1000000;
        let mut length = "10-12".to_string();
        let mut order = 3;
        let mut temp = 0.95;
        let mut gpu = false;

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--mode" => {
                    if i + 1 < args.len() {
                        mode = Some(args[i+1].clone());
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --mode");
                        return Ok(());
                    }
                }
                "--input" => {
                    if i + 1 < args.len() {
                        input = Some(args[i+1].clone());
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --input");
                        return Ok(());
                    }
                }
                "--output" => {
                    if i + 1 < args.len() {
                        output = Some(args[i+1].clone());
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --output");
                        return Ok(());
                    }
                }
                "--model" => {
                    if i + 1 < args.len() {
                        model = Some(args[i+1].clone());
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --model");
                        return Ok(());
                    }
                }
                "--count" => {
                    if i + 1 < args.len() {
                        count = args[i+1].parse::<u64>().unwrap_or(1000000);
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --count");
                        return Ok(());
                    }
                }
                "--length" => {
                    if i + 1 < args.len() {
                        length = args[i+1].clone();
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --length");
                        return Ok(());
                    }
                }
                "--order" => {
                    if i + 1 < args.len() {
                        order = args[i+1].parse::<usize>().unwrap_or(3);
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --order");
                        return Ok(());
                    }
                }
                "--temp" => {
                    if i + 1 < args.len() {
                        temp = args[i+1].parse::<f64>().unwrap_or(0.95);
                        i += 2;
                    } else {
                        println!("[!] Error: Missing value for --temp");
                        return Ok(());
                    }
                }
                "--gpu" => {
                    gpu = true;
                    i += 1;
                }
                _ => {
                    println!("[!] Unknown argument: {}", args[i]);
                    print_help();
                    return Ok(());
                }
            }
        }

        let mode_str = match mode {
            Some(m) => m,
            None => {
                println!("[!] Error: --mode is required when running in CLI mode.");
                print_help();
                return Ok(());
            }
        };

        return run_cli_mode(&mode_str, input, output, model, count, length, order, temp, gpu);
    }

    // Interactive TTY fallback check
    if !Term::stdout().is_term() {
        println!("====================================================");
        println!("     DEEPPASSGEN - HYBRID PASSWORD SYNTHESIS ENGINE ");
        println!("====================================================");
        println!("[!] Error: Interactive mode requires an active terminal (TTY).");
        println!("[*] Use --help or pass command line flags to run in script/pipeline mode.");
        println!();
        print_help();
        return Ok(());
    }

    println!("====================================================");
    println!("     DEEPPASSGEN - HYBRID PASSWORD SYNTHESIS ENGINE ");
    println!("====================================================");
    println!("   System: Python/PyTorch (AI) & Rust (Markov/Pattern) ");
    println!("====================================================\n");

    loop {
        let selections = &[
            "[1] AI PyTorch Generator (PassGAN GRU Engine)",
            "[2] Markov Chain Generator (Rust Native - Ultra Fast)",
            "[3] Real-World Random Generator (Human Pattern-Based)",
            "[4] Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose operations category")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        match selection {
            0 => ai_menu()?,
            1 => markov_menu()?,
            2 => pattern_menu()?,
            3 => {
                println!("\n[*] Exiting DeepPassGen Engine. Goodbye!");
                break;
            }
            _ => unreachable!(),
        }
        println!();
    }

    Ok(())
}

fn print_help() {
    println!("DeepPassGen - Command Line Interface Usage:");
    println!("  passgan.exe [OPTIONS]");
    println!();
    println!("Options:");
    println!("  --mode <MODE>       Mode selection:");
    println!("                        ai-train     : Train the PyTorch GRU model");
    println!("                        ai-gen       : Generate passwords using PyTorch GRU");
    println!("                        markov-train : Train the native Rust Markov model");
    println!("                        markov-gen   : Generate passwords using Markov model");
    println!("                        pattern-gen  : Generate real-world random passwords");
    println!("  --input <PATH>      Path to input wordlist file (required for train modes)");
    println!("  --output <PATH>     Path to save generated passwords");
    println!("  --model <PATH>      Path to model file (.pt for AI, .json for Markov)");
    println!("  --count <NUM>       Number of passwords to generate (default: 1,000,000)");
    println!("  --length <RANGE>    Generated password length range (default: 10-12)");
    println!("  --order <NUM>       Markov chain history order (default: 3)");
    println!("  --temp <FLOAT>      AI generation temperature/creativity (default: 0.95)");
    println!("  --gpu               Enable GPU (CUDA) acceleration for AI generation");
    println!("  -h, --help          Show this help menu");
    println!();
    println!("Examples:");
    println!("  passgan.exe --mode pattern-gen --count 100000 --length 10-12 --output sample.txt");
    println!("  passgan.exe --mode ai-train --input passwords.txt --model passgan.pt");
}

fn run_cli_mode(
    mode: &str,
    input: Option<String>,
    output: Option<String>,
    model_path: Option<String>,
    count: u64,
    length: String,
    order: usize,
    temp: f64,
    gpu: bool,
) -> io::Result<()> {
    match mode {
        "ai-train" => {
            let input_file = input.unwrap_or_else(|| "test.txt".to_string());
            let model = model_path.unwrap_or_else(|| "passgan_model.pt".to_string());
            if !Path::new(&input_file).exists() {
                println!("[!] Error: Input training file '{}' not found.", input_file);
                return Ok(());
            }
            println!("[*] Spawning PyTorch Training Subprocess...");
            let script_path = find_python_script();
            let mut args = vec![
                script_path.to_str().unwrap(),
                "--mode", "train",
                "--input-file", &input_file,
                "--model-path", &model,
            ];
            if gpu {
                args.push("--gpu");
            }
            run_python_script(&args)?;
        }
        "ai-gen" => {
            let model = model_path.unwrap_or_else(|| "passgan_model.pt".to_string());
            let out = output.unwrap_or_else(|| "generated_passwords.txt".to_string());
            if !Path::new(&model).exists() {
                println!("[!] Error: Model checkpoint file '{}' not found.", model);
                return Ok(());
            }
            println!("[*] Spawning PyTorch Generation Subprocess...");
            let script_path = find_python_script();
            let count_str = count.to_string();
            let temp_str = temp.to_string();
            let mut args = vec![
                script_path.to_str().unwrap(),
                "--mode", "generate",
                "--model-path", &model,
                "--output-file", &out,
                "--count", &count_str,
                "--temp", &temp_str,
            ];
            if gpu {
                args.push("--gpu");
            }
            run_python_script(&args)?;
        }
        "markov-train" => {
            let input_file = input.unwrap_or_else(|| "test.txt".to_string());
            let model = model_path.unwrap_or_else(|| "markov_model.json".to_string());
            if !Path::new(&input_file).exists() {
                println!("[!] Error: Input training file '{}' not found.", input_file);
                return Ok(());
            }
            println!("[*] Training Rust Markov Engine on '{}'...", input_file);
            let mut markov_model = MarkovModel::new(order);
            markov_model.train(&input_file)?;
            markov_model.save(&model)?;
            println!("[+] Markov model trained and saved to '{}'", model);
        }
        "markov-gen" => {
            let model_file = model_path.unwrap_or_else(|| "markov_model.json".to_string());
            let out = output.unwrap_or_else(|| "markov_passwords.txt".to_string());
            if !Path::new(&model_file).exists() {
                println!("[!] Error: Markov model file '{}' not found.", model_file);
                return Ok(());
            }
            let (min_len, max_len) = parse_range(&length).unwrap_or((10, 12));
            println!("[*] Loading Markov model from '{}'...", model_file);
            let model = MarkovModel::load(&model_file)?;
            
            println!("[*] Generating {} passwords with lengths {}-{} in Rust...", format_thousands(count), min_len, max_len);
            let file = File::create(&out)?;
            let mut writer = BufWriter::with_capacity(1024 * 1024, file);
            let mut generated = 0;
            while generated < count {
                if let Some(pwd) = model.generate(min_len, max_len) {
                    writeln!(writer, "{}", pwd)?;
                    generated += 1;
                }
            }
            writer.flush()?;
            println!("[+] Saved {} passwords to '{}'", format_thousands(generated), out);
        }
        "pattern-gen" => {
            let out = output.unwrap_or_else(|| "pattern_passwords.txt".to_string());
            let (min_len, max_len) = parse_range(&length).unwrap_or((10, 12));
            println!("[*] Generating {} real-world pattern passwords with lengths {}-{} in Rust...", format_thousands(count), min_len, max_len);
            let file = File::create(&out)?;
            let mut writer = BufWriter::with_capacity(1024 * 1024, file);
            for _ in 0..count {
                let pwd = pattern::generate_real_world_password(min_len, max_len);
                writeln!(writer, "{}", pwd)?;
            }
            writer.flush()?;
            println!("[+] Saved {} passwords to '{}'", format_thousands(count), out);
        }
        _ => {
            println!("[!] Error: Unknown mode '{}'", mode);
            print_help();
        }
    }
    Ok(())
}

fn ai_menu() -> io::Result<()> {
    loop {
        let selections = &[
            "[a] Train New AI Model (PyTorch)",
            "[b] Generate Passwords from Saved Model",
            "[c] Back to Main Menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("AI PyTorch Engine Operations")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        match selection {
            0 => {
                let input_file: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to training text file (wordlist)")
                    .default("test.txt".to_string())
                    .interact_text()
                    .unwrap();

                if !Path::new(&input_file).exists() {
                    println!("[!] Error: Input file '{}' not found.", input_file);
                    continue;
                }

                let model_path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to save trained model checkpoint")
                    .default("passgan_model.pt".to_string())
                    .interact_text()
                    .unwrap();

                let epochs: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Number of training epochs")
                    .default(3)
                    .interact_text()
                    .unwrap();

                let batch_size: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Training batch size")
                    .default(1024)
                    .interact_text()
                    .unwrap();

                let hidden_size: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("GRU hidden layer size")
                    .default(128)
                    .interact_text()
                    .unwrap();

                let embed_size: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Character embedding size")
                    .default(32)
                    .interact_text()
                    .unwrap();

                let layers: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Number of GRU layers")
                    .default(1)
                    .interact_text()
                    .unwrap();

                let lr: f64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Learning rate")
                    .default(0.005)
                    .interact_text()
                    .unwrap();

                let use_gpu = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Use GPU for training if available?")
                    .default(true)
                    .interact()
                    .unwrap();

                println!("\n[*] Spawning PyTorch Training Subprocess...");
                let script_path = find_python_script();
                
                let epochs_str = epochs.to_string();
                let batch_size_str = batch_size.to_string();
                let hidden_size_str = hidden_size.to_string();
                let embed_size_str = embed_size.to_string();
                let layers_str = layers.to_string();
                let lr_str = lr.to_string();

                let mut args = vec![
                    script_path.to_str().unwrap(),
                    "--mode", "train",
                    "--input-file", &input_file,
                    "--model-path", &model_path,
                    "--epochs", &epochs_str,
                    "--batch-size", &batch_size_str,
                    "--hidden-size", &hidden_size_str,
                    "--embed-size", &embed_size_str,
                    "--layers", &layers_str,
                    "--lr", &lr_str,
                ];
                if use_gpu {
                    args.push("--gpu");
                }

                let success = run_python_script(&args)?;

                if success {
                    println!("\n[+] AI Model successfully trained and saved to '{}'", model_path);
                } else {
                    println!("\n[!] Error: AI Model training failed.");
                }
            }
            1 => {
                let model_path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to saved model checkpoint")
                    .default("passgan_model.pt".to_string())
                    .interact_text()
                    .unwrap();

                if !Path::new(&model_path).exists() {
                    println!("[!] Error: Model checkpoint file '{}' not found. Please train a model first.", model_path);
                    continue;
                }

                let output_file: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to save generated passwords")
                    .default("generated_passwords.txt".to_string())
                    .interact_text()
                    .unwrap();

                let count: u64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Number of passwords to generate")
                    .default(10000000)
                    .interact_text()
                    .unwrap();

                let batch_size: u32 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Generation batch size (larger = faster)")
                    .default(16384)
                    .interact_text()
                    .unwrap();

                let temperature: f64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Sampling temperature (creativity, e.g., 0.7 - 1.2)")
                    .default(0.95)
                    .interact_text()
                    .unwrap();

                let use_gpu = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Use GPU for generation if available?")
                    .default(true)
                    .interact()
                    .unwrap();

                println!("\n[*] Spawning PyTorch Generation Subprocess...");
                let script_path = find_python_script();
                
                let count_str = count.to_string();
                let batch_size_str = batch_size.to_string();
                let temp_str = temperature.to_string();

                let mut args = vec![
                    script_path.to_str().unwrap(),
                    "--mode", "generate",
                    "--model-path", &model_path,
                    "--output-file", &output_file,
                    "--count", &count_str,
                    "--batch-size", &batch_size_str,
                    "--temp", &temp_str,
                ];
                if use_gpu {
                    args.push("--gpu");
                }

                let success = run_python_script(&args)?;

                if success {
                    println!("\n[+] AI Passwords successfully generated and saved to '{}'", output_file);
                } else {
                    println!("\n[!] Error: AI Password generation failed.");
                }
            }
            2 => break,
            _ => unreachable!(),
        }
        println!();
    }
    Ok(())
}

fn markov_menu() -> io::Result<()> {
    loop {
        let selections = &[
            "[a] Train Markov Model on Wordlist",
            "[b] Generate Passwords from Markov Model",
            "[c] Back to Main Menu",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Markov Chain Engine Operations")
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        match selection {
            0 => {
                let input_file: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to training text file (wordlist)")
                    .default("test.txt".to_string())
                    .interact_text()
                    .unwrap();

                if !Path::new(&input_file).exists() {
                    println!("[!] Error: Input file '{}' not found.", input_file);
                    continue;
                }

                let model_path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to save trained Markov model (JSON)")
                    .default("markov_model.json".to_string())
                    .interact_text()
                    .unwrap();

                let order: usize = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Markov model order (length of context history, e.g. 2, 3, or 4)")
                    .default(3)
                    .interact_text()
                    .unwrap();

                println!("[*] Training Rust Markov Engine on '{}'...", input_file);
                let start_time = std::time::Instant::now();
                
                let mut model = MarkovModel::new(order);
                model.train(&input_file)?;
                model.save(&model_path)?;
                
                println!("[+] Markov model trained in {:.2?} and saved to '{}'", start_time.elapsed(), model_path);
            }
            1 => {
                let model_path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to trained Markov model (JSON)")
                    .default("markov_model.json".to_string())
                    .interact_text()
                    .unwrap();

                if !Path::new(&model_path).exists() {
                    println!("[!] Error: Markov model file '{}' not found. Please train a Markov model first.", model_path);
                    continue;
                }

                let output_file: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Path to save generated passwords")
                    .default("markov_passwords.txt".to_string())
                    .interact_text()
                    .unwrap();

                let count: u64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Number of passwords to generate")
                    .default(1000000)
                    .interact_text()
                    .unwrap();

                let len_range: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Generated password length range (e.g. 8-16, or 10-12)")
                    .default("10-12".to_string())
                    .interact_text()
                    .unwrap();

                let (min_len, max_len) = match parse_range(&len_range) {
                    Some(r) => r,
                    None => {
                        println!("[!] Error: Invalid length range format. Use 'min-max' format, e.g., '10-12'.");
                        continue;
                    }
                };

                println!("[*] Loading Markov model from '{}'...", model_path);
                let model = MarkovModel::load(&model_path)?;
                
                println!("[*] Generating {} passwords with lengths {}-{} in Rust...", format_thousands(count), min_len, max_len);
                let start_time = std::time::Instant::now();

                let file = File::create(&output_file)?;
                let mut writer = BufWriter::with_capacity(1024 * 1024, file);
                
                let pb = ProgressBar::new(count);
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
                    .unwrap()
                    .progress_chars("#>-"));

                let mut generated = 0;
                let mut failures_row = 0;

                while generated < count {
                    if let Some(pwd) = model.generate(min_len, max_len) {
                        writeln!(writer, "{}", pwd)?;
                        generated += 1;
                        failures_row = 0;
                        if generated % 10000 == 0 {
                            pb.set_position(generated);
                        }
                    } else {
                        failures_row += 1;
                        if failures_row > 10000 {
                            println!("\n[!] Warning: High generation failure rate. Your training data might not support length range {}-{}.", min_len, max_len);
                            break;
                        }
                    }
                }
                
                writer.flush()?;
                pb.finish_with_message("Done!");
                
                let duration = start_time.elapsed();
                let speed = generated as f64 / duration.as_secs_f64();
                println!("[+] Generation complete in {:.2?}.", duration);
                println!("    Saved {} passwords to '{}' (Speed: {:.2} pwd/sec)", format_thousands(generated), output_file, speed);
            }
            2 => break,
            _ => unreachable!(),
        }
        println!();
    }
    Ok(())
}

fn pattern_menu() -> io::Result<()> {
    let output_file: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to save generated passwords")
        .default("pattern_passwords.txt".to_string())
        .interact_text()
        .unwrap();

    let count: u64 = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Number of passwords to generate")
        .default(1000000)
        .interact_text()
        .unwrap();

    let len_range: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Generated password length range (e.g. 10-12, or 8-15)")
        .default("10-12".to_string())
        .interact_text()
        .unwrap();

    let (min_len, max_len) = match parse_range(&len_range) {
        Some(r) => r,
        None => {
            println!("[!] Error: Invalid length range format. Use 'min-max' format, e.g., '10-12'.");
            return Ok(());
        }
    };

    println!("[*] Generating {} real-world pattern passwords with lengths {}-{} in Rust...", format_thousands(count), min_len, max_len);
    let start_time = std::time::Instant::now();

    let file = File::create(&output_file)?;
    let mut writer = BufWriter::with_capacity(1024 * 1024, file);
    
    let pb = ProgressBar::new(count);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
        .unwrap()
        .progress_chars("#>-"));

    for i in 0..count {
        let pwd = pattern::generate_real_world_password(min_len, max_len);
        writeln!(writer, "{}", pwd)?;
        if i % 10000 == 0 {
            pb.set_position(i);
        }
    }
    
    writer.flush()?;
    pb.finish_with_message("Done!");

    let duration = start_time.elapsed();
    let speed = count as f64 / duration.as_secs_f64();
    println!("[+] Generation complete in {:.2?}.", duration);
    println!("    Saved {} passwords to '{}' (Speed: {:.2} pwd/sec)", format_thousands(count), output_file, speed);

    Ok(())
}

fn parse_range(range_str: &str) -> Option<(usize, usize)> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() == 1 {
        let val = parts[0].trim().parse::<usize>().ok()?;
        Some((val, val))
    } else if parts.len() == 2 {
        let min = parts[0].trim().parse::<usize>().ok()?;
        let max = parts[1].trim().parse::<usize>().ok()?;
        if min <= max {
            Some((min, max))
        } else {
            Some((max, min))
        }
    } else {
        None
    }
}

fn run_python_script(args: &[&str]) -> io::Result<bool> {
    let mut cmd = Command::new("python");
    cmd.args(args);
    
    match cmd.status() {
        Ok(status) => Ok(status.success()),
        Err(_) => {
            let mut cmd3 = Command::new("python3");
            cmd3.args(args);
            match cmd3.status() {
                Ok(status) => Ok(status.success()),
                Err(e) => {
                    println!("\n[!] Error: Could not execute 'python' or 'python3'. Please verify that Python is installed and added to your system PATH.");
                    Err(e)
                }
            }
        }
    }
}

fn format_thousands(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn find_python_script() -> std::path::PathBuf {
    let cwd_path = std::path::PathBuf::from("python").join("neural_engine.py");
    if cwd_path.exists() {
        return cwd_path;
    }
    if let Ok(mut exe_path) = std::env::current_exe() {
        while exe_path.pop() {
            let candidate = exe_path.join("python").join("neural_engine.py");
            if candidate.exists() {
                return candidate;
            }
        }
    }
    std::path::PathBuf::from("python").join("neural_engine.py")
}
