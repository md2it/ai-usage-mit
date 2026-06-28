pub fn clean_terminal_output(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut cleaned = String::new();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            0x1b => {
                index += 1;
                if index >= bytes.len() {
                    break;
                }

                match bytes[index] {
                    b'[' => {
                        index += 1;
                        while index < bytes.len() && !bytes[index].is_ascii_alphabetic() {
                            index += 1;
                        }
                        index += 1;
                    }
                    b']' => {
                        index += 1;
                        while index < bytes.len() {
                            if bytes[index] == 0x07 {
                                index += 1;
                                break;
                            }
                            if bytes[index] == b'\\'
                                && index > 0
                                && bytes[index.saturating_sub(1)] == 0x1b
                            {
                                index += 1;
                                break;
                            }
                            index += 1;
                        }
                    }
                    _ => {
                        index += 1;
                    }
                }
            }
            b'\r' | b'\n' | b'\t' => {
                cleaned.push(bytes[index] as char);
                index += 1;
            }
            byte if byte.is_ascii_control() => {
                index += 1;
            }
            _ => {
                let rest = &input[index..];
                if let Some(character) = rest.chars().next() {
                    cleaned.push(character);
                    index += character.len_utf8();
                } else {
                    break;
                }
            }
        }
    }

    cleaned
}

pub fn compact_terminal_text(input: &str) -> String {
    let mut output = String::new();
    let mut pending_word = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.chars().count() == 1 {
            pending_word.push_str(trimmed);
            continue;
        }

        flush_pending_word(&mut output, &mut pending_word);

        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(trimmed);
    }

    flush_pending_word(&mut output, &mut pending_word);

    if !output.is_empty() {
        output.push('\n');
    }

    output
}

fn flush_pending_word(output: &mut String, pending_word: &mut String) {
    if pending_word.is_empty() {
        return;
    }

    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(pending_word);
    pending_word.clear();
}
