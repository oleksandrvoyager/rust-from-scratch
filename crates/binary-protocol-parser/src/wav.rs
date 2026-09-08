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

#[cfg(test)]
mod tests {
    use super::*;

    fn build_wav(
        num_channels: u16,
        sample_rate: u32,
        bits_per_sample: u16,
        payload: &[u8],
    ) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36u32 + payload.len() as u32).to_le_bytes());
        v.extend_from_slice(b"WAVE");
        v.extend_from_slice(b"fmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&num_channels.to_le_bytes());
        v.extend_from_slice(&sample_rate.to_le_bytes());
        let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
        v.extend_from_slice(&byte_rate.to_le_bytes());
        let block_align = num_channels * (bits_per_sample / 8);
        v.extend_from_slice(&block_align.to_le_bytes());
        v.extend_from_slice(&bits_per_sample.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        v.extend_from_slice(payload);
        v
    }

    #[test]
    fn parses_valid_wav() {
        let payload = [1u8, 2, 3, 4, 5, 6];
        let bytes = build_wav(2, 44100, 16, &payload);
        let wav = WavFile::parse(&bytes).expect("should parse");
        assert_eq!(wav.num_channels, 2);
        assert_eq!(wav.sample_rate, 44100);
        assert_eq!(wav.bits_per_sample, 16);
        assert_eq!(wav.data, &payload);
    }

    #[test]
    fn rejects_too_short() {
        let bytes = vec![0u8; 10];
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedEof)
        ));
    }

    #[test]
    fn rejects_bad_riff_magic() {
        let mut bytes = build_wav(1, 8000, 8, &[9, 9]);
        bytes[0] = b'X';
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedValue { .. })
        ));
    }

    #[test]
    fn rejects_bad_wave_magic() {
        let mut bytes = build_wav(1, 8000, 8, &[9, 9]);
        bytes[8] = b'X';
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedValue { .. })
        ));
    }

    #[test]
    fn rejects_bad_fmt_magic() {
        let mut bytes = build_wav(1, 8000, 8, &[9, 9]);
        bytes[12] = b'X';
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedValue { .. })
        ));
    }

    #[test]
    fn rejects_bad_data_magic() {
        let mut bytes = build_wav(1, 8000, 8, &[9, 9]);
        bytes[36] = b'X';
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedValue { .. })
        ));
    }

    #[test]
    fn rejects_truncated_payload() {
        let mut bytes = build_wav(1, 8000, 8, &[9, 9, 9, 9]);
        bytes.truncate(bytes.len() - 2);
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnexpectedEof)
        ));
    }

    #[test]
    fn rejects_non_pcm_format() {
        let mut bytes = build_wav(1, 8000, 8, &[1, 2]);
        bytes[20] = 2;
        assert!(matches!(
            WavFile::parse(&bytes),
            Err(WavParseError::UnsupportedFormat { format: 2 })
        ));
    }

    #[test]
    fn zero_length_payload() {
        let bytes = build_wav(1, 8000, 8, &[]);
        let wav = WavFile::parse(&bytes).expect("should parse");
        assert_eq!(wav.data.len(), 0);
    }

    #[test]
    fn stereo_16bit_roundtrip() {
        let payload = vec![0u8; 100];
        let bytes = build_wav(2, 48000, 24, &payload);
        let wav = WavFile::parse(&bytes).expect("should parse");
        assert_eq!(wav.num_channels, 2);
        assert_eq!(wav.sample_rate, 48000);
        assert_eq!(wav.bits_per_sample, 24);
        assert_eq!(wav.data.len(), 100);
    }
}
