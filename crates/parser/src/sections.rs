/// Separador de seções usado pelo jogo.
const SEPARATOR: &str = "--------";

/// Uma seção = linhas não vazias entre dois separadores.
pub type Section<'a> = Vec<&'a str>;

/// Divide o texto do item em seções, normalizando `\r\n` e descartando linhas vazias.
pub fn split(text: &str) -> Vec<Section<'_>> {
    let mut sections = vec![Vec::new()];
    for line in text.lines().map(str::trim_end) {
        if line == SEPARATOR {
            sections.push(Vec::new());
        } else if !line.is_empty() {
            if let Some(current) = sections.last_mut() {
                current.push(line);
            }
        }
    }
    sections.retain(|s| !s.is_empty());
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_separator_and_crlf() {
        let s = split("a\r\nb\r\n--------\r\nc\r\n--------\r\n");
        assert_eq!(s, vec![vec!["a", "b"], vec!["c"]]);
    }
}
