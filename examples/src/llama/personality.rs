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
