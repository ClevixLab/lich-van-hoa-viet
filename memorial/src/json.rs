//! JSON tối giản, không phụ thuộc ngoài: đọc tài liệu sao lưu / bộ dữ liệu kiểm thử (01 §5) và ghi
//! các dòng dump. Chỉ số nguyên (lược đồ v1 không có số thực); thứ tự khóa của đối tượng được giữ.
//! Giới hạn độ sâu 32 (01 §6.3).

use std::fmt::Write as _;

pub use lvhv_core::dayinfo::write_json_str;

/// Độ sâu lồng tối đa (01 §6.3).
pub const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<Value>),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Obj(v) => v.iter().find(|(k, _)| k == key).map(|(_, x)| x),
            _ => None,
        }
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn as_arr(&self) -> Option<&[Value]> {
        match self {
            Value::Arr(a) => Some(a),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Ghi gọn (không khoảng trắng), chuỗi theo `write_json_str` của lõi.
    pub fn write(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Int(i) => {
                let _ = write!(out, "{i}");
            }
            Value::Str(s) => write_json_str(out, s),
            Value::Arr(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    v.write(out);
                }
                out.push(']');
            }
            Value::Obj(o) => {
                out.push('{');
                for (i, (k, v)) in o.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    write_json_str(out, k);
                    out.push(':');
                    v.write(out);
                }
                out.push('}');
            }
        }
    }

    /// Ghi thụt lề 2 dấu cách (dạng của `schema/example-backup-v1.json`).
    pub fn write_pretty(&self, out: &mut String, indent: usize) {
        let pad = |out: &mut String, n: usize| {
            for _ in 0..n {
                out.push_str("  ");
            }
        };
        match self {
            Value::Arr(a) if !a.is_empty() => {
                out.push_str("[\n");
                for (i, v) in a.iter().enumerate() {
                    pad(out, indent + 1);
                    v.write_pretty(out, indent + 1);
                    if i + 1 < a.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                pad(out, indent);
                out.push(']');
            }
            Value::Obj(o) if !o.is_empty() => {
                out.push_str("{\n");
                for (i, (k, v)) in o.iter().enumerate() {
                    pad(out, indent + 1);
                    write_json_str(out, k);
                    out.push_str(": ");
                    v.write_pretty(out, indent + 1);
                    if i + 1 < o.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                pad(out, indent);
                out.push('}');
            }
            v => v.write(out),
        }
    }
}

/// Đối tượng JSON theo thứ tự khóa cho trước.
pub fn obj(fields: Vec<(&str, Value)>) -> Value {
    Value::Obj(
        fields
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    )
}

pub fn s(x: &str) -> Value {
    Value::Str(x.to_string())
}

pub fn opt_int(x: Option<i64>) -> Value {
    x.map_or(Value::Null, Value::Int)
}

/// Lỗi phân tích (vị trí byte).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub pos: usize,
    pub msg: &'static str,
}

