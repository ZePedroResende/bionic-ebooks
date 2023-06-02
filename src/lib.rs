use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use xmltree::Element;
use xmltree::XMLNode;
use zip::ZipArchive;
use zip::ZipWriter;

type EpubArchive = ZipArchive<File>;
type EpubWriter = ZipWriter<File>;

pub struct EpubProcessor {
    pub in_path: PathBuf,
    pub in_zip: EpubArchive,
    pub out_zip: EpubWriter,
}

impl EpubProcessor {
    pub fn new(in_path: PathBuf, out_path: PathBuf) -> Result<Self, Box<dyn Error>> {
        File::create(out_path.clone())?;

        Ok(Self {
            in_path: in_path.clone(),
            in_zip: ZipArchive::new(File::open(in_path)?)?,
            out_zip: ZipWriter::new(File::create(out_path)?),
        })
    }

    pub fn process(&mut self) {
        for i in 0..self.in_zip.len() {
            let mut file = self.in_zip.by_index(i).unwrap();
            let mut buf = Vec::new();
            let _ = file.read_to_end(&mut buf);
            let re = Regex::new(r".*html$").unwrap();
            if re.is_match(file.name()) {
                buf = modify_xml(&buf);
            }

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);
            self.out_zip.start_file(file.name(), options).unwrap();

            let _ = self.out_zip.write(&buf);
        }
    }
}

fn modify_xml(buf: &[u8]) -> Vec<u8> {
    let mut names_element = Element::parse(buf).unwrap();

    mutate_text(&mut names_element);

    let mut out_buf: Vec<u8> = vec![];
    names_element.write(&mut out_buf).unwrap();
    out_buf
}

fn mutate_text(element: &mut Element) {
    for node in element.children.iter_mut() {
        match node {
            XMLNode::Element(ref mut elem) => {
                if elem.name != "b" {
                    mutate_text(elem)
                }
            }
            XMLNode::Text(ref mut text) => {
                let bionic: Vec<String> = text.split_whitespace().map(to_bionic).collect();
                let joined = bionic.join(" ");

                let bionic_string = format!("<bionic> {} </bionic>", joined);

                let bionic_element = Element::parse(bionic_string.as_bytes()).unwrap();
                *node = xmltree::XMLNode::Element(bionic_element);
            }
            _ => (),
        }
    }
}

fn to_bionic(word: &str) -> String {
    let trimmed_word = word.trim().replace('&', "&amp;");
    let chars: Vec<char> = trimmed_word.chars().collect();
    let mid_point = chars.len() / 2;

    if mid_point >= chars.len() || chars.is_empty() {
        return trimmed_word;
    }

    if chars.len() == 1 {
        return if chars[0].is_ascii() {
            format!("<b>{} </b>", trimmed_word)
        } else {
            trimmed_word
        };
    }

    let (bold, remaining) = chars.split_at(mid_point);
    let bold_string = String::from_iter(bold);
    let remaining_string = String::from_iter(remaining);

    format!("<b>{}</b>{}", bold_string, remaining_string).replace('&', "&amp;")
}
