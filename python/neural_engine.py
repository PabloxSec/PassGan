import torch
import torch.nn as nn
import torch.optim as optim
from torch.utils.data import Dataset, DataLoader
import time
import os
import argparse
import sys

print("[*] Initializing PyTorch Neural Generation Engine...")

# Special character tokens
SOP = '\x02' # Start of Password
EOP = '\x03' # End of Password
PAD = '\x00' # Padding

vocab = []
char2idx = {}
idx2char = {}

def build_vocab(filepath):
    chars = set([SOP, EOP, PAD])
    with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            pwd = line.strip()
            if pwd:
                chars.update(list(pwd))
    
    global vocab, char2idx, idx2char
    vocab = sorted(list(chars))
    char2idx = {c: i for i, c in enumerate(vocab)}
    idx2char = {i: c for i, c in enumerate(vocab)}
    print(f"[*] Extracted Vocabulary: {len(vocab)} unique characters")
    return len(vocab)

class PasswordDataset(Dataset):
    def __init__(self, filepath, seq_length):
        self.seq_length = seq_length
        self.data = []
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            for line in f:
                pwd = line.strip()
                if not pwd: continue
                chars = [SOP] + list(pwd) + [EOP]
                encoded = [char2idx.get(c, char2idx[PAD]) for c in chars]
                self.data.append(encoded)
                
    def __len__(self):
        return len(self.data)
        
    def __getitem__(self, idx):
        item = self.data[idx]
        if len(item) < self.seq_length + 1:
            item = item + [char2idx[PAD]] * (self.seq_length + 1 - len(item))
        else:
            item = item[:self.seq_length + 1]
            
        x = torch.tensor(item[:-1], dtype=torch.long)
        y = torch.tensor(item[1:], dtype=torch.long)
        return x, y

class CharRNN(nn.Module):
    def __init__(self, vocab_size, embed_size, hidden_size, num_layers):
        super(CharRNN, self).__init__()
        self.embed = nn.Embedding(vocab_size, embed_size)
        self.gru = nn.GRU(embed_size, hidden_size, num_layers, batch_first=True)
        self.fc = nn.Linear(hidden_size, vocab_size)
        
    def forward(self, x, hidden):
        out = self.embed(x)
        out, hidden = self.gru(out, hidden)
        out = out.reshape(-1, out.size(2))
        out = self.fc(out)
        return out, hidden

def train_model(model, dataloader, epochs, lr, model_path, hp_dict):
    criterion = nn.CrossEntropyLoss(ignore_index=char2idx[PAD])
    optimizer = optim.Adam(model.parameters(), lr=lr)
    
    print(f"[*] Training Deep Learning Model (GRU) on device: {device}...")
    model.train()
    
    try:
        for epoch in range(epochs):
            start_time = time.time()
            total_loss = 0
            for i, (x, y) in enumerate(dataloader):
                x, y = x.to(device), y.to(device)
                
                outputs, _ = model(x, None)
                loss = criterion(outputs, y.view(-1))
                
                optimizer.zero_grad()
                loss.backward()
                optimizer.step()
                
                total_loss += loss.item()
                
            avg_loss = total_loss / len(dataloader)
            print(f"    Epoch [{epoch+1}/{epochs}], Loss: {avg_loss:.4f}, Time: {time.time()-start_time:.2f}s", flush=True)
    except KeyboardInterrupt:
        print("\n[!] Training interrupted by user. Saving progress...")

    # Save checkpoint
    checkpoint = {
        'model_state_dict': model.state_dict(),
        'vocab': vocab,
        'char2idx': char2idx,
        'idx2char': idx2char,
        'hyperparameters': hp_dict
    }
    torch.save(checkpoint, model_path)
    print(f"[+] Model checkpoint saved to: {model_path}")

