# PassGan 🚀

PassGan is a **hybrid, high-performance password generation suite** that bridges state-of-the-art **Deep Learning (PyTorch)** with ultra-fast **Systems Programming (Rust)**. 

By combining GPU-accelerated recurrent neural networks for sequence modeling with native Rust engines for statistical Markov chains, rule-based pattern generators, and high-speed I/O filtering, DeepPassGen offers a robust and comprehensive toolkit for security audits, password cracking research, and security training.

---

## 🏗️ System Architecture

PassGan uses a dual-engine architecture designed to maximize hardware utilization:

```
                  ┌──────────────────────────────┐
                  │      Rust main CLI Entry     │
                  └──────────────┬───────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         ▼                       ▼                       ▼
┌──────────────────┐   ┌───────────────────┐   ┌──────────────────┐
│   AI Engine      │   │   Markov Engine   │   │  Pattern Engine  │
│ (PyTorch Subproc)│   │   (Native Rust)   │   │  (Native Rust)   │
└────────┬─────────┘   └─────────┬─────────┘   └────────┬─────────┘
         │                       │                      │
         ▼                       ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│              High-Speed BufWriter Output Streams                │
└────────────────────────────────┬────────────────────────────────┘
                                 ▼
                    ┌──────────────────────────┐
                    │     Output Wordlist      │
                    └──────────────────────────┘
```

---

## 🧠 Core Components & How They Work

### 1. AI PyTorch GRU Engine (`python/neural_engine.py`)
This component uses a **Character-level Recurrent Neural Network (Char-RNN)** powered by a Gated Recurrent Unit (GRU) to learn the structural "language" of passwords from training datasets (like the RockYou leak).

#### The Model Architecture:
- **Embedding Layer**: Converts categorical character indices into a $D$-dimensional continuous vector space. This allows the network to learn similarities between characters (e.g., grouping lowercase letters together, digits together).
- **GRU Layer**: A recurrent network with gate mechanisms (Reset Gate and Update Gate) that maintains a hidden state vector $h_t$ over time. It dynamically learns context dependencies (e.g., that `1` is likely to be followed by `2`, or `p` by `a`).
- **Linear Decoder**: Projects the final hidden states back to the size of the vocabulary to output raw logits (transition scores) for the next character.

#### Checkpoint Serialization (Save/Load):
A major limitation of basic PassGAN scripts is that they require retraining the model every single time you want to generate passwords. DeepPassGen solves this by saving a complete **Checkpoint Package** containing:
1. `model_state_dict`: The trained weights.
2. `vocab`, `char2idx`, `idx2char`: The exact character-to-index mapping learned during training. This is critical because if the vocabulary mapping changes, the loaded model will output garbage text.
3. `hyperparameters`: Hidden layers, embedding size, and sequence length config.

#### Temperature-Based Sampling:
During generation, the next character is chosen probabilistically using a Softmax function modified by a **temperature parameter** ($T$):
$$P(x_i) = \frac{e^{z_i / T}}{\sum_j e^{z_j / T}}$$
- **Low Temperature (e.g., $T = 0.7$)**: Makes the model conservative, sticking closely to common password structures found in the training data (highly realistic, lower search entropy).
- **High Temperature (e.g., $T = 1.2$)**: Increases randomness, leading to highly creative and novel password candidates (higher search entropy).

---

### 2. Rust-Native Markov Chain Generator (`src/markov.rs`)
Markov Chain generation is statistical, deterministic, and extremely fast. It models passwords as a sequence of state transitions where the probability of generating a character depends only on the previous $N$ characters (where $N$ is the **Markov Order**).

#### How Training Works:
1. The engine reads the training file line-by-line using buffered streams.
2. It breaks passwords into character $n$-grams of length $N$ and counts how often a character $c$ follows each $n$-gram.
3. It compiles these frequencies into a sparse transition matrix stored in memory:
  ```rust
Transition[prefix] -> Vec<(next_char, count)>
```
4. The trained model is saved as a lightweight JSON file (`markov_model.json`).

#### How Generation Works:
1. It selects a starting sequence (n-gram) based on the frequency distribution of initial prefixes.
2. At each step, it performs a **weighted random walk** over the transition list corresponding to the current state.
3. If it generates the End-of-Password token (`\x03`), or reaches the maximum length, the sequence terminates.
4. The transition state shifts left by one character, appends the newly generated character, and repeats.
5. Runs entirely in compiled Rust, allowing generation speeds exceeding **100+ million passwords per second** depending on your CPU.

---

### 3. Rust-Native Real-World Pattern Generator (`src/pattern.rs`)
This generator is rule-based and simulates how humans construct passwords to bypass complexity rules. It requires no training data and runs natively in Rust.

#### Phonetic Construction (Pronounceability):
Instead of choosing random, unrememberable strings of characters, the engine constructs pronounceable syllables by alternating consonants and vowels (e.g., `Bramado`, `Patereso`). This mimics human lexicographical habits.

