/*
 *  Copyright (c) 2021-2024 Works Applications Co., Ltd.
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *   Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 */

use sudachi::analysis::stateless_tokenizer::DictionaryAccess;
use sudachi::prelude::{MorphemeList, SudachiResult};

#[derive(serde::Serialize)]
struct MorphemeJson<'a> {
    surface: &'a str,
    poses: Vec<&'a str>,
    normalized_form: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    dictionary_form: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading_form: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_oov: Option<bool>,
}

pub trait SudachiOutput<T> {
    fn write(&self, writer: &mut Vec<u8>, morphemes: &MorphemeList<T>) -> SudachiResult<()>;
}

pub struct WakachiJSON {
    exclude_pos: Vec<String>,
}

impl WakachiJSON {
    pub fn new(exclude_pos: Vec<String>) -> WakachiJSON {
        WakachiJSON { exclude_pos }
    }
}

impl<T: DictionaryAccess> SudachiOutput<T> for WakachiJSON {
    fn write(&self, writer: &mut Vec<u8>, morphemes: &MorphemeList<T>) -> SudachiResult<()> {
        if morphemes.is_empty() {
            return Ok(());
        }
        for m in morphemes.iter() {
            if !self.exclude_pos.contains(&m.part_of_speech()[0]) {
                serde_json::to_writer(&mut *writer, &*m.surface()).unwrap();
                writer.push(b',');
            }
        }
        if writer.last() == Some(&b',') { writer.pop(); }
        Ok(())
    }
}

pub struct WakachiRaw {
    exclude_pos: Vec<String>,
}

impl WakachiRaw {
    pub fn new(exclude_pos: Vec<String>) -> WakachiRaw {
        WakachiRaw { exclude_pos }
    }
}

impl<T: DictionaryAccess> SudachiOutput<T> for WakachiRaw {
    fn write(&self, writer: &mut Vec<u8>, morphemes: &MorphemeList<T>) -> SudachiResult<()> {
        if morphemes.is_empty() {
            writer.push(b'\n');
            return Ok(());
        }
        for m in morphemes.iter() {
            if !self.exclude_pos.contains(&m.part_of_speech()[0]) {
                writer.extend_from_slice(m.surface().as_bytes());
                writer.push(b' ');
            }
        }
        if writer.last() == Some(&b' ') { writer.pop(); }
        Ok(())
    }
}

pub struct SimpleJSON {
    print_all: bool,
    exclude_pos: Vec<String>,
}

impl SimpleJSON {
    pub fn new(print_all: bool, exclude_pos: Vec<String>) -> SimpleJSON {
        SimpleJSON { print_all, exclude_pos }
    }
}

impl<T: DictionaryAccess> SudachiOutput<T> for SimpleJSON {
    fn write(&self, writer: &mut Vec<u8>, morphemes: &MorphemeList<T>) -> SudachiResult<()> {
        for m in morphemes.iter() {
            if !self.exclude_pos.contains(&m.part_of_speech()[0]) {
                let entry = MorphemeJson {
                    surface: &m.surface(),
                    poses: m.part_of_speech().iter().map(|s| s.as_str()).collect(),
                    normalized_form: m.normalized_form(),
                    dictionary_form: if self.print_all { Some(m.dictionary_form()) } else { None },
                    reading_form: if self.print_all { Some(m.reading_form()) } else { None },
                    is_oov: if self.print_all && m.is_oov() { Some(true) } else { None },
                };
                serde_json::to_writer(&mut *writer, &entry).unwrap();
                writer.push(b',');
            }
        }
        if writer.last() == Some(&b',') { writer.pop(); }
        Ok(())
    }
}
pub struct SimpleRaw {
    print_all: bool,
    exclude_pos: Vec<String>,
}

impl SimpleRaw {
    pub fn new(print_all: bool, exclude_pos: Vec<String>) -> SimpleRaw {
        SimpleRaw { print_all, exclude_pos }
    }
}

impl<T: DictionaryAccess> SudachiOutput<T> for SimpleRaw {
    fn write(&self, writer: &mut Vec<u8>, morphemes: &MorphemeList<T>) -> SudachiResult<()> {
        for m in morphemes.iter() {
            if !self.exclude_pos.contains(&m.part_of_speech()[0]) {
                writer.extend_from_slice(m.surface().as_bytes());
                writer.push(b'\t');
                let all_pos = m.part_of_speech();
                for (idx, pos) in all_pos.iter().enumerate() {
                    writer.extend_from_slice(pos.as_bytes());
                    if idx + 1 != all_pos.len() {
                        writer.push(b',');
                    }
                }
                writer.push(b'\t');
                writer.extend_from_slice(m.normalized_form().as_bytes());

                if self.print_all {
                    writer.push(b'\t');
                    writer.extend_from_slice(m.dictionary_form().as_bytes());
                    writer.push(b'\t');
                    writer.extend_from_slice(m.reading_form().as_bytes());
                    writer.push(b'\t');
                    writer.extend_from_slice(format!("{}", m.dictionary_id()).as_bytes());
                    writer.push(b'\t');
                    writer.extend_from_slice(format!("{:?}", m.synonym_group_ids()).as_bytes());

                    if m.is_oov() {
                        writer.extend_from_slice(b"\t(OOV)");
                    }
                }
                writer.push(b'\n');
            }
        }
        Ok(())
    }
}