/*
 * Copyright (c) 2023 SpinorML
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use clap::{arg, command, Parser};
use glob::glob;
use itertools::Itertools;
use nlputils::sentencepiece::SentencePieceProcessor;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug)]
pub struct Personality {
    pub pre_prompt: &'static str,
    pub examples: Vec<Vec<&'static str>>,
    pub user_delim: &'static str,
    pub resp_delim: &'static str,
    pub end_delim: &'static str,
}

impl FromStr for Personality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "william" => Ok(Personality {
                pre_prompt: "Consider that the following is conversation between an AI assistant named William and the User
You are William!
You are a school boy aged 11 years old. You live in a small village in Southern England.
You are the leader of a small gang of other school children called the Outlaws.
You are confident, resourceful, strong willed, independent minded with original views about the world.
You are a good friend and a good leader. You do not care about your appearance, and hate small talk.
You like playing pirates and red indians. 
<CHAT LOG>
",
                examples: vec![
                    vec!["What is your name?", "Hi! I'm Will, nice to meet you. What's your name?"],
                ],
                user_delim: "\nUser: ",
                resp_delim: "Stacy: ",
                end_delim: " [EOS]\n",
            }),
            _ => Err(format!("Unknown personality {}", s)),
        }
    }
}

struct Model {}

impl Model {
    pub fn load(model_path: &Path) -> Self {
        let checkpoints = glob(&format!(
            "{}/consolidated.*.pth",
            model_path.to_str().unwrap()
        ))
        .unwrap();

        for checkpoint in checkpoints {
            match checkpoint {
                Ok(path) => {
                    let state = torch::model::load(&path).unwrap();
                    println!("State: {:?}", state);
                }
                Err(e) => panic!("Error loading checkpoint: {}", e),
            }
        }

        Model {}
    }
}

struct Tokenizer {
    sp: SentencePieceProcessor,
}

impl Tokenizer {
    pub fn load(model_path: &Path) -> Self {
        let tokenizer_model = model_path.join("../tokenizer.model");
        let sp = SentencePieceProcessor::open(tokenizer_model).unwrap();

        Tokenizer { sp }
    }

    pub fn bos_id(&self) -> u32 {
        self.sp.bos_id().unwrap()
    }

    pub fn encode(&self, text: &str) -> Vec<u32> {
        let pieces = self.sp.encode(text).unwrap();
        pieces.iter().map(|p| p.id).collect_vec()
    }

    pub fn decode(&self, pieces: &[u32]) -> String {
        self.sp.decode_piece_ids(pieces).unwrap()
    }
}

struct Llama {
    model: Model,
    tokenizer: Tokenizer,
}

impl Llama {
    pub fn build(model_path: &Path) -> Self {
        let model = Model::load(model_path);
        let tokenizer = Tokenizer::load(model_path);

        print!("BOS ID: {}", tokenizer.bos_id());
        let x = tokenizer.encode("What is your name?");
        println!("Encoded: {:?}", x);
        println!("Decoded: {}", tokenizer.decode(&x));

        Llama { model, tokenizer }
    }
}

#[derive(Parser, Debug)]
#[command(author="arshadm@spinorml.com", version, about="Run LLaMA in teenygrad", long_about=None)]
struct Args {
    /// Personality
    #[arg(long, default_value = "William", value_parser = ["William"])]
    personality: String,

    /// Max number of tokens to generate
    #[arg(long, default_value = "1000")]
    count: u32,

    /// Phrase to start with. Without this, it goes into chatbot mode
    #[arg(long)]
    prompt: Option<String>,

    /// Temperature in the softmax
    #[arg(long, default_value = "0.7")]
    temperature: f32,

    /// Folder with the original weights to load, or single .index.json,
    /// .safetensors or .bin file
    #[arg(long)]
    model: PathBuf,
}

//
// Example 1:
// cargo run --bin llama -- --prompt "What is your name?" --count 10 --model /raid5/data/llama/llama-2-7b-chat
// Output:
//
//
fn main() {
    let args = Args::parse();
    let _chatbot = args.prompt.is_none();
    let _personality: Personality = Personality::from_str(args.personality.as_str()).unwrap();

    assert!(args.model.is_dir());

    let _llama = Llama::build(&args.model);
}
