use crate::configuration::config::ConfigError;
use std::borrow::Cow;

/// Expands environment placeholders in the given text.
///
/// Replaces occurrences of `${ENV_VAR}` with the corresponding value from the
/// process environment (via `std::env`). If a referenced variable is not set,
/// returns `ConfigError::MissingEnvVar`.
///
/// The sequence `\${` is treated as an escape and is converted to a literal `${`
/// without performing substitution.
///
/// On malformed placeholders (e.g. missing closing }, empty/invalid variable
/// name, or a newline inside the placeholder), returns
/// `ConfigError::InvalidEnvPlaceholder { line, char }`, where line and char
/// are 1-based positions of the $ that started the placeholder.
///
/// For efficiency, if the input contains no `${`, the function
/// returns a `borrowed Cow::Borrowed` without allocating
pub fn expand_env_placeholders<'a>(input: &'a str) -> Result<Cow<'a, str>, ConfigError> {
    if !input.contains("${") {
        return Ok(Cow::Borrowed(input));
    }

    fn bump(ch: char, line: &mut usize, col: &mut usize) {
        if ch == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
    }

    fn is_valid_env_name(name: &str) -> bool {
        let mut it = name.chars();
        let Some(first) = it.next() else {
            return false;
        };
        if !(first == '_' || first.is_ascii_alphabetic()) {
            return false;
        }
        it.all(|c| c == '_' || c.is_ascii_alphanumeric())
    }

    let mut out = String::with_capacity(input.len());
    let mut it = input.char_indices().peekable();

    let mut line: usize = 1;
    let mut col: usize = 1;

    while let Some((_i, ch)) = it.next() {
        let start_line = line;
        let start_col = col;

        match ch {
            '\\' => {
                let mut look = it.clone();
                if matches!(look.next(), Some((_, '$'))) && matches!(look.next(), Some((_, '{'))) {
                    // съедаем '$' и '{'
                    let (_, d) = it.next().unwrap();
                    let (_j, b) = it.next().unwrap();

                    // учёт позиции
                    bump('\\', &mut line, &mut col);
                    bump(d, &mut line, &mut col);
                    bump(b, &mut line, &mut col);

                    out.push_str("${");
                    continue;
                }

                bump('\\', &mut line, &mut col);
                out.push('\\');
            }
            '$' => {
                if !matches!(it.peek(), Some((_, '{'))) {
                    bump('$', &mut line, &mut col);
                    out.push('$');
                    continue;
                }
                let (brace_idx, brace) = it.next().unwrap();

                bump('$', &mut line, &mut col);
                bump(brace, &mut line, &mut col);
                let name_start = brace_idx + brace.len_utf8();
                let mut name_end: Option<usize> = None;

                while let Some((k, c)) = it.next() {
                    match c {
                        '}' => {
                            name_end = Some(k);
                            bump('}', &mut line, &mut col);
                            break;
                        }
                        '\n' => {
                            return Err(ConfigError::InvalidEnvPlaceholder {
                                line: start_line,
                                char: start_col,
                            });
                        }
                        _ => bump(c, &mut line, &mut col),
                    }
                }

                let name_end = name_end.ok_or(ConfigError::InvalidEnvPlaceholder {
                    line: start_line,
                    char: start_col,
                })?;

                let name = &input[name_start..name_end];

                if name.is_empty() || !is_valid_env_name(name) {
                    return Err(ConfigError::InvalidEnvPlaceholder {
                        line: start_line,
                        char: start_col,
                    });
                }

                let Some(val) = std::env::var_os(name) else {
                    return Err(ConfigError::MissingEnvVar(name.to_string()));
                };

                out.push_str(&val.to_string_lossy());
            }

            _ => {
                bump(ch, &mut line, &mut col);
                out.push(ch);
            }
        }
    }

    Ok(Cow::Owned(out))
}
