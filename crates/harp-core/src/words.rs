use flate2::read::GzDecoder;
use std::io::Read;
use std::sync::LazyLock;

const ADJECTIVES_GZ: &[u8] = include_bytes!("../data/adjectives.txt.gz");
const NOUNS_GZ: &[u8] = include_bytes!("../data/nouns.txt.gz");

fn decompress(data: &[u8]) -> String {
    let mut decoder = GzDecoder::new(data);
    let mut s = String::new();
    decoder.read_to_string(&mut s).expect("failed to decompress");
    s
}

pub static ADJECTIVES: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let decompressed = Box::leak(decompress(ADJECTIVES_GZ).into_boxed_str());
    decompressed.lines().filter(|s| !s.is_empty()).collect()
});

pub static NOUNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let decompressed = Box::leak(decompress(NOUNS_GZ).into_boxed_str());
    decompressed.lines().filter(|s| !s.is_empty()).collect()
});
