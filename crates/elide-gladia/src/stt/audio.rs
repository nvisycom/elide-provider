//! Container detection for the multipart upload.
//!
//! Gladia identifies the container from the filename's extension, so an
//! upload needs one — `meeting.wav`, not `meeting`. [`SttRequest`] carries
//! bytes, so the name comes from their magic number.
//!
//! The containers here are the ones [`AudioFormat`] names: what this
//! toolkit accepts, not everything Gladia would take.
//!
//! [`SttRequest`]: elide_audio::stt::SttRequest
//! [`AudioFormat`]: elide_audio::modality::AudioFormat

/// The upload filename for `audio`.
///
/// Falls back to `audio.wav` when the bytes are not recognisably MP3:
/// Gladia sniffs the payload itself, so a wrong guess is recoverable,
/// whereas no extension at all leaves it nothing to dispatch on.
pub(super) fn upload_filename(audio: &[u8]) -> &'static str {
    if infer::audio::is_mp3(audio) {
        return "audio.mp3";
    }
    "audio.wav"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mp3_by_id3_tag_or_frame_sync() {
        assert_eq!(upload_filename(b"ID3\x04\0\0\0"), "audio.mp3");
        assert_eq!(upload_filename(&[0xFF, 0xFB, 0x90, 0x00]), "audio.mp3");
        assert_eq!(upload_filename(&[0xFF, 0xF3, 0x90, 0x00]), "audio.mp3");
    }

    /// ADTS AAC shares MPEG's sync word, and a hand-rolled frame-sync check
    /// swallows it. `infer` separates the two, so AAC does not arrive
    /// labelled `.mp3`.
    #[test]
    fn adts_aac_is_not_mistaken_for_mp3() {
        assert_eq!(upload_filename(&[0xFF, 0xF1, 0x50, 0x80]), "audio.wav");
        assert_eq!(upload_filename(&[0xFF, 0xF9, 0x50, 0x80]), "audio.wav");
    }

    /// Everything else is uploaded as WAV, including bytes that are not
    /// audio at all: Gladia sniffs the payload, so the extension only has
    /// to give it somewhere to start.
    #[test]
    fn everything_else_is_wav() {
        assert_eq!(upload_filename(b"RIFF\0\0\0\0WAVEfmt "), "audio.wav");
        assert_eq!(upload_filename(b"not audio at all"), "audio.wav");
        assert_eq!(upload_filename(b""), "audio.wav");
    }
}
