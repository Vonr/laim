use base64::{prelude::BASE64_STANDARD_NO_PAD, Engine};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub position: u32,
    pub score: u32,
    pub millis: u128,
    pub rows: u32,
    pub columns: u32,
    pub active: u32,
}

impl Record {
    #[inline]
    pub const fn new(
        position: u32,
        score: u32,
        millis: u128,
        rows: u32,
        columns: u32,
        active: u32,
    ) -> Self {
        Self {
            position,
            score,
            millis,
            rows,
            columns,
            active,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 36 {
            return None;
        }

        Some(Self {
            position: u32::from_le_bytes(bytes[..4].try_into().unwrap()),
            score: u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            millis: u128::from_le_bytes(bytes[8..24].try_into().unwrap()),
            rows: u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            columns: u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            active: u32::from_le_bytes(bytes[32..36].try_into().unwrap()),
        })
    }

    pub fn from_str(str: &str) -> Option<Self> {
        BASE64_STANDARD_NO_PAD
            .decode(str)
            .ok()
            .as_deref()
            .map(Self::from_bytes)?
    }

    pub fn to_string(&self) -> String {
        let mut bytes = Vec::with_capacity(std::mem::size_of::<Self>());
        bytes.extend_from_slice(&self.position.to_le_bytes());
        bytes.extend_from_slice(&self.score.to_le_bytes());
        bytes.extend_from_slice(&self.millis.to_le_bytes());
        bytes.extend_from_slice(&self.rows.to_le_bytes());
        bytes.extend_from_slice(&self.columns.to_le_bytes());
        bytes.extend_from_slice(&self.active.to_le_bytes());

        BASE64_STANDARD_NO_PAD.encode(bytes)
    }
}
