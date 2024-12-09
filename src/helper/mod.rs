/// Prefix adds each prefix in order, checking for empty prefix.
/// result is constructed using snake_case
///
pub(crate) fn prefix(prefixes: &[&str]) -> String {
    let mut e: String = "".into();

    for prefix in prefixes {
        if e.is_empty() {
            e = prefix.to_string();
        } else if !prefix.is_empty() {
            e = snake_case(format!("{}_{}", prefix, e));
        }
    }

    e
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
pub(crate) fn snake_case(elt: String) -> String {
    let mut acc = String::new();
    let mut prev = '_';
    for ch in elt.chars() {
        if ch.is_uppercase() && prev != '_' {
            acc.push('_');
        }
        acc.push(ch);
        prev = ch;
    }

    acc.to_lowercase() // only modification
}

// taken straight from paste crate (https://github.com/dtolnay/paste/blob/6a302522990cbfd9de4e0c61d91854622f7b2999/src/segment.rs#L176)
pub(crate) fn camel_case(elt: String) -> String {
    let mut acc = String::new();
    let mut prev = '_';
    for ch in elt.chars() {
        if ch != '_' {
            if prev == '_' {
                for chu in ch.to_uppercase() {
                    acc.push(chu);
                }
            } else if prev.is_uppercase() {
                for chl in ch.to_lowercase() {
                    acc.push(chl);
                }
            } else {
                acc.push(ch);
            }
        }
        prev = ch;
    }

    acc
}
