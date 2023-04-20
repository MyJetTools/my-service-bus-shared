use std::io::Read;

use rust_extensions::{AsSliceOrVec, SliceOrVecSeqReader};
use zip::result::ZipError;

use crate::protobuf_models::MessageProtobufModel;

use super::CompressedPageReaderError;

pub struct CompressedPageReader<'s> {
    zip_archive: zip::ZipArchive<SliceOrVecSeqReader<'s, u8>>,
    file_index: usize,
}

impl<'s> CompressedPageReader<'s> {
    pub fn new(zipped: impl Into<AsSliceOrVec<'s, u8>>) -> Result<Self, ZipError> {
        let zipped: AsSliceOrVec<'_, u8> = zipped.into();

        let zipped: SliceOrVecSeqReader<'_, u8> = zipped.into();

        let zip_archive = zip::ZipArchive::new(zipped)?;
        Ok(Self {
            zip_archive,
            file_index: 0,
        })
    }

    pub fn get_files_amount(&self) -> usize {
        return self.zip_archive.len();
    }

    pub fn get_next_message(
        &mut self,
    ) -> Result<Option<MessageProtobufModel>, CompressedPageReaderError> {
        if self.file_index >= self.zip_archive.len() {
            return Ok(None);
        }

        let mut zip_file = self.zip_archive.by_index(self.file_index)?;

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

        Ok(Some(MessageProtobufModel::parse(result_buffer.as_slice())?))
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
