#[derive(Debug, Clone)]
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        Buffer {
            data: vec![0; size],
        }
    }

    pub fn from_slice(data: &[u8]) -> Self {
        Buffer {
            data: data.to_vec(),
        }
    }

    pub fn from_string(s: &str, encoding: Option<&str>) -> Self {
        match encoding {
            Some("hex") => Buffer {
                data: decode_hex(s),
            },
            Some("base64") => Buffer {
                data: decode_base64(s),
            },
            Some("utf16le") | Some("ucs2") | Some("utf16") => Buffer {
                data: encode_utf16le(s),
            },
            _ => Buffer {
                data: s.as_bytes().to_vec(),
            },
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<u8> {
        self.data.get(index).copied()
    }

    pub fn set(&mut self, index: usize, value: u8) {
        if index < self.data.len() {
            self.data[index] = value;
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn to_string(&self, encoding: Option<&str>) -> String {
        match encoding {
            Some("hex") => encode_hex(&self.data),
            Some("base64") => encode_base64(&self.data),
            Some("utf16le") | Some("ucs2") | Some("utf16") => decode_utf16le(&self.data),
            Some("latin1") => decode_latin1(&self.data),
            _ => String::from_utf8_lossy(&self.data).to_string(),
        }
    }

    pub fn copy_within(&mut self, target: usize, start: usize, end: usize) {
        let len = end - start;
        if target + len <= self.data.len() && end <= self.data.len() {
            let slice = self.data[start..end].to_vec();
            self.data[target..target + len].copy_from_slice(&slice);
        }
    }

    pub fn slice(&self, start: usize, end: usize) -> Buffer {
        let start = if start >= self.data.len() {
            self.data.len()
        } else {
            start
        };
        let end = if end > self.data.len() {
            self.data.len()
        } else {
            end
        };
        Buffer {
            data: self.data[start..end].to_vec(),
        }
    }

    pub fn fill(&mut self, value: u8, start: usize, end: usize) {
        let end = if end > self.data.len() {
            self.data.len()
        } else {
            end
        };
        for i in start..end {
            self.data[i] = value;
        }
    }

    pub fn concat(buffers: &[Buffer]) -> Buffer {
        let total_len: usize = buffers.iter().map(|b| b.len()).sum();
        let mut data = Vec::with_capacity(total_len);
        for b in buffers {
            data.extend_from_slice(b.as_slice());
        }
        Buffer { data }
    }

    pub fn equals(&self, other: &Buffer) -> bool {
        self.data == other.data
    }

    pub fn compare(&self, other: &Buffer) -> i32 {
        let min_len = self.data.len().min(other.data.len());
        for i in 0..min_len {
            match self.data[i].cmp(&other.data[i]) {
                std::cmp::Ordering::Less => return -1,
                std::cmp::Ordering::Greater => return 1,
                std::cmp::Ordering::Equal => {}
            }
        }
        match self.data.len().cmp(&other.data.len()) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Greater => 1,
            std::cmp::Ordering::Equal => 0,
        }
    }

    pub fn index_of(&self, value: u8, start: Option<usize>) -> Option<usize> {
        let start = start.unwrap_or(0);
        self.data
            .iter()
            .skip(start)
            .position(|&b| b == value)
            .map(|i| i + start)
    }

    pub fn includes(&self, value: u8, start: Option<usize>) -> bool {
        self.index_of(value, start).is_some()
    }
}

fn encode_hex(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn decode_hex(s: &str) -> Vec<u8> {
    let s = s.trim();
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);
    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

fn decode_base64(s: &str) -> Vec<u8> {
    let mut result = Vec::new();
    let s = s.trim_end_matches('=');
    let mut buf = 0u32;
    let mut bits = 0u32;
    for ch in s.chars() {
        let val = match ch {
            'A'..='Z' => ch as u32 - 'A' as u32,
            'a'..='z' => ch as u32 - 'a' as u32 + 26,
            '0'..='9' => ch as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            _ => continue,
        };
        buf = (buf << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buf >> bits) as u8);
        }
    }
    result
}

fn encode_utf16le(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .flat_map(|c| c.to_le_bytes().to_vec())
        .collect()
}

fn decode_utf16le(data: &[u8]) -> String {
    let chunks = data.chunks_exact(2);
    let remainder = chunks.remainder();
    let code_points: Vec<u16> = chunks
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let mut result = String::from_utf16_lossy(&code_points);
    if !remainder.is_empty() {
        result.push(char::from(remainder[0]));
    }
    result
}

fn decode_latin1(data: &[u8]) -> String {
    data.iter().map(|&b| b as char).collect()
}
