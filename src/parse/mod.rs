mod encoding;
mod multi;
mod ptr_encoding;
mod struct_encoding;

use encoding::Primitive;

pub use self::encoding::StrEncoding;
pub use self::ptr_encoding::StrPointerEncoding;
pub use self::struct_encoding::StrStructEncoding;

pub fn chomp(s: &str) -> (Option<&str>, &str) {
    let head_len = chomp_ptr(s)
        .or_else(|| chomp_nested_delims(s, '[', ']'))
        .or_else(|| chomp_nested_delims(s, '{', '}'))
        .or_else(|| chomp_nested_delims(s, '(', ')'))
        .or_else(|| {
            if let (Some(_), t) = chomp_primitive(s) {
                Some(s.len() - t.len())
            } else {
                None
            }
        });

    if let Some(head_len) = head_len {
        let (h, t) = s.split_at(head_len);
        (Some(h), t)
    } else {
        (None, s)
    }
}

fn chomp_ptr(s: &str) -> Option<usize> {
    if s.starts_with("^") {
        let (h, _) = chomp(&s[1..]);
        h.map(|h| h.len() + 1)
    } else {
        None
    }
}

fn chomp_nested_delims(s: &str, open: char, close: char) -> Option<usize> {
    if !s.starts_with(open) {
        return None;
    }

    let mut depth = 1;
    for (i, c) in s.char_indices().skip(1) {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
        }

        if depth == 0 {
            return Some(i + 1);
        }
    }

    None
}

fn chomp_primitive(s: &str) -> (Option<Primitive>, &str) {
    if s.is_empty() {
        return (None, s);
    }

    let (h, t) = s.split_at(1);
    match h {
        "c" => (Some(Primitive::Char), t),
        "i" => (Some(Primitive::Int), t),
        "b" => {
            // Chomp until we hit a non-digit
            let (num, t) = match t.find(|c: char| !c.is_digit(10)) {
                Some(i) => t.split_at(i),
                None => (t, ""),
            };
            match num.parse() {
                Ok(b) => (Some(Primitive::BitField(b)), t),
                Err(_) => (None, s),
            }
        }
        _ => (None, s),
    }
}

enum ParseResult {
    Primitive(Primitive),
    Pointer,
    Array,
    Struct,
    Union,
    Error,
}

fn parse(s: &str) -> ParseResult {
    if s.starts_with('^') {
        ParseResult::Pointer
    } else if s.starts_with('[') && s.ends_with(']') {
        ParseResult::Array
    } else if s.starts_with('{') && s.ends_with('}') {
        ParseResult::Struct
    } else if s.starts_with('(') && s.ends_with(')') {
        ParseResult::Union
    } else {
        let (h, t) = chomp_primitive(s);
        if !t.is_empty() {
            ParseResult::Error
        } else if let Some(p) = h {
            ParseResult::Primitive(p)
        } else {
            ParseResult::Error
        }
    }
}

fn parse_struct(s: &str) -> Option<(&str, &str)> {
    if let Some(sep_pos) = s.find('=') {
        let name = &s[1..sep_pos];
        let fields = &s[sep_pos + 1..s.len() - 1];
        Some((name, fields))
    } else {
        None
    }
}

fn parse_array(s: &str) -> Option<(u32, &str)> {
    if let Some(sep_pos) = s.find('^') {
        let len = &s[1..sep_pos];
        let item = &s[sep_pos + 1..s.len() - 1];
        if let Ok(len) = len.parse() {
            Some((len, item))
        } else {
            None
        }
    } else {
        None
    }
}

fn is_valid(s: &str) -> bool {
    match parse(s) {
        ParseResult::Primitive(_) => true,
        ParseResult::Pointer => {
            let target = &s[1..];
            is_valid(target)
        },
        ParseResult::Struct => {
            let mut fields = match parse_struct(s) {
                Some((_, fields)) => fields,
                _ => return false,
            };
            while !fields.is_empty() {
                let (h, t) = chomp(fields);
                if h.map_or(false, is_valid) {
                    return false;
                }
                fields = t;
            }
            true
        }
        ParseResult::Array |
        ParseResult::Union |
        ParseResult::Error => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chomp() {
        let (h, t) = chomp("{A={B=ci^{C=c}}ci}c^i{C=c}");
        assert_eq!(h, Some("{A={B=ci^{C=c}}ci}"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("c"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("^i"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("{C=c}"));

        let (h, _) = chomp(t);
        assert_eq!(h, None);
    }

    #[test]
    fn test_chomp_delims() {
        let (h, t) = chomp("{A=(B=ci)ci}[12^{C=c}]c(D=ci)i");
        assert_eq!(h, Some("{A=(B=ci)ci}"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("[12^{C=c}]"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("c"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("(D=ci)"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("i"));

        let (h, _) = chomp(t);
        assert_eq!(h, None);
    }

    #[test]
    fn test_chomp_bad_delims() {
        let (h, _) = chomp("{A={B=ci}ci");
        assert_eq!(h, None);

        let (h, t) = chomp("{A=(B=ci}[12^{C=c})]");
        assert_eq!(h, Some("{A=(B=ci}"));

        let (h, t) = chomp(t);
        assert_eq!(h, Some("[12^{C=c})]"));

        let (h, _) = chomp(t);
        assert_eq!(h, None);
    }

    #[test]
    fn test_parse_bitfield() {
        match parse("b32") {
            ParseResult::Primitive(Primitive::BitField(32)) => (),
            _ => panic!("Bit field parsed incorrectly"),
        };

        match parse("b-32") {
            ParseResult::Error => (),
            _ => panic!("Invalid bit field was accepted"),
        };

        match chomp_primitive("b32b32") {
            (Some(Primitive::BitField(32)), "b32") => (),
            _ => panic!("Bit field wasn't chomped correctly"),
        };

        match chomp_primitive("bb32") {
            (None, "bb32") => (),
            _ => panic!("Invalid bit field was chomped"),
        };
    }
}