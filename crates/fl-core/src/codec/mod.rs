//! Byte-exact codecs: the lossless text codecs of the managed documents (IR Def. 1), and the
//! game's binary settings files (`settings_dat`, `graphics`).
//!
//! The text codecs keep the file as a list of lines with their exact line endings, so that
//! `render(parse(x)) == x` byte for byte (the lens law ρ∘π = id). Edits touch only the lines
//! of the managed entries; the binary ones write only documented bytes.

pub mod graphics;
pub mod map_teams;
pub mod settings_dat;
pub mod sider_ini;

use serde::{Deserialize, Serialize};

/// How the raw bytes of a document were decoded. Latin-1 is the lossless fallback for files
/// that are not valid UTF-8 (every byte maps to one char and back).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Encoding {
    Utf8,
    Latin1,
}

/// The text of a document plus its encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocText {
    pub encoding: Encoding,
    pub text: String,
}

impl DocText {
    pub fn decode(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(s) => DocText {
                encoding: Encoding::Utf8,
                text: s.to_string(),
            },
            Err(_) => DocText {
                encoding: Encoding::Latin1,
                text: bytes.iter().map(|&b| b as char).collect(),
            },
        }
    }

    /// Encodes text back to bytes. Fails only if a Latin-1 document received a character
    /// outside U+0000..U+00FF (from a manifest entry).
    pub fn encode(encoding: Encoding, text: &str) -> Result<Vec<u8>, char> {
        match encoding {
            Encoding::Utf8 => Ok(text.as_bytes().to_vec()),
            Encoding::Latin1 => text
                .chars()
                .map(|c| u8::try_from(u32::from(c)).map_err(|_| c))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Eol {
    CrLf,
    Lf,
    /// The last line of a file without a trailing newline.
    None,
}

impl Eol {
    fn as_str(self) -> &'static str {
        match self {
            Eol::CrLf => "\r\n",
            Eol::Lf => "\n",
            Eol::None => "",
        }
    }
}

/// One physical line: its text without the terminator, and the terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub text: String,
    pub eol: Eol,
}

pub(crate) fn split_lines(s: &str) -> Vec<Line> {
    let line = |l: &str| {
        let (text, eol) = match l.strip_suffix('\n') {
            Some(b) => b
                .strip_suffix('\r')
                .map_or((b, Eol::Lf), |t| (t, Eol::CrLf)),
            None => (l, Eol::None),
        };
        let text = text.to_string();
        Line { text, eol }
    };
    s.split_inclusive('\n').map(line).collect()
}

pub(crate) fn join_lines(lines: &[Line]) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(&l.text);
        s.push_str(l.eol.as_str());
    }
    s
}

/// The line ending used for new lines: CRLF if the file uses any CRLF, else LF.
pub(crate) fn dominant_eol(lines: &[Line]) -> Eol {
    if lines.iter().any(|l| l.eol == Eol::CrLf) || lines.is_empty() {
        Eol::CrLf
    } else {
        Eol::Lf
    }
}

/// Repairs terminators after lines were inserted or removed: only the last line may lack a
/// terminator, and it lacks one iff the original file ended without a newline.
pub(crate) fn fix_terminators(lines: &mut [Line], dominant: Eol, unterminated: bool) {
    for l in lines.iter_mut() {
        if l.eol == Eol::None {
            l.eol = dominant;
        }
    }
    if unterminated && let Some(last) = lines.last_mut() {
        last.eol = Eol::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_round_trip() {
        for s in ["", "a", "a\r\nb\nc", "a\r\n", "\r\n\r\n", "x\ry\n"] {
            assert_eq!(join_lines(&split_lines(s)), s);
        }
    }

    #[test]
    fn latin1_fallback_is_lossless() {
        let bytes = b"caf\xe9\r\n";
        let d = DocText::decode(bytes);
        assert_eq!(d.encoding, Encoding::Latin1);
        assert_eq!(DocText::encode(d.encoding, &d.text).unwrap(), bytes);
        assert!(DocText::encode(Encoding::Latin1, "\u{20ac}").is_err());
    }

    #[test]
    fn appending_keeps_missing_final_newline() {
        let mut l = split_lines("a\r\nb");
        l.push(Line {
            text: "c".into(),
            eol: Eol::CrLf,
        });
        fix_terminators(&mut l, Eol::CrLf, true);
        assert_eq!(join_lines(&l), "a\r\nb\r\nc");
    }
}
