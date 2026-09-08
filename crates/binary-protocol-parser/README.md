# binary-protocol-parser

Topic: memory layout (alignment, endianness, zero-copy lifetimes).

Zero-copy parser for the WAV (RIFF/WAVE) audio file format, PCM only.
`WavFile<'a>` borrows the raw sample data directly from the input `&[u8]` —
no allocation, no copying. Field layout: [`WAV_FORMAT.md`](WAV_FORMAT.md).

## Usage

```rust
use binary_protocol_parser::wav::WavFile;

let wav = WavFile::parse(&bytes)?;
println!("{} Hz, {} channels, {} bits", wav.sample_rate, wav.num_channels, wav.bits_per_sample);
// wav.data is a &[u8] slice into `bytes` — no copy of the audio samples was made.
```

## Zero-copy vs. copying

An owned/copying parser would allocate a new `Vec<u8>` and copy the sample
data into it — costs an allocation plus a copy proportional to file size,
but the result has no lifetime tied to the input.

`WavFile<'a>` instead borrows: `data: &'a [u8]` points directly into the
buffer passed to `parse`. No allocation, no copy, regardless of how large
the audio data is — but `WavFile<'a>` can't outlive that buffer, enforced
at compile time via `'a`.

## Fields kept vs. only validated

Not every field in the WAV spec is stored on `WavFile`. Constant markers
(`"RIFF"`, `"WAVE"`, `"fmt "`, `"data"`) and fields derivable from others
(`ByteRate`, `BlockAlign`, `ChunkSize`, `Subchunk2Size`) are checked during
parsing but not kept — they carry no information beyond what's already
validated or already stored.