@torch.inference_mode()
def generate_passwords(model, num_generate, batch_size, temp, output_file):
    model.eval()
    print(f"[*] Active generating. Target: {num_generate:,} passwords on device: {device}...")
    
    generated_set = set()
    start_time = time.time()
    
    max_len = 24
    sop_idx = char2idx[SOP]
    eop_idx = char2idx[EOP]
    pad_idx = char2idx[PAD]
    
    idx2char_arr = [idx2char[i] for i in range(len(idx2char))]
    
    try:
        with open(output_file, 'w', encoding='utf-8', buffering=1024*1024) as f:
            while len(generated_set) < num_generate:
                batch_passwords_tensor = torch.empty((batch_size, max_len), dtype=torch.long, device=device)
                
                x = torch.full((batch_size, 1), sop_idx, dtype=torch.long, device=device)
                hidden = None
                
                for step in range(max_len):
                    outputs, hidden = model(x, hidden)
                    probs = torch.softmax(outputs / temp, dim=1)
                    
                    sampled_indices = torch.multinomial(probs, 1)
                    batch_passwords_tensor[:, step] = sampled_indices.squeeze(1)
                    x = sampled_indices
                    
                # Decode batch on CPU
                batch_lists = batch_passwords_tensor.tolist()
                new_passwords = []
                
                for pwd_indices in batch_lists:
                    pwd_chars = []
                    for idx in pwd_indices:
                        if idx == eop_idx or idx == pad_idx:
                            break
                        if idx != sop_idx:
                            pwd_chars.append(idx2char_arr[idx])
                            
                    if len(pwd_chars) >= 4:
                        pwd_str = "".join(pwd_chars)
                        if pwd_str not in generated_set:
                            generated_set.add(pwd_str)
                            new_passwords.append(pwd_str)
                            
                if new_passwords:
                    for pwd_str in new_passwords:
                        f.write(pwd_str + "\n")
                    
                    if len(generated_set) % 250000 < len(new_passwords):
                        elapsed = time.time() - start_time
                        speed = len(generated_set) / elapsed if elapsed > 0 else 0
                        print(f"    Synthesized {len(generated_set):,} / {num_generate:,} (Elapsed: {elapsed:.1f}s, Speed: {speed:.1f} pwd/s)", flush=True)
                        
                if len(generated_set) >= num_generate:
                    break
    except KeyboardInterrupt:
        print("\n[!] Generation interrupted by user. Flushing file and exiting...")
        
    elapsed = time.time() - start_time
    print(f"[+] Completed. Saved {len(generated_set):,} passwords to '{output_file}' in {elapsed:.2f}s")

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="PyTorch Password Generator Backend")
    parser.add_argument("--mode", required=True, choices=["train", "generate"], help="Operation mode")
    parser.add_argument("--input-file", help="Path to input wordlist (required for train)")
    parser.add_argument("--model-path", default="passgan_model.pt", help="Path to model checkpoint")
    parser.add_argument("--output-file", default="generated_passwords.txt", help="Path to output generated passwords")
    parser.add_argument("--epochs", type=int, default=3)
    parser.add_argument("--batch-size", type=int, default=1024)
    parser.add_argument("--hidden-size", type=int, default=128)
    parser.add_argument("--embed-size", type=int, default=32)
    parser.add_argument("--layers", type=int, default=1)
    parser.add_argument("--lr", type=float, default=0.005)
    parser.add_argument("--seq-len", type=int, default=16)
    parser.add_argument("--count", type=int, default=10000000)
    parser.add_argument("--temp", type=float, default=0.95)
    parser.add_argument("--gpu", action="store_true", help="Attempt GPU/CUDA execution")
    
    args = parser.parse_args()

    # Device configuration
    if args.gpu and torch.cuda.is_available():
        device = torch.device("cuda")
        torch.backends.cudnn.benchmark = True
    else:
        device = torch.device("cpu")
        torch.set_num_threads(os.cpu_count() or 4)

    if args.mode == "train":
        if not args.input_file:
            print("[!] Error: --input-file is required for train mode.")
            sys.exit(1)
            
        vocab_size = build_vocab(args.input_file)
        dataset = PasswordDataset(args.input_file, args.seq_len)
        dataloader = DataLoader(dataset, batch_size=args.batch_size, shuffle=True)
        
        hp = {
            'embed_size': args.embed_size,
            'hidden_size': args.hidden_size,
            'num_layers': args.layers,
            'seq_length': args.seq_len
        }
        
        model = CharRNN(vocab_size, args.embed_size, args.hidden_size, args.layers).to(device)
        train_model(model, dataloader, args.epochs, args.lr, args.model_path, hp)

    elif args.mode == "generate":
        if not os.path.exists(args.model_path):
            print(f"[!] Error: Model checkpoint file '{args.model_path}' not found.")
            sys.exit(1)
            
        # Load checkpoint
        print(f"[*] Loading model checkpoint from '{args.model_path}'...")
        checkpoint = torch.load(args.model_path, map_location=device)
        
        # Restore mappings
        vocab = checkpoint['vocab']
        char2idx = checkpoint['char2idx']
        idx2char = checkpoint['idx2char']
        hp = checkpoint['hyperparameters']
        
        vocab_size = len(vocab)
        
        # Initialize and load weights
        model = CharRNN(vocab_size, hp['embed_size'], hp['hidden_size'], hp['num_layers']).to(device)
        model.load_state_dict(checkpoint['model_state_dict'])
        
        generate_passwords(model, args.count, args.batch_size, args.temp, args.output_file)
