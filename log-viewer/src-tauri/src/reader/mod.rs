use memmap2::Mmap;
use std::fs::File;
use std::io::{BufRead, Read, Seek, SeekFrom};
use std::path::PathBuf;

pub struct LogFileReader {
    file_path: PathBuf,
    file_size: u64,
}

impl LogFileReader {
    pub fn new(path: PathBuf) -> Result<Self, String> {
        let metadata = std::fs::metadata(&path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;
        
        Ok(Self {
            file_path: path,
            file_size: metadata.len(),
        })
    }
    
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
    
    pub fn read_lines(&self) -> Result<LineIterator, String> {
        let file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        Ok(LineIterator::new(file, self.file_size))
    }
    
    pub fn read_lines_mmap(&self) -> Result<MmapLineIterator, String> {
        let file = File::open(&self.file_path)
            .map_err(|e| format!("Failed to open file: {}", e))?;
        
        let mmap = unsafe { Mmap::map(&file) }
            .map_err(|e| format!("Failed to mmap file: {}", e))?;
        
        Ok(MmapLineIterator::new(mmap))
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

pub struct MmapLineIterator {
    mmap: Option<Mmap>,
    pos: usize,
    line_number: u64,
}

impl MmapLineIterator {
    fn new(mmap: Mmap) -> Self {
        Self { 
            mmap: Some(mmap), 
            pos: 0,
            line_number: 0,
        }
    }
    
    pub fn progress(&self) -> f32 {
        if let Some(ref mmap) = self.mmap {
            if mmap.len() == 0 {
                100.0
            } else {
                (self.pos as f32 / mmap.len() as f32) * 100.0
            }
        } else {
            100.0
        }
    }
}

pub struct MmapLine {
    pub data: Vec<u8>,
    pub line_number: u64,
    pub offset: u64,
}

impl Iterator for MmapLineIterator {
    type Item = MmapLine;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mmap = self.mmap.as_ref()?;
        
        if self.pos >= mmap.len() {
            return None;
        }
        
        let start = self.pos;
        let mut end = start;
        
        while end < mmap.len() && mmap[end] != b'\n' {
            end += 1;
        }
        
        let line = mmap[start..end].to_vec();
        self.line_number += 1;
        let line_number = self.line_number;
        let offset = start as u64;
        
        if end < mmap.len() {
            self.pos = end + 1;
        } else {
            self.pos = end;
        }
        
        Some(MmapLine {
            data: line,
            line_number,
            offset,
        })
    }
}
