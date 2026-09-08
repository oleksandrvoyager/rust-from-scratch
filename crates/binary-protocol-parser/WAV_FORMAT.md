# WAV file format (reference)

WAV is a RIFF container: a generic chunk-based wrapper format also used by
AVI and others. All multi-byte numeric fields are **little-endian**. Text
fields are plain ASCII, 1 byte per character.

## RIFF header (bytes 0-11)

| Offset | Size | Field       | Value                                             |
|--------|------|-------------|----------------------------------------------------|
| 0      | 4    | `ChunkID`   | ASCII `"RIFF"`                                      |
| 4      | 4    | `ChunkSize` | `u32`, bytes remaining after this field to EOF      |
| 8      | 4    | `Format`    | ASCII `"WAVE"`                                      |

## `fmt` chunk (bytes 12-35)

| Offset | Size | Field            | Value                                           |
|--------|------|------------------|--------------------------------------------------|
| 12     | 4    | `Subchunk1ID`    | ASCII `"fmt "` (trailing space)                   |
| 16     | 4    | `Subchunk1Size`  | `u32`, size of this chunk's remaining fields (16 for PCM) |
| 20     | 2    | `AudioFormat`    | `u16`, 1 = PCM (uncompressed)                     |
| 22     | 2    | `NumChannels`    | `u16`, 1 = mono, 2 = stereo                       |
| 24     | 4    | `SampleRate`     | `u32`, samples per second (e.g. 44100)            |
| 28     | 4    | `ByteRate`       | `u32`, bytes played per second                    |
| 32     | 2    | `BlockAlign`     | `u16`, bytes per sample frame (all channels)      |
| 34     | 2    | `BitsPerSample`  | `u16`, bit depth per sample (e.g. 16)             |

The spec documents these two fields as always satisfying:
```
BlockAlign = NumChannels * (BitsPerSample / 8)
ByteRate   = SampleRate * BlockAlign
```
This is a stated invariant of the format, not something to derive — real parsers use it to validate a file isn't corrupt, and it means these two fields carry no information beyond what `NumChannels`/`SampleRate`/`BitsPerSample` already give you.

## `data` chunk (bytes 36+)

| Offset | Size          | Field           | Value                                 |
|--------|---------------|-----------------|-----------------------------------------|
| 36     | 4             | `Subchunk2ID`   | ASCII `"data"`                          |
| 40     | 4             | `Subchunk2Size` | `u32`, length of the raw sample data below |
| 44     | `Subchunk2Size` | (sample data) | raw audio samples — this is what we zero-copy borrow instead of copying |

Fixed header size: 44 bytes, followed by `Subchunk2Size` bytes of audio data.
