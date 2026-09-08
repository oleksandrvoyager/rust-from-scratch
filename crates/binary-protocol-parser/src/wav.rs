//! Zero-copy parser for the WAV (RIFF/WAVE) file format.
//!
//! See `WAV_FORMAT.md` at the crate root for the full field layout.

use std::ops::Range;

/// Fixed header size: everything up to and including `Subchunk2Size`.
const HEADER_SIZE: usize = 44;

/// `ChunkID`: must be ASCII `"RIFF"`.
const RIFF_MAGIC_RANGE: Range<usize> = 0..4;

/// `Format`: must be ASCII `"WAVE"`.
const WAVE_MAGIC_RANGE: Range<usize> = 8..12;

/// `Subchunk1ID`: must be ASCII `"fmt "` (trailing space).
const FMT_MAGIC_RANGE: Range<usize> = 12..16;

/// `AudioFormat`: `u16`, must be `1` (PCM) — anything else is unsupported.
const AUDIO_FORMAT_RANGE: Range<usize> = 20..22;

/// `NumChannels`: `u16`, e.g. 1 = mono, 2 = stereo.
const NUM_CHANNELS_RANGE: Range<usize> = 22..24;

/// `SampleRate`: `u32`, samples per second (e.g. 44100).
const SAMPLE_RATE_RANGE: Range<usize> = 24..28;

/// `BitsPerSample`: `u16`, bit depth per sample (e.g. 16).
const BITS_PER_SAMPLE_RANGE: Range<usize> = 34..36;

/// `Subchunk2ID`: must be ASCII `"data"`.
const DATA_MAGIC_RANGE: Range<usize> = 36..40;

/// `Subchunk2Size`: `u32`, length of the raw sample data that follows the header.
const PAYLOAD_LEN_RANGE: Range<usize> = 40..HEADER_SIZE;

pub struct WavFile<'a> {
    pub num_channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub data: &'a [u8],
}

#[derive(Debug)]
pub enum WavParseError<'a> {
    UnexpectedEof,
    UnsupportedFormat { format: u16 },
    UnexpectedValue { expected: &'a [u8], found: &'a [u8] },
}

impl<'a> WavFile<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self, WavParseError<'a>> {
        if input.len() < HEADER_SIZE {
            return Err(WavParseError::UnexpectedEof);
        }

        expect_magic(&input[RIFF_MAGIC_RANGE], b"RIFF")?;
        expect_magic(&input[WAVE_MAGIC_RANGE], b"WAVE")?;
        expect_magic(&input[FMT_MAGIC_RANGE], b"fmt ")?;

        let audio_format = read_u16(&input[AUDIO_FORMAT_RANGE]);
        if audio_format != 1 {
            return Err(WavParseError::UnsupportedFormat {
                format: audio_format,
            });
        }

        expect_magic(&input[DATA_MAGIC_RANGE], b"data")?;

        let payload_len = read_u32(&input[PAYLOAD_LEN_RANGE]);
        if HEADER_SIZE + payload_len as usize > input.len() {
            return Err(WavParseError::UnexpectedEof);
        }

        let wav_file = Self {
            num_channels: read_u16(&input[NUM_CHANNELS_RANGE]),
            sample_rate: read_u32(&input[SAMPLE_RATE_RANGE]),
            bits_per_sample: read_u16(&input[BITS_PER_SAMPLE_RANGE]),
            data: &input[HEADER_SIZE..(HEADER_SIZE + payload_len as usize)],
        };

        Ok(wav_file)
    }
}

fn expect_magic<'a>(found: &'a [u8], expected: &'static [u8]) -> Result<(), WavParseError<'a>> {
    if found == expected {
        Ok(())
    } else {
        Err(WavParseError::UnexpectedValue { expected, found })
    }
}

fn read_u16(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().expect("caller must pass exactly 2 bytes"))
}

fn read_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("caller must pass exactly 4 bytes"))
}