pub fn parse(text: &str) -> Result<Value, ParseError> {
    let mut p = Parser {
        b: text.as_bytes(),
        i: 0,
    };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i != p.b.len() {
        return Err(p.err("dữ liệu thừa sau giá trị JSON"));
    }
    Ok(v)
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn err(&self, msg: &'static str) -> ParseError {
        ParseError { pos: self.i, msg }
    }
    fn ws(&mut self) {
        while self.i < self.b.len() && matches!(self.b[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }
    fn lit(&mut self, w: &[u8], v: Value) -> Result<Value, ParseError> {
        if self.b[self.i..].starts_with(w) {
            self.i += w.len();
            Ok(v)
        } else {
            Err(self.err("giá trị không hợp lệ"))
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, ParseError> {
        if depth >= MAX_DEPTH {
            return Err(self.err("lồng quá sâu"));
        }
        match self.b.get(self.i) {
            None => Err(self.err("hết dữ liệu")),
            Some(b'n') => self.lit(b"null", Value::Null),
            Some(b't') => self.lit(b"true", Value::Bool(true)),
            Some(b'f') => self.lit(b"false", Value::Bool(false)),
            Some(b'"') => Ok(Value::Str(self.string()?)),
            Some(b'[') => {
                self.i += 1;
                let mut a = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Value::Arr(a));
                }
                loop {
                    self.ws();
                    a.push(self.value(depth + 1)?);
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Value::Arr(a));
                        }
                        _ => return Err(self.err("thiếu , hoặc ]")),
                    }
                }
            }
            Some(b'{') => {
                self.i += 1;
                let mut o: Vec<(String, Value)> = Vec::new();
                self.ws();
                if self.b.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Value::Obj(o));
                }
                loop {
                    self.ws();
                    if self.b.get(self.i) != Some(&b'"') {
                        return Err(self.err("thiếu khóa"));
                    }
                    let k = self.string()?;
                    if o.iter().any(|(x, _)| *x == k) {
                        return Err(self.err("khóa trùng"));
                    }
                    self.ws();
                    if self.b.get(self.i) != Some(&b':') {
                        return Err(self.err("thiếu :"));
                    }
                    self.i += 1;
                    self.ws();
                    let v = self.value(depth + 1)?;
                    o.push((k, v));
                    self.ws();
                    match self.b.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Value::Obj(o));
                        }
                        _ => return Err(self.err("thiếu , hoặc }")),
                    }
                }
            }
            Some(c) if *c == b'-' || c.is_ascii_digit() => self.number(),
            _ => Err(self.err("ký tự không mong đợi")),
        }
    }
    fn number(&mut self) -> Result<Value, ParseError> {
        let st = self.i;
        if self.b[self.i] == b'-' {
            self.i += 1;
        }
        let ds = self.i;
        while self.i < self.b.len() && self.b[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if self.i == ds || (self.b[ds] == b'0' && self.i - ds > 1) {
            return Err(self.err("số không hợp lệ"));
        }
        if matches!(self.b.get(self.i), Some(b'.' | b'e' | b'E')) {
            return Err(self.err("chỉ chấp nhận số nguyên"));
        }
        std::str::from_utf8(&self.b[st..self.i])
            .ok()
            .and_then(|t| t.parse::<i64>().ok())
            .map(Value::Int)
            .ok_or(self.err("số ngoài phạm vi"))
    }
    fn hex4(&mut self) -> Result<u32, ParseError> {
        let h = self
            .b
            .get(self.i..self.i + 4)
            .and_then(|x| std::str::from_utf8(x).ok())
            .and_then(|x| u32::from_str_radix(x, 16).ok())
            .ok_or(self.err("\\u không hợp lệ"))?;
        self.i += 4;
        Ok(h)
    }
    fn string(&mut self) -> Result<String, ParseError> {
        self.i += 1; // "
        let mut out = String::new();
        loop {
            let st = self.i;
            while self.i < self.b.len() && self.b[self.i] != b'"' && self.b[self.i] != b'\\' {
                if self.b[self.i] < 0x20 {
                    return Err(self.err("ký tự điều khiển trong chuỗi"));
                }
                self.i += 1;
            }
            out.push_str(
                std::str::from_utf8(&self.b[st..self.i]).map_err(|_| self.err("UTF-8 hỏng"))?,
            );
            match self.b.get(self.i) {
                None => return Err(self.err("chuỗi chưa đóng")),
                Some(b'"') => {
                    self.i += 1;
                    return Ok(out);
                }
                _ => {
                    self.i += 1;
                    let c = *self.b.get(self.i).ok_or(self.err("chuỗi chưa đóng"))?;
                    self.i += 1;
                    match c {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let mut cp = self.hex4()?;
                            if (0xD800..0xDC00).contains(&cp) {
                                if self.b.get(self.i..self.i + 2) != Some(b"\\u") {
                                    return Err(self.err("cặp thay thế thiếu nửa sau"));
                                }
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(self.err("cặp thay thế hỏng"));
                                }
                                cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                            }
                            out.push(char::from_u32(cp).ok_or(self.err("điểm mã không hợp lệ"))?);
                        }
                        _ => return Err(self.err("thoát chuỗi không hợp lệ")),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let t = r#"{"a":[1,-2,null,true,false,"x\"\\\nü"],"b":{},"c":[]}"#;
        let v = parse(t).unwrap();
        let mut o = String::new();
        v.write(&mut o);
        assert_eq!(o, t);
    }

    #[test]
    fn rejects() {
        for bad in [
            "",
            "{",
            "[1,]",
            "01",
            "1.5",
            "{\"a\":1,\"a\":2}",
            "\"\u{1}\"",
            "[1] x",
        ] {
            assert!(parse(bad).is_err(), "{bad}");
        }
        let deep = "[".repeat(40) + &"]".repeat(40);
        assert!(parse(&deep).is_err());
        assert_eq!(
            parse("\"\\ud83d\\ude00\"").unwrap(),
            Value::Str("😀".into())
        );
    }
}
