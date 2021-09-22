use std::{
    io::{Cursor, Read, Write},
    num::ParseIntError,
};

use zip::result::ZipError;

use crate::MessageId;

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

    pub fn add_page(&mut self, msg_id: MessageId, payload: &[u8]) -> Result<(), ZipError> {
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
    Other(String),
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

pub struct CompressedPageReader {
    zip_archive: zip::ZipArchive<Cursor<Vec<u8>>>,
    file_index: usize,
}

impl CompressedPageReader {
    pub fn new(zipped: Vec<u8>) -> Result<Self, ZipError> {
        let zip_archive = zip::ZipArchive::new(Cursor::new(zipped))?;
        Ok(Self {
            zip_archive,
            file_index: 0,
        })
    }

    pub fn get_files_amount(&self) -> usize {
        return self.zip_archive.len();
    }

    pub fn get_next_page(
        &mut self,
    ) -> Result<Option<(MessageId, Vec<u8>)>, CompressedPageReaderError> {
        if self.file_index >= self.zip_archive.len() {
            return Ok(None);
        }

        let mut zip_file = self.zip_archive.by_index(self.file_index)?;

        let message_id = zip_file.name().parse::<i64>()?;

        let mut result_buffer: Vec<u8> = Vec::new();

        loop {
            let mut buffer = [0u8; 1024 * 1024];

            let read_size = zip_file.read(&mut buffer[..]);

            if let Err(err) = read_size {
                return Err(CompressedPageReaderError::ZipError(err.into()));
            }

            let read_size = read_size.unwrap();

            if read_size == 0 {
                break;
            }

            result_buffer.extend(&buffer[..read_size]);
        }
        self.file_index += 1;

        Ok(Some((message_id, result_buffer)))
    }

    pub fn decompress_as_single_file(&mut self) -> Result<Vec<u8>, CompressedPageReaderError> {
        let mut page_buffer: Vec<u8> = Vec::new();

        let mut zip_file = self.zip_archive.by_index(0)?;

        if zip_file.name() != "d" {
            return Err(CompressedPageReaderError::Other(
                "Single file has to have d name in zipped page".to_string(),
            ));
        }
        let mut buffer = [0u8; 1024 * 1024];

        loop {
            let read_size = zip_file.read(&mut buffer[..]);
            if let Err(err) = read_size {
                return Err(CompressedPageReaderError::ZipError(err.into()));
            }

            let read_size = read_size.unwrap();

            if read_size == 0 {
                break;
            }

            page_buffer.extend(&buffer[..read_size]);
        }

        Ok(page_buffer)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let mut builder = CompressedPageBuilder::new();

        builder.add_page(1, vec![0u8, 1u8, 2u8].as_slice()).unwrap();
        builder
            .add_page(2, vec![3u8, 4u8, 5u8, 6u8].as_slice())
            .unwrap();

        let compressed = builder.get_payload().unwrap();

        let mut reader = CompressedPageReader::new(compressed).unwrap();

        let (msg_id, buf) = reader.get_next_page().unwrap().unwrap();

        assert_eq!(1, msg_id);
        assert_eq!(3, buf.len());
        let (msg_id, buf) = reader.get_next_page().unwrap().unwrap();

        assert_eq!(2, msg_id);
        assert_eq!(4, buf.len());
        let result = reader.get_next_page().unwrap();

        assert_eq!(true, result.is_none());
    }
}
