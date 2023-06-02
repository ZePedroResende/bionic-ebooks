use bionic_ebooks::EpubProcessor;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let in_file_path = &args[1];
    let out_file_path = &args[2];

    let mut zip = EpubProcessor::new(in_file_path.into(), out_file_path.into())?;

    zip.process();

    Ok(())
}
