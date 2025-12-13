const fn is_valid_base_char(c: char) -> bool {
    matches!(c, '0'..='9' | 'a'..='z' | '_' | '-' | '.')
}

pub const fn is_valid_namespace_char(c: char) -> bool {
    is_valid_base_char(c) || c == '#'
}

pub const fn is_valid_path_char(c: char) -> bool {
    is_valid_base_char(c) || c == '/'
}
