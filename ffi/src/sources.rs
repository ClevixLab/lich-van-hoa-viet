//! Danh mục nguồn (Markdown) → khối văn bản đơn giản `[{"kind":"h"|"p"|"li","text":..}]`, để
//! mọi nền tảng hiển thị giống nhau mà không cần bộ dựng Markdown. Liên kết `[chữ](url)` thành
//! `chữ (url)`; bỏ `*`, `_`, `` ` ``. Bảng: bỏ dòng tiêu đề và dòng `---`, mỗi hàng thành một
//! mục `li` với các ô nối bằng " — ".

use serde::Serialize;

/// content/ngochap/SOURCES.md — nguồn quy tắc xem ngày.
pub const NGOCHAP_SOURCES_MD: &str = include_str!("../../content/ngochap/SOURCES.md");

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct Block {
    pub kind: &'static str,
    pub text: String,
}

fn clean_inline(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(i) = rest.find('[') {
        out.push_str(&rest[..i]);
        let after = &rest[i + 1..];
        match (after.find("]("), after.find(']')) {
            (Some(j), Some(k)) if j == k => {
                let label = &after[..j];
                let tail = &after[j + 2..];
                if let Some(e) = tail.find(')') {
                    let url = &tail[..e];
                    out.push_str(label);
                    if !url.is_empty() && url != label {
                        out.push_str(" (");
                        out.push_str(url);
                        out.push(')');
                    }
                    rest = &tail[e + 1..];
                    continue;
                }
                out.push('[');
                rest = after;
            }
            _ => {
                out.push('[');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out.replace(['*', '`'], "")
        .replace("\\|", "|")
        .trim()
        .to_string()
}

fn is_separator_row(t: &str) -> bool {
    t.starts_with('|') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

/// Chuyển Markdown thành khối.
pub fn md_to_blocks(md: &str) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para = String::new();
    let mut in_table = false;
    let mut header_skipped = false;
    let flush = |para: &mut String, out: &mut Vec<Block>| {
        if !para.trim().is_empty() {
            out.push(Block {
                kind: "p",
                text: clean_inline(para),
            });
        }
        para.clear();
    };
    for line in md.lines() {
        let t = line.trim();
        if t.starts_with('|') {
            flush(&mut para, &mut out);
            if !in_table {
                in_table = true;
                header_skipped = false;
            }
            if is_separator_row(t) {
                continue;
            }
            if !header_skipped {
                header_skipped = true;
                continue;
            }
            // Tách ô, giữ "\|" trong ô.
            let body = t.trim_matches('|').replace("\\|", "\u{0}");
            let cells: Vec<String> = body
                .split('|')
                .map(|c| clean_inline(&c.replace('\u{0}', "|")))
                .filter(|c| !c.is_empty())
                .collect();
            if !cells.is_empty() {
                out.push(Block {
                    kind: "li",
                    text: cells.join(" — "),
                });
            }
            continue;
        }
        in_table = false;
        if t.is_empty() {
            flush(&mut para, &mut out);
            continue;
        }
        if let Some(h) = t.strip_prefix('#') {
            flush(&mut para, &mut out);
            out.push(Block {
                kind: "h",
                text: clean_inline(h.trim_start_matches('#')),
            });
            continue;
        }
        if let Some(li) = t.strip_prefix("- ").or_else(|| t.strip_prefix("* ")) {
            flush(&mut para, &mut out);
            out.push(Block {
                kind: "li",
                text: clean_inline(li),
            });
            continue;
        }
        let t = t.strip_prefix('>').map(str::trim).unwrap_or(t);
        if !para.is_empty() {
            para.push(' ');
        }
        para.push_str(t);
    }
    flush(&mut para, &mut out);
    out
}

pub fn blocks_json(md: &str) -> String {
    serde_json::to_string(&md_to_blocks(md)).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_tables_lists() {
        let md = "# Tiêu đề\n\nĐoạn *một*\ndòng hai [Wiki](https://w.org).\n\n| A | B |\n|---|---|\n| x | y \\| z |\n\n- mục `1`\n";
        let b = md_to_blocks(md);
        assert_eq!(
            b[0],
            Block {
                kind: "h",
                text: "Tiêu đề".into()
            }
        );
        assert_eq!(b[1].text, "Đoạn một dòng hai Wiki (https://w.org).");
        assert_eq!(
            b[2],
            Block {
                kind: "li",
                text: "x — y | z".into()
            }
        );
        assert_eq!(
            b[3],
            Block {
                kind: "li",
                text: "mục 1".into()
            }
        );
        assert_eq!(b.len(), 4);
    }

    #[test]
    fn real_files_parse() {
        assert!(md_to_blocks(NGOCHAP_SOURCES_MD).len() > 10);
        assert!(md_to_blocks(crate::cadao::SOURCES_MD).len() > 10);
    }
}
