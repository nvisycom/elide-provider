//! Container detection for the multipart upload.
//!
//! Gladia identifies the container from the filename's extension, so an
//! upload needs one — `meeting.wav`, not `meeting`. [`SttRequest`] carries
//! only bytes, so the extension comes from the magic number at the head of
//! those bytes.
//!
//! [`SttRequest`]: elide_audio::stt::SttRequest

/// The upload filename for `audio`, extension included.
///
/// Falls back to `.wav` when the bytes match nothing known: Gladia still
/// sniffs the payload itself, so a wrong guess is recoverable, whereas no
/// extension at all gives it nothing to dispatch on.
pub(super) fn upload_filename(audio: &[u8]) -> &'static str {
    match extension(audio) {
        Some("mp3") => "audio.mp3",
        Some("flac") => "audio.flac",
        Some("ogg") => "audio.ogg",
        Some("m4a") => "audio.m4a",
        Some("webm") => "audio.webm",
        _ => "audio.wav",
    }
}

/// The extension the leading magic number names, or `None`.
fn extension(audio: &[u8]) -> Option<&'static str> {
    // RIFF....WAVE
    if audio.starts_with(b"RIFF") && audio.get(8..12) == Some(b"WAVE") {
        return Some("wav");
    }
    if audio.starts_with(b"fLaC") {
        return Some("flac");
    }
    if audio.starts_with(b"OggS") {
        return Some("ogg");
    }
    // ISO base media (MP4/M4A): a `ftyp` box at offset 4.
    if audio.get(4..8) == Some(b"ftyp") {
        return Some("m4a");
    }
    // Matroska/WebM EBML header.
    if audio.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return Some("webm");
    }
    // MP3: an ID3 tag, or a frame sync (11 set bits).
    if audio.starts_with(b"ID3") || matches!(audio, [0xFF, b, ..] if b & 0xE0 == 0xE0) {
        return Some("mp3");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_containers_by_magic_number() {
        assert_eq!(upload_filename(b"RIFF\0\0\0\0WAVEfmt "), "audio.wav");
        assert_eq!(upload_filename(b"fLaC\0\0\0\0"), "audio.flac");
        assert_eq!(upload_filename(b"OggS\0\0\0\0"), "audio.ogg");
        assert_eq!(upload_filename(b"\0\0\0\0ftypM4A "), "audio.m4a");
        assert_eq!(
            upload_filename(&[0x1A, 0x45, 0xDF, 0xA3, 0, 0]),
            "audio.webm"
        );
        assert_eq!(upload_filename(b"ID3\x04\0\0\0"), "audio.mp3");
        // A bare MPEG frame sync, with no ID3 tag in front of it.
        assert_eq!(upload_filename(&[0xFF, 0xFB, 0x90, 0x00]), "audio.mp3");
    }

    /// Unknown bytes still get an extension: Gladia sniffs the payload
    /// itself, so a wrong guess is recoverable where a missing extension
    /// leaves it nothing to dispatch on.
    #[test]
    fn unknown_bytes_fall_back_to_wav() {
        assert_eq!(upload_filename(b"not audio at all"), "audio.wav");
        assert_eq!(upload_filename(b""), "audio.wav");
    }

    /// RIFF alone is not enough — it heads other container families too.
    #[test]
    fn riff_without_wave_is_not_wav() {
        assert!(extension(b"RIFF\0\0\0\0AVI ").is_none());
    }
}
