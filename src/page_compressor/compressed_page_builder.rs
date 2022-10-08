use std::{io::Write, num::ParseIntError};

use my_service_bus_abstractions::MessageId;
use prost::DecodeError;
use zip::result::ZipError;

use super::VecWriter;

pub struct CompressedPageBuilder {
    zip_writer: zip::ZipWriter<VecWriter>,
    options: zip::write::FileOptions,
}

impl CompressedPageBuilder {
    pub fn new() -> Self {
        let result = Self {
            zip_writer: zip::ZipWriter::new(VecWriter::new()),
            options: zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        };

        result
    }

    pub fn add_message(&mut self, msg_id: MessageId, payload: &[u8]) -> Result<(), ZipError> {
        let file_name = format!("{}", msg_id);

        self.zip_writer.start_file(file_name, self.options)?;

        let mut pos = 0;
        while pos < payload.len() {
            let size = self.zip_writer.write(&payload[pos..])?;

            pos += size;
        }

        Ok(())
    }

    pub fn get_payload(&mut self) -> Result<Vec<u8>, ZipError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}

#[derive(Debug)]
pub enum CompressedPageReaderError {
    ParseIntError(ParseIntError),
    ZipError(ZipError),
    InvalidSingleFileCompressedPage,
    DecodeError(DecodeError),
}

impl From<ZipError> for CompressedPageReaderError {
    fn from(src: ZipError) -> Self {
        Self::ZipError(src)
    }
}

impl From<ParseIntError> for CompressedPageReaderError {
    fn from(src: ParseIntError) -> Self {
        Self::ParseIntError(src)
    }
}

impl From<DecodeError> for CompressedPageReaderError {
    fn from(src: DecodeError) -> Self {
        Self::DecodeError(src)
    }
}

#[cfg(test)]
mod tests {

    use crate::page_compressor::CompressedPageReader;

    use super::*;

    #[test]
    fn test() {
        let mut builder = CompressedPageBuilder::new();

        builder
            .add_message(1, vec![0u8, 1u8, 2u8].as_slice())
            .unwrap();
        builder
            .add_message(2, vec![3u8, 4u8, 5u8, 6u8].as_slice())
            .unwrap();

        let compressed = builder.get_payload().unwrap();

        let mut reader = CompressedPageReader::new(compressed).unwrap();

        assert_eq!(2, reader.get_files_amount());

        let (msg_id, buf) = reader.get_next_message().unwrap().unwrap();

        assert_eq!(1, msg_id);
        assert_eq!(3, buf.len());
        let (msg_id, buf) = reader.get_next_message().unwrap().unwrap();

        assert_eq!(2, msg_id);
        assert_eq!(4, buf.len());
        let result = reader.get_next_message().unwrap();

        assert_eq!(true, result.is_none());
    }
}