#### Structure Templates:
The generator assigns each password a template based on real-world leak distributions:
1. **Word + Digits** (e.g., `Bramado2024`)
2. **Word + Symbol** (e.g., `Patereso!`)
3. **Word + Digits + Symbol** (e.g., `Shilame123#`)
4. **Word + Symbol + Digits** (e.g., `Bramado!99`)
5. **Capitalized Separation / CamelCase** (e.g., `BlueSky`, `HappyDog`)

#### Casing & Leetspeak Substitutions:
- **Capitalization**: Applies capital letters to word starts with an 80% probability.
- **Leet Substitutions**: Applies substitution mappings (e.g., `a` $\rightarrow$ `@`, `o` $\rightarrow$ `0`, `e` $\rightarrow$ `3`, `s` $\rightarrow$ `$`) with a 15% probability.
- **Truncation/Padding**: Truncates or pads (using symbols/digits) the resulting password to fit user-specified ranges like `10-12` characters exactly.

---

## 🛠️ Installation & Setup

### Prerequisites
- **Rust Compiler** (Cargo): [Install Rust](https://www.rust-lang.org/tools/install)
- **Python 3.8+** with **PyTorch**: [Install PyTorch](https://pytorch.org/get-started/locally/)

### 1. Build the Rust Binary
Clone the repository and build the release executable:
```bash
git clone https://github.com/yourusername/passgan.git
cd passgan
cargo build --release
```
The compiled release executable will be located at `target/release/passgan.exe` (on Windows) or `target/release/passgan` (on Linux/macOS).

### 2. Install Python Dependencies
Install PyTorch and its dependencies:
```bash
pip install -r requirements.txt
```

---

## 🚀 Execution Modes

PassGan supports two interface modes depending on how you run it:

### 1. Interactive Terminal UI (TUI) Mode
If run in a terminal (TTY) with no arguments, a colorful menu guides you through configuring files and parameters interactively:
```bash
./target/release/passgan
```

### 2. Scriptable CLI Mode
If run with arguments (or inside shell scripts/pipelines), it uses command-line flags. This is optimal for automation and piping into hashcat.

```text
DeepPassGen - Command Line Interface Usage:
  passgan.exe [OPTIONS]

Options:
  --mode <MODE>       Mode selection:
                        ai-train     : Train the PyTorch GRU model
                        ai-gen       : Generate passwords using PyTorch GRU
                        markov-train : Train the native Rust Markov model
                        markov-gen   : Generate passwords using Markov model
                        pattern-gen  : Generate real-world random passwords
  --input <PATH>      Path to input wordlist file (required for train modes)
  --output <PATH>     Path to save generated passwords
  --model <PATH>      Path to model file (.pt for AI, .json for Markov)
  --count <NUM>       Number of passwords to generate (default: 1,000,000)
  --length <RANGE>    Generated password length range (default: 10-12)
  --order <NUM>       Markov chain history order (default: 3)
  --temp <FLOAT>      AI generation temperature/creativity (default: 0.95)
  --gpu               Enable GPU (CUDA) acceleration for AI generation
  -h, --help          Show this help menu
```

---

## 📝 Usage Examples

### 1. AI PyTorch Generation (PassGAN GRU)
**Train the model:**
```bash
./target/release/passgan --mode ai-train --input rockyou.txt --model passgan_model.pt
```

**Generate 10 million passwords using GPU acceleration:**
```bash
./target/release/passgan --mode ai-gen --model passgan_model.pt --count 10000000 --output generated_ai.txt --gpu
```

### 2. Native Rust Markov Chain
**Train the Markov model (Order 3):**
```bash
./target/release/passgan --mode markov-train --input rockyou.txt --model markov_order3.json --order 3
```

**Generate 5 million passwords of length 8-16:**
```bash
./target/release/passgan --mode markov-gen --model markov_order3.json --count 5000000 --length 8-16 --output markov_words.txt
```

### 3. Real-World Pattern Generator (Rule-based)
**Generate 1 million human-like passwords of length 10-12:**
```bash
./target/release/passgan --mode pattern-gen --count 1000000 --length 10-12 --output human_passwords.txt
```

---

## 📈 Performance & Tuning

- **CUDA/GPU**: Always pass the `--gpu` flag during AI generation if you have an NVIDIA card. PyTorch handles parallel inference efficiently.
- **Batch Size Tuning**: If you run out of GPU memory (OOM) during generation, decrease the generation batch size inside the TUI or python script (default: `16,384`).
- **Markov Orders**: 
  - **Order 2**: High variability, generates many new and unique words.
  - **Order 3 (Recommended)**: Balanced; produces realistic syllables and words.
  - **Order 4**: Tight fit; matches the input dictionaries closely.
