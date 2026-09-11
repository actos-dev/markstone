use markstone_core::ast::{AstDocument, Node};

/// Pattern matching Actos username constraints (mirrors `ck_actors_username_format`).
pub const USERNAME_PATTERN: &str = r"^[a-z0-9_]{3,32}$";

/// Pattern matching Actos tag constraints (mirrors `ck_tags_name_format`).
pub const TAG_PATTERN: &str = r"^[a-z0-9][a-z0-9-]{0,31}$";

/// Validates whether a candidate string satisfies the Actos username constraints:
/// - Length between 3 and 32 characters inclusive
/// - Permitted characters: ASCII lowercase letters, ASCII digits, underscore (`[a-z0-9_]`)
#[must_use]
pub fn is_valid_username(s: &str) -> bool {
    let len = s.len();
    if !(3..=32).contains(&len) {
        return false;
    }
    s.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

/// Validates whether a candidate string satisfies the Actos tag constraints:
/// - Length between 1 and 32 characters inclusive
/// - First character: ASCII lowercase letter or digit (`[a-z0-9]`)
/// - Remaining characters: ASCII lowercase letters, digits, or hyphens (`[a-z0-9-]`)
#[must_use]
pub fn is_valid_tag(s: &str) -> bool {
    let len = s.len();
    if !(1..=32).contains(&len) {
        return false;
    }
    let mut bytes = s.bytes();
    let first = bytes.next().unwrap();
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Splits a Text node's string value into a sequence of Text, Mention, and Tag nodes.
///
/// Matching rules:
/// - Candidate begins with `@` (mention) or `#` (tag).
/// - Preceding character must NOT be an ASCII letter, digit, or underscore (`[a-zA-Z0-9_]`).
/// - Trailing punctuation (`.`, `,`, `!`, `?`, etc.) stays outside the mention/tag.
/// - Candidates that do not match the exact format remain plain text.
#[must_use]
pub fn split_text_node(value: String, pos: [usize; 4]) -> Vec<Node> {
    if !value.contains('@') && !value.contains('#') {
        return vec![Node::Text { value, pos }];
    }

    let mut result = Vec::new();
    let bytes = value.as_bytes();
    let len = bytes.len();

    let mut cur_line = pos[0];
    let mut cur_col = pos[1];

    let mut text_start = 0;
    let mut text_start_line = cur_line;
    let mut text_start_col = cur_col;

    let mut prev_char: Option<char> = None;
    let mut prev_line = cur_line;
    let mut prev_col = cur_col;

    let mut i = 0;
    while i < len {
        let ch = value[i..].chars().next().unwrap();
        let ch_len = ch.len_utf8();

        if ch == '@' || ch == '#' {
            let is_mention = ch == '@';
            let preceding_forbidden =
                prev_char.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');

            if !preceding_forbidden {
                // Look ahead to extract candidate token
                let cand_start = i + 1;
                let mut cand_end = cand_start;
                for c in value[cand_start..].chars() {
                    if c.is_alphanumeric() || c == '_' || c == '-' {
                        cand_end += c.len_utf8();
                    } else {
                        break;
                    }
                }

                let cand = &value[cand_start..cand_end];
                let is_valid = if is_mention {
                    is_valid_username(cand)
                } else {
                    is_valid_tag(cand)
                };

                if is_valid {
                    // Emit preceding text if any
                    if i > text_start {
                        let text_val = value[text_start..i].to_string();
                        result.push(Node::Text {
                            value: text_val,
                            pos: [text_start_line, text_start_col, prev_line, prev_col],
                        });
                    }

                    let node_start_line = cur_line;
                    let node_start_col = cur_col;
                    let token_char_count = 1 + cand.chars().count();
                    let node_end_col = node_start_col + token_char_count - 1;
                    let node_pos = [
                        node_start_line,
                        node_start_col,
                        node_start_line,
                        node_end_col,
                    ];

                    if is_mention {
                        result.push(Node::Mention {
                            username: cand.to_string(),
                            text: format!("@{cand}"),
                            pos: node_pos,
                        });
                    } else {
                        result.push(Node::Tag {
                            name: cand.to_string(),
                            text: format!("#{cand}"),
                            pos: node_pos,
                        });
                    }

                    // Advance cursor past the mention or tag
                    cur_col += token_char_count;
                    i = cand_end;
                    text_start = i;
                    text_start_line = cur_line;
                    text_start_col = cur_col;
                    prev_char = cand.chars().last();
                    prev_line = cur_line;
                    prev_col = node_end_col;
                    continue;
                }
            }
        }

        prev_line = cur_line;
        prev_col = cur_col;
        if ch == '\n' {
            cur_line += 1;
            cur_col = 1;
        } else {
            cur_col += 1;
        }
        prev_char = Some(ch);
        i += ch_len;
    }

    if text_start < len {
        let text_val = value[text_start..].to_string();
        result.push(Node::Text {
            value: text_val,
            pos: [text_start_line, text_start_col, pos[2], pos[3]],
        });
    }

    if result.is_empty() {
        vec![Node::Text { value, pos }]
    } else {
        result
    }
}

struct TransformFrame {
    node: Node,
    in_link: bool,
    old_children: Vec<Node>,
    new_children: Vec<Node>,
    next_idx: usize,
}

/// Transforms an AST node and all its descendants strictly non-recursively.
///
/// Mentions and tags are extracted from Text nodes, except inside link text
/// (descendants of `Node::Link`), code blocks, code spans, and non-text constructs.
pub fn transform_node(mut root: Node) -> Node {
    let old_children = match root.children_mut() {
        Some(c) => std::mem::take(c),
        None => return root,
    };

    let in_link = matches!(root, Node::Link { .. });
    let mut stack = vec![TransformFrame {
        in_link,
        new_children: Vec::with_capacity(old_children.len()),
        old_children,
        node: root,
        next_idx: 0,
    }];

    while let Some(frame) = stack.last_mut() {
        if frame.next_idx < frame.old_children.len() {
            let mut child = std::mem::replace(
                &mut frame.old_children[frame.next_idx],
                Node::ThematicBreak { pos: [0, 0, 0, 0] },
            );
            frame.next_idx += 1;

            if child.children_mut().is_some() {
                let child_in_link = frame.in_link || matches!(child, Node::Link { .. });
                let child_old_children = std::mem::take(child.children_mut().unwrap());
                stack.push(TransformFrame {
                    node: child,
                    in_link: child_in_link,
                    new_children: Vec::with_capacity(child_old_children.len()),
                    old_children: child_old_children,
                    next_idx: 0,
                });
            } else {
                if !frame.in_link {
                    if let Node::Text { ref mut value, pos } = child {
                        let text_val = std::mem::take(value);
                        let spliced = split_text_node(text_val, pos);
                        frame.new_children.extend(spliced);
                        continue;
                    }
                }
                frame.new_children.push(child);
            }
        } else {
            let mut finished_frame = stack.pop().unwrap();
            *finished_frame.node.children_mut().unwrap() = finished_frame.new_children;
            if let Some(parent) = stack.last_mut() {
                parent.new_children.push(finished_frame.node);
            } else {
                return finished_frame.node;
            }
        }
    }

    unreachable!("stack should terminate at root pop")
}

/// Applies the Actos mention and tag pass over the AST document in-place,
/// strictly without recursion.
pub fn transform_ast(doc: &mut AstDocument) {
    let dummy = Node::Document {
        pos: [0, 0, 0, 0],
        children: Vec::new(),
    };
    let root = std::mem::replace(&mut doc.root, dummy);
    doc.root = transform_node(root);
}
