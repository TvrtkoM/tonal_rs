#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TonalParseError {
    pub entity: String,
    pub input: String,
}

impl std::fmt::Display for TonalParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Invalid {}: {}", self.entity, self.input)
    }
}

impl std::error::Error for TonalParseError {}
