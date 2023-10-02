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

use std::{path::PathBuf, str::FromStr};

use clap::{arg, command, Parser};

mod personality;

use personality::Personality;

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
    model: PathBuf,
}

fn main() {
    let args = Args::parse();
    let chatbot = args.prompt.is_none();
    let personality: Personality = Personality::from_str(args.personality.as_str()).unwrap();

    println!("Args: {:?}, {:?}, {:?}", args, chatbot, personality)
}
