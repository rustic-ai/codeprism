//! Serialization utilities for storage

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerializationFormat {
    Bincode,
    Json,
    MessagePack,
}

/// Compression algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Zstd,
}

/// Serializer with optional compression
pub struct Serializer {
    format: SerializationFormat,
    compression: CompressionAlgorithm,
}

impl Serializer {
    /// Create a new serializer
    pub fn new(format: SerializationFormat, compression: CompressionAlgorithm) -> Self {
        Self {
            format,
            compression,
        }
    }

    /// Serialize and optionally compress data
    pub fn serialize<T>(&self, value: &T) -> Result<Vec<u8>>
    where
        T: Serialize,
    {
        // First serialize
        let serialized = match self.format {
            SerializationFormat::Bincode => bincode::serialize(value)?,
            SerializationFormat::Json => serde_json::to_vec(value)?,
            SerializationFormat::MessagePack => rmp_serde::to_vec(value)?,
        };

        // Then optionally compress
        let compressed = match self.compression {
            CompressionAlgorithm::None => serialized,
            CompressionAlgorithm::Gzip => {
                use std::io::Write;
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(&serialized)?;
                encoder.finish()?
            }
            CompressionAlgorithm::Zstd => zstd::bulk::compress(&serialized, 3)?,
        };

        Ok(compressed)
    }

    /// Decompress and deserialize data
    pub fn deserialize<T>(&self, data: &[u8]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        // First decompress if needed
        let decompressed = match self.compression {
            CompressionAlgorithm::None => data.to_vec(),
            CompressionAlgorithm::Gzip => {
                use std::io::Read;
                let mut decoder = flate2::read::GzDecoder::new(data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                decompressed
            }
            CompressionAlgorithm::Zstd => {
                zstd::bulk::decompress(data, 10 * 1024 * 1024)? // 10MB max decompressed size
            }
        };

        // Then deserialize
        let value = match self.format {
            SerializationFormat::Bincode => bincode::deserialize(&decompressed)?,
            SerializationFormat::Json => serde_json::from_slice(&decompressed)?,
            SerializationFormat::MessagePack => rmp_serde::from_slice(&decompressed)?,
        };

        Ok(value)
    }
}

impl Default for Serializer {
    /// Create a default serializer (bincode with gzip compression)
    fn default() -> Self {
        Self {
            format: SerializationFormat::Bincode,
            compression: CompressionAlgorithm::Gzip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData {
        name: String,
        value: i32,
        items: Vec<String>,
    }

    #[test]
    fn test_serialization_bincode() {
        let serializer = Serializer::new(SerializationFormat::Bincode, CompressionAlgorithm::None);
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_serialization_json() {
        let serializer = Serializer::new(SerializationFormat::Json, CompressionAlgorithm::None);
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_compression_gzip() {
        let serializer = Serializer::new(SerializationFormat::Json, CompressionAlgorithm::Gzip);
        let data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["a".to_string(), "b".to_string()],
        };

        let serialized = serializer.serialize(&data).unwrap();
        let deserialized: TestData = serializer.deserialize(&serialized).unwrap();

        assert_eq!(data, deserialized);
    }
}
