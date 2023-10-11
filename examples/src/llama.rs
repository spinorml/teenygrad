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
use std::path::PathBuf;
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
    pub fn load(model_path: PathBuf, gen: String, size: String, quantize: bool) -> Self {
        Model {}
    }
}

struct Tokenizer {}

impl Tokenizer {
    pub fn load(tokenizer_path: PathBuf) -> Self {
        Tokenizer {}
    }
}

struct LLaMa {
    model: Model,
    tokenizer: Tokenizer,
    gen: String,
    size: String,
    quantize: bool,
}

impl LLaMa {
    pub fn build(
        model_path: PathBuf,
        tokenizer_path: PathBuf,
        gen: String,
        size: String,
        quantize: bool,
    ) -> Self {
        let model = Model::load(model_path, gen, size, quantize);
        let tokenizer = Tokenizer::load(tokenizer_path);

        LLaMA {
            model,
            tokenizer,
            gen,
            size,
            quantize,
        }
    }
}

#[derive(Parser, Debug)]
#[command(author="arshadm@spinorml.com", version, about="Run LLaMA in teenygrad", long_about=None)]
struct Args {
    /// Phrase to start with. Without this, it goes into chatbot mode
    #[arg(long)]
    prompt: Option<String>,

    /// Max number of tokens to generate
    #[arg(long, default_value = "1000")]
    count: u32,

    /// Personality
    #[arg(long, default_value = "William", value_parser = ["William"])]
    personality: String,

    /// Temperature in the softmax
    #[arg(long, default_value = "0.7")]
    temperature: f32,

    /// Print timing per token
    #[arg(long, default_value = "false")]
    timing: bool,

    /// Output profile data to out.prof
    #[arg(long, default_value = "false")]
    profile: bool,

    /// Size of model to use [7B, 13B, 30B, 65B] for Gen 1, [7B, 13B, 70B] for Gen 2, [7B, 13B, 34B] for Code LLaMA
    #[arg(long, default_value = "7B", value_parser = ["7B", "13B", "30B", "34B", "65B", "70B"])]
    size: String,

    /// Generation of the model to use [1, 2, code]
    #[arg(long, default_value = "1", value_parser = ["1", "2", "code"])]
    gen: String,

    /// Quantize the weights to int8 in memory
    #[arg(long, default_value = "false")]
    quantize: bool,

    /// Folder with the original weights to load, or single .index.json,
    /// .safetensors or .bin file
    #[arg(long)]
    model: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    let _chatbot = args.prompt.is_none();
    let _personality: Personality = Personality::from_str(args.personality.as_str()).unwrap();

    let llama_suffix = match args.gen.as_str() {
        "1" => "",
        "2" => "-2",
        "code" => "-code",
        _ => panic!("Unknown generation {}", args.gen),
    };
    let model_path = args
        .model
        .unwrap_or_else(|| PathBuf::from(format!("weights/LLaMA{}/{}", llama_suffix, args.size)));

    let tokenizer_path = if model_path.is_dir() {
        model_path.join("tokenizer.model")
    } else {
        model_path.parent().unwrap().join("tokenizer.model")
    };

    print!("using LLaMA{}-{} model", { llama_suffix }, { args.size });
    let _llama = LLaMa::build(
        model_path,
        tokenizer_path,
        args.gen,
        args.size,
        args.quantize,
    );
}
