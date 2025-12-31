use crate::error::SupportError;
use crate::tas::{FrameFlags, InputFrame, Movie, TasData};
use std::io::{BufRead, Cursor};

/// FM2-specific header metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fm2Header {
    pub version: u32,
    pub emu_version: String,
    pub rerecord_count: u32,
    pub pal_flag: bool,
    pub rom_filename: String,
    pub guid: String,
    pub fourscore: bool,
    pub microphone: bool,
    pub ports: [u8; 3],
    pub fds: bool,
    pub ppu_flag: bool,
    pub ram_init_option: u32,
    pub ram_init_seed: u32,
    pub comments: Vec<String>,
    pub subtitles: Vec<String>,
    pub binary_flag: bool,
}

#[derive(Debug, PartialEq, Eq)]
enum ParseState {
    Newline,
    Key,
    Separator,
    Value,
}

pub fn parse<R: BufRead>(mut reader: R) -> Result<Movie, SupportError> {
    let mut version_buf = [0u8; 9];
    reader
        .read_exact(&mut version_buf)
        .map_err(SupportError::Io)?;
    if &version_buf != b"version 3" {
        return Err(SupportError::InvalidTasData(
            "Invalid FM2 version header".to_string(),
        ));
    }

    let mut header = Fm2Header {
        version: 3,
        ..Default::default()
    };
    let mut frames = Vec::new();
    let mut savestate = None;

    let mut state = ParseState::Newline;
    let mut key = String::new();
    let mut value = String::new();

    let mut bytes_iter = reader.bytes();

    loop {
        let byte = match bytes_iter.next() {
            Some(Ok(b)) => b,
            Some(Err(e)) => return Err(SupportError::Io(e)),
            None => break,
        };

        let c = byte as char;
        let is_whitespace = c == ' ' || c == '\t';
        let is_rec_char = c == '|';
        let is_newline = c == '\n' || c == '\r';

        match state {
            ParseState::Newline => {
                if is_newline || is_whitespace {
                    continue;
                }
                if is_rec_char {
                    let mut line_buf = Vec::new();
                    line_buf.push(byte);
                    for res in bytes_iter.by_ref() {
                        let b = res.map_err(SupportError::Io)?;
                        if b == b'\n' || b == b'\r' {
                            break;
                        }
                        line_buf.push(b);
                    }
                    parse_record_line(&line_buf, &header, &mut frames)?;
                    state = ParseState::Newline;
                } else {
                    key.clear();
                    value.clear();
                    key.push(c);
                    state = ParseState::Key;
                }
            }
            ParseState::Key => {
                if is_whitespace {
                    state = ParseState::Separator;
                } else if is_newline {
                    install_value(&mut header, &mut savestate, &key, &value)?;
                    state = ParseState::Newline;
                } else {
                    key.push(c);
                }
            }
            ParseState::Separator => {
                if is_newline {
                    install_value(&mut header, &mut savestate, &key, &value)?;
                    state = ParseState::Newline;
                } else if !is_whitespace {
                    value.push(c);
                    state = ParseState::Value;
                }
            }
            ParseState::Value => {
                if is_newline {
                    install_value(&mut header, &mut savestate, &key, &value)?;
                    state = ParseState::Newline;
                } else {
                    value.push(c);
                }
            }
        }
    }

    if state == ParseState::Value || state == ParseState::Key {
        install_value(&mut header, &mut savestate, &key, &value)?;
    }

    Ok(Movie {
        is_pal: header.pal_flag,
        rom_hash: None,
        data: TasData::Fm2(header),
        frames,
        savestate,
    })
}

fn install_value(
    header: &mut Fm2Header,
    savestate: &mut Option<Vec<u8>>,
    key: &str,
    val: &str,
) -> Result<(), SupportError> {
    match key {
        "emuVersion" => header.emu_version = val.to_string(),
        "rerecordCount" => header.rerecord_count = val.parse().unwrap_or(0),
        "palFlag" => header.pal_flag = val == "1",
        "romFilename" => header.rom_filename = val.to_string(),
        "guid" => header.guid = val.to_string(),
        "fourscore" => header.fourscore = val == "1",
        "microphone" => header.microphone = val == "1",
        "port0" => header.ports[0] = val.parse().unwrap_or(0),
        "port1" => header.ports[1] = val.parse().unwrap_or(0),
        "port2" => header.ports[2] = val.parse().unwrap_or(0),
        "FDS" => header.fds = val == "1",
        "NewPPU" => header.ppu_flag = val == "1",
        "RAMInitOption" => header.ram_init_option = val.parse().unwrap_or(0),
        "RAMInitSeed" => header.ram_init_seed = val.parse().unwrap_or(0),
        "comment" => header.comments.push(val.to_string()),
        "subtitle" => header.subtitles.push(val.to_string()),
        "binary" => header.binary_flag = val == "1",
        "savestate" => {
            if let Ok(bytes) = hex::decode(val) {
                *savestate = Some(bytes);
            }
        }
        _ => {}
    }
    Ok(())
}

fn parse_record_line(
    line: &[u8],
    header: &Fm2Header,
    frames: &mut Vec<InputFrame>,
) -> Result<(), SupportError> {
    let s = std::str::from_utf8(line)
        .map_err(|_| SupportError::InvalidTasData("Invalid UTF-8 in movie record".to_string()))?;
    let parts: Vec<&str> = s.split('|').collect();
    if parts.len() < 3 {
        return Ok(());
    }

    let commands_u8: u8 = parts[1].parse().unwrap_or(0);
    let commands = FrameFlags::from_bits_truncate(commands_u8);
    let mut frame = InputFrame {
        commands,
        ports: [0; 4],
    };
    let mut part_idx = 2;

    if header.fourscore {
        for i in 0..4 {
            if part_idx < parts.len() {
                frame.ports[i] = parse_joy(parts[part_idx]);
                part_idx += 1;
            }
        }
    } else {
        for i in 0..2 {
            if part_idx < parts.len() {
                if header.ports[i] != 2 {
                    frame.ports[i] = parse_joy(parts[part_idx]);
                }
                part_idx += 1;
            }
        }
    }
    frames.push(frame);
    Ok(())
}

fn parse_joy(s: &str) -> u8 {
    let mut mask = 0u8;
    for c in s.chars() {
        if c == '|' {
            break;
        }
        mask <<= 1;
        if c != '.' && c != ' ' {
            mask |= 1;
        }
    }
    mask
}

pub fn parse_str(s: &str) -> Result<Movie, SupportError> {
    parse(Cursor::new(s))
}
