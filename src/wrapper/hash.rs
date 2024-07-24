#[allow(dead_code)]
pub fn hash(input: &[u8]) -> String {
    blake3::hash(input).to_string()
}

pub struct Hasher {
    hasher: blake3::Hasher
}

impl Hasher {
    pub fn new() -> Hasher {
        Hasher {
            hasher: blake3::Hasher::new()
        }
    }

    pub fn update(&mut self, input: &[u8]) {
        self.hasher.update(input);
    }

    pub fn finalize(self) -> String {
        self.hasher.finalize().to_string()
    }
}