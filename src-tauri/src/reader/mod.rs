use memmap2::Mmap;
use std::fs::File;
use std::io::{BufRead, Read, Seek, SeekFrom};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileEncoding {
    Auto,
    Utf8,
    Ansi,
    Utf16LE,
    Utf16BE,
}

impl FileEncoding {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "utf-8" => FileEncoding::Utf8,
            "ansi" => FileEncoding::Ansi,
            "utf-16le" => FileEncoding::Utf16LE,
            "utf-16be" => FileEncoding::Utf16BE,
            _ => FileEncoding::Auto,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            FileEncoding::Auto => "auto",
            FileEncoding::Utf8 => "utf-8",
            FileEncoding::Ansi => "ansi",
            FileEncoding::Utf16LE => "utf-16le",
            FileEncoding::Utf16BE => "utf-16be",
        }
    }
}

impl Default for FileEncoding {
    fn default() -> Self {
        FileEncoding::Auto
    }
}

pub struct LogFileReader {
    file_path: PathBuf,
    file_size: u64,
    #[allow(dead_code)]
    encoding: FileEncoding,
}

impl LogFileReader {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        Ok(Self {
            file_path: path,
            file_size: metadata.len(),
            encoding: FileEncoding::Auto,
        })
    }
    
    pub fn with_encoding(path: PathBuf, encoding: FileEncoding) -> Result<Self, String> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        Ok(Self {
            file_path: path,
            file_size: metadata.len(),
            encoding,
        })
    }
    
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
    
    pub fn detect_encoding(data: &[u8]) -> FileEncoding {
        if data.len() >= 3 && &data[0..3] == b"\xEF\xBB\xBF" {
            return FileEncoding::Utf8;
        }
        if data.len() >= 2 {
            if &data[0..2] == b"\xFF\xFE" {
                return FileEncoding::Utf16LE;
            }
            if &data[0..2] == b"\xFE\xFF" {
                return FileEncoding::Utf16BE;
            }
        }
        FileEncoding::Utf8
    }
    
    pub fn decode_line(data: &[u8], encoding: FileEncoding) -> String {
        match encoding {
            FileEncoding::Utf8 | FileEncoding::Auto => {
                String::from_utf8_lossy(data).into_owned()
            }
            FileEncoding::Utf16LE => {
                encoding_rs::UTF_16LE.decode(data).0.into_owned()
            }
            FileEncoding::Utf16BE => {
                encoding_rs::UTF_16BE.decode(data).0.into_owned()
            }
            FileEncoding::Ansi => {
                encoding_rs::WINDOWS_1252.decode(data).0.into_owned()
            }
        }
    }
    
    pub fn read_lines(&self) -> Result<LineIterator, String> {
        let file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        Ok(LineIterator::new(file, self.file_size))
    }
    
    pub fn read_lines_mmap(&self) -> Result<MmapReader, String> {
        let file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("Failed to mmap file: {}", e))?;
        
        Ok(MmapReader::new(mmap))
    }
    
    pub fn read_raw_content(&self, offset: u64, length: u32) -> Result<String, String> {
        let mut file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| format!("Failed to seek: {}", e))?;
        
        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer)
            .map_err(|e| format!("Failed to read: {}", e))?;
        
        String::from_utf8(buffer)
            .map_err(|e| format!("Failed to convert to string: {}", e))
    }
}

pub struct LineIterator {
    reader: std::io::BufReader<File>,
    current_offset: u64,
    file_size: u64,
    line_number: u64,
}

impl LineIterator {
    fn new(file: File, file_size: u64) -> Self {
        Self {
            reader: std::io::BufReader::new(file),
            current_offset: 0,
            file_size,
            line_number: 0,
        }
    }
    
    pub fn progress(&self) -> f32 {
        if self.file_size == 0 {
            100.0
        } else {
            (self.current_offset as f32 / self.file_size as f32) * 100.0
        }
    }
}

impl Iterator for LineIterator {
    type Item = Result<(String, u64, u64), std::io::Error>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        let start_offset = self.current_offset;
        
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(bytes_read) => {
                self.current_offset += bytes_read as u64;
                self.line_number += 1;
                let line = line.trim_end_matches('\n').trim_end_matches('\r').to_string();
                Some(Ok((line, self.line_number, start_offset)))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

pub struct MmapLine<'a> {
    pub data: &'a [u8],
    pub line_number: u64,
    pub offset: u64,
}

pub struct MmapReader {
    mmap: Mmap,
    pos: usize,
    line_number: u64,
}

impl MmapReader {
    fn new(mmap: Mmap) -> Self {
        let mut pos = 0;
        
        // Skip UTF-8 BOM if present
        if mmap.len() >= 3 && &mmap[0..3] == b"\xEF\xBB\xBF" {
            pos = 3;
        }
        
        Self {
            mmap,
            pos,
            line_number: 0,
        }
    }

    pub fn next_line(&mut self) -> Option<MmapLine<'_>> {
        if self.pos >= self.mmap.len() {
            return None;
        }

        let start = self.pos;
        let mut end = start;

        while end < self.mmap.len() && self.mmap[end] != b'\n' {
            end += 1;
        }

        let line_data = &self.mmap[start..end];
        self.line_number += 1;
        let line_number = self.line_number;
        let offset = start as u64;

        if end < self.mmap.len() {
            self.pos = end + 1;
        } else {
            self.pos = end;
        }

        Some(MmapLine {
            data: line_data,
            line_number,
            offset,
        })
    }

    pub fn progress(&self) -> f32 {
        if self.mmap.len() == 0 {
            100.0
        } else {
            (self.pos as f32 / self.mmap.len() as f32) * 100.0
        }
    }

    pub fn remaining(&self) -> usize {
        self.mmap.len().saturating_sub(self.pos)
    }

    pub fn total_size(&self) -> usize {
        self.mmap.len()
    }
}
