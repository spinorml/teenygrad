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

use std::path::PathBuf;

use clap::{arg, command, value_parser, Command};

fn cli() -> Command {
    command!()
        .about("Run the LLaMA model")
        .arg(arg!(--prompt <PROMPT> "Phrase to start with. Without this, it goes into chatbot mode"))
        .arg(arg!(--count <COUNT> "Max number of tokens to generate").default_value("1000"))
        .arg(arg!(--temperature <TEMPERATURE> "Temperature in the softmax").default_value("0.7"))
        .arg(arg!(--timing <TIMING> "Print timing per token").default_value("true"))
        .arg(arg!(--profile "Output profile data to out.prof").default_value("true"))
        .arg(arg!(--size <SIZE> "Size of model for Gen 1 - [7B, 13B, 70B] and for Gen 2 - [7B, 13B, 34B] for Code LLaMA")
            .default_value("7B")
            .value_parser(["7B", "13B", "30B", "34B", "65B", "70B"])
        )
        .arg(arg!(--gen <GEN> "Generation of the model to use [1, 2, code]")
            .default_value("1")
            .value_parser(["1", "2", "code"])
        )
        .arg(arg!(--quantize "Quantize the weights to int8 in memory"))
        .arg(arg!(--model <MODEL> "Folder with the original weights to load, or single .index.json, .safetensors or .bin file")
            .value_parser(value_parser!(PathBuf)))
}

fn main() {
    let _ = cli().get_matches();
}
