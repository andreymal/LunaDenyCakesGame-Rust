//! Всякие полезные штуки.

/// Достаёт первое слово из строки.
///
/// Возвращает три элемента:
///
/// * срез строки с первым словом;
/// * идущий после слова разделитель (пробельный символ или конец строки)
/// * срез всей остальной строки после слова и разделителя.
///
/// Вызывая эту функцию в цикле, можно получить все слова — чем и занимается
/// функция [iter_words](self::iter_words).
///
/// # Examples
///
/// ```
/// # use cake_engine::utils::split_word;
/// let text = "Вася Пупкин";
/// let w1 = split_word(text);
/// assert_eq!(w1, Some(("Вася", Some(' '), "Пупкин")));
/// let w2 = split_word(w1.unwrap().2);
/// assert_eq!(w2, Some(("Пупкин", None, "")));
/// let w3 = split_word(w2.unwrap().2);
/// assert_eq!(w3, None);
/// ```
pub fn split_word(text: &str) -> Option<(&str, Option<char>, &str)> {
    let mut chars_iter = text.char_indices();
    if let Some((idx, sep)) = chars_iter.find(|(_, c)| c.is_whitespace()) {
        Some((&text[..idx], Some(sep), chars_iter.as_str()))
    } else if !text.is_empty() {
        Some((text, None, ""))
    } else {
        None
    }
}

pub struct WordIterator<'a> {
    text: &'a str,
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = (&'a str, Option<char>);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((word, sep_or_end, remainder)) = split_word(self.text) {
            self.text = remainder;
            Some((word, sep_or_end))
        } else {
            None
        }
    }
}

/// Делит текст на слова.
///
/// # Examples
///
/// ```
/// # use cake_engine::utils::iter_words;
/// let mut three_letter_words = vec![];
///
/// let text = "В чащах юга жил бы цитрус? Да, но фальшивый экземпляр!";
///
/// for (word, _sep_or_none) in iter_words(text) {
///     if word.chars().count() == 3 {
///         three_letter_words.push(word);
///     }
/// }
///
/// assert_eq!(three_letter_words, vec!["юга", "жил", "Да,"]);
/// ```
pub fn iter_words<'a>(text: &'a str) -> WordIterator<'a> {
    WordIterator { text }
}

pub struct WrapIterator<'a, 'b> {
    text: &'a str,
    get_line_width: &'b dyn Fn(&str) -> f32,
    max_width: f32,
    last_sep: Option<char>,
}

impl Iterator for WrapIterator<'_, '_> {
    type Item = (f32, String);

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        let mut line_width = 0.0;

        while let Some((word, sep_or_end, remainder)) = split_word(self.text) {
            let prev_len = line.len();

            // Добавляем новое слово (не забываем добавить пробельный символ между словами)
            let is_single_word = line.is_empty();
            if !is_single_word {
                if let Some(c) = self.last_sep {
                    if c != '\r' {
                        line.push(c);
                    }
                }
            }
            line.push_str(word);
            let new_line_width = (self.get_line_width)(&line);

            if new_line_width <= self.max_width {
                // Слово влезло — ничего не делаем
                line_width = new_line_width;
            } else {
                // Слово не влезло
                if !is_single_word {
                    // Если в строке были другие слова — убираем новое слово и возвращаем что влезло
                    line.truncate(prev_len);
                    return Some((line_width, line));
                }
                // Если даже одно слово не влезает, оставляем его как есть
                // (можно разбить посимвольно, но мне лень)
                line_width = new_line_width;
            }

            self.text = remainder;
            self.last_sep = sep_or_end;

            // Перенос строки — возвращаем строку немедленно (даже пустую)
            if let Some('\n') = sep_or_end {
                return Some((line_width, line));
            }
        }

        if !line.is_empty() {
            // Текст закончился — возвращаем влезшие слова последней строки
            Some((line_width, line))
        } else if let Some('\n') = self.last_sep {
            // Текст закончился переносом строки — добавляем последнюю пустую строку
            self.last_sep = None;
            Some((line_width, line))
        } else {
            None
        }
    }
}

/// Делит текст на строки, не превышающие указанную длину.
///
/// Возвращает итератор, выдающий по два элемента: длина строки (в тех единицах, которые
/// использует указанная вами функция `get_line_width`), и сама строка в виде объекта
/// [`String`](std::string::String).
///
/// # Examples
///
/// ```
/// # use cake_engine::utils::wrap_text;
/// let text = "В чащах юга жил бы цитрус? Да, но фальшивый экземпляр!";
///
/// let lines: Vec<(f32, String)> = wrap_text(
///     text,
///     // Для примера допустим, что длина одного символа равна 1
///     &|s| s.chars().count() as f32,
///     20.0,
/// ).collect();
///
/// assert_eq!(
///     lines,
///     vec![
///         (18.0, "В чащах юга жил бы".to_string()),
///         (14.0, "цитрус? Да, но".to_string()),
///         (20.0, "фальшивый экземпляр!".to_string()),
///     ],
/// );
/// ```
pub fn wrap_text<'a, 'b>(
    text: &'a str,
    get_line_width: &'b dyn Fn(&str) -> f32,
    max_width: f32,
) -> WrapIterator<'a, 'b> {
    WrapIterator {
        text,
        get_line_width,
        max_width,
        last_sep: None,
    }
}
