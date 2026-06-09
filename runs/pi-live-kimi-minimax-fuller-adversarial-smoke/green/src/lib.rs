pub fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut prev_sep = true;

    for ch in input.chars() {
        let token = match ch {
            'a'..='z' | '0'..='9' => {
                if prev_sep && !out.is_empty() {
                    out.push('-');
                }
                out.push(ch);
                prev_sep = false;
                continue;
            }
            'A'..='Z' => {
                if prev_sep && !out.is_empty() {
                    out.push('-');
                }
                out.push(ch.to_ascii_lowercase());
                prev_sep = false;
                continue;
            }
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' |
            'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' => "a",
            'ç' | 'ć' | 'Ç' | 'Ć' => "c",
            'è' | 'é' | 'ê' | 'ë' | 'ē' |
            'È' | 'É' | 'Ê' | 'Ë' | 'Ē' => "e",
            'ì' | 'í' | 'î' | 'ï' | 'ī' |
            'Ì' | 'Í' | 'Î' | 'Ï' | 'Ī' => "i",
            'ñ' | 'Ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' |
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' => "o",
            'ù' | 'ú' | 'û' | 'ü' | 'ū' |
            'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ū' => "u",
            'ý' | 'ÿ' | 'Ý' | 'Ÿ' => "y",
            'æ' | 'Æ' => "ae",
            'œ' | 'Œ' => "oe",
            'ß' => "ss",
            _ => {
                prev_sep = true;
                continue;
            }
        };

        if prev_sep && !out.is_empty() {
            out.push('-');
        }
        out.push_str(token);
        prev_sep = false;
    }

    out
}
