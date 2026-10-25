//! SHA-256 digests over raw bytes.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest as _, Sha256};

/// A SHA-256 digest, serialized as 64 lowercase hex characters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    /// Hashes `bytes` exactly as given. No line-ending, whitespace or Unicode
    /// normalization: any byte difference must change the digest.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(Sha256::digest(bytes).into())
    }

    /// Parses 64 lowercase hex characters. Uppercase is rejected so that a
    /// digest has a single textual form.
    pub fn parse_hex(text: &str) -> Option<Self> {
        let raw = text.as_bytes();
        if raw.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        let (pairs, _) = raw.as_chunks::<2>();
        for (byte, [high, low]) in out.iter_mut().zip(pairs) {
            *byte = (nibble(*high)? << 4) | nibble(*low)?;
        }
        Some(Self(out))
    }
}

fn nibble(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        _ => None,
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sha256Digest({self})")
    }
}

impl Serialize for Sha256Digest {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Sha256Digest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse_hex(&text)
            .ok_or_else(|| serde::de::Error::custom("expected 64 lowercase hex characters"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(
            Sha256Digest::of(b"").to_string(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            Sha256Digest::of(b"abc").to_string(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn hex_round_trip() {
        let digest = Sha256Digest::of(b"syngard");
        assert_eq!(Sha256Digest::parse_hex(&digest.to_string()), Some(digest));
    }

    #[test]
    fn rejects_non_canonical_hex() {
        let upper = Sha256Digest::of(b"abc").to_string().to_uppercase();
        assert_eq!(Sha256Digest::parse_hex(&upper), None);
        assert_eq!(Sha256Digest::parse_hex("abc"), None);
        assert_eq!(Sha256Digest::parse_hex(&"g".repeat(64)), None);
    }

    #[test]
    fn line_endings_are_not_normalized() {
        assert_ne!(Sha256Digest::of(b"a\n"), Sha256Digest::of(b"a\r\n"));
    }
}
