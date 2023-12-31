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

use std::{fs::File, path::Path};

use serde_pickle_rs::{DeOptions, Value};

pub fn load(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut zip = zip::ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let file = zip.by_index(i)?;
        println!("File: {}", file.name());
        if file.name().ends_with(".pkl") {
            println!("Found pkl file");
            let result: Value = serde_pickle_rs::from_reader(
                file,
                DeOptions::default().replace_unresolved_globals(),
            )?;
            println!("Value {:?}", result)
        }
    }
    todo!("Implement load")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let path = Path::new("/raid5/data/llama/llama-2-7b-chat/consolidated.00.pth");
        let _ = load(path).unwrap();
    }
}
