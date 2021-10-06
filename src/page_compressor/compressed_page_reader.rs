use std::io::{Cursor, Read};

use zip::result::ZipError;

use crate::{
    protobuf_models::{MessageProtobufModel, MessagesProtobufModel},
    MessageId,
};

use super::CompressedPageReaderError;

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

    pub fn unzip_messages(&mut self) -> Result<MessagesProtobufModel, CompressedPageReaderError> {
        let result = self.decompress_as_single_file();

        if let Ok(uncompressed) = &result {
            let result = MessagesProtobufModel::parse(uncompressed)?;
            return Ok(result);
        }

        let err = result.err().unwrap();

        if let CompressedPageReaderError::InvalidSingleFileCompressedPage = &err {
            let mut result = MessagesProtobufModel {
                messages: Vec::new(),
            };

            while let Some(next_message) = self.get_next_message()? {
                let msg = MessageProtobufModel::parse(&next_message.1)?;
                result.messages.push(msg)
            }

            return Ok(result);
        }

        return Err(err);
    }

    pub fn get_next_message(
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

        if self.zip_archive.len() == 0 {
            return Err(CompressedPageReaderError::InvalidSingleFileCompressedPage);
        }

        let mut zip_file = self.zip_archive.by_index(0)?;

        if zip_file.name() != "d" {
            return Err(CompressedPageReaderError::InvalidSingleFileCompressedPage);
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
