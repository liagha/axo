use core::cmp::min;
use axo_hash::HashMap;

pub fn damerau_levenshtein_distance(s1: &str, s2: &str) -> usize {
    if s1 == s2 {
        return 0;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    let len_s1 = s1_chars.len();
    let len_s2 = s2_chars.len();

    if len_s1 == 0 {
        return len_s2;
    }
    if len_s2 == 0 {
        return len_s1;
    }

    let mut matrix = vec![vec![0; len_s2 + 1]; len_s1 + 1];

    for i in 0..=len_s1 {
        matrix[i][0] = i;
    }
    for j in 0..=len_s2 {
        matrix[0][j] = j;
    }

    for i in 1..=len_s1 {
        for j in 1..=len_s2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };

            matrix[i][j] = min(
                matrix[i - 1][j] + 1,
                min(
                    matrix[i][j - 1] + 1,
                    matrix[i - 1][j - 1] + cost
                )
            );

            if i > 1 && j > 1 && s1_chars[i - 1] == s2_chars[j - 2] && s1_chars[i - 2] == s2_chars[j - 1] {
                matrix[i][j] = min(
                    matrix[i][j],
                    matrix[i - 2][j - 2] + cost
                );
            }
        }
    }

    matrix[len_s1][len_s2]
}

pub fn create_qwerty_layout() -> HashMap<char, Vec<char>> {
    let mut layout = HashMap::new();

    let adjacency_map = [
        ('q', vec!['w', 'a']),
        ('w', vec!['q', 'e', 'a', 's']),
        ('e', vec!['w', 'r', 's', 'd']),
        ('r', vec!['e', 't', 'd', 'f']),
        ('t', vec!['r', 'y', 'f', 'g']),
        ('y', vec!['t', 'u', 'g', 'h']),
        ('u', vec!['y', 'i', 'h', 'j']),
        ('i', vec!['u', 'o', 'j', 'k']),
        ('o', vec!['i', 'p', 'k', 'l']),
        ('p', vec!['o', 'l']),
        ('a', vec!['q', 'w', 's', 'z']),
        ('s', vec!['w', 'e', 'a', 'd', 'z', 'x']),
        ('d', vec!['e', 'r', 's', 'f', 'x', 'c']),
        ('f', vec!['r', 't', 'd', 'g', 'c', 'v']),
        ('g', vec!['t', 'y', 'f', 'h', 'v', 'b']),
        ('h', vec!['y', 'u', 'g', 'j', 'b', 'n']),
        ('j', vec!['u', 'i', 'h', 'k', 'n', 'm']),
        ('k', vec!['i', 'o', 'j', 'l', 'm']),
        ('l', vec!['o', 'p', 'k']),
        ('z', vec!['a', 's', 'x']),
        ('x', vec!['z', 's', 'd', 'c']),
        ('c', vec!['x', 'd', 'f', 'v']),
        ('v', vec!['c', 'f', 'g', 'b']),
        ('b', vec!['v', 'g', 'h', 'n']),
        ('n', vec!['b', 'h', 'j', 'm']),
        ('m', vec!['n', 'j', 'k']),
        ('1', vec!['2', '`']),
        ('2', vec!['1', '3', 'q']),
        ('3', vec!['2', '4', 'w']),
        ('4', vec!['3', '5', 'e']),
        ('5', vec!['4', '6', 'r']),
        ('6', vec!['5', '7', 't']),
        ('7', vec!['6', '8', 'y']),
        ('8', vec!['7', '9', 'u']),
        ('9', vec!['8', '0', 'i']),
        ('0', vec!['9', '-', 'o']),
        ('-', vec!['0', '=', 'p']),
        ('=', vec!['-']),
    ];

    for (key, adjacent) in adjacency_map {
        layout.insert(key, adjacent);
    }

    layout
}

pub struct KeyboardLayout {
    pub layout: HashMap<char, Vec<char>>,
    pub name: String,
}

pub fn create_dvorak_layout() -> HashMap<char, Vec<char>> {
    let mut layout = HashMap::new();

    let adjacency_map = [
        ('\'', vec![',', 'a']),
        (',', vec!['\'', '.', 'a', 'o']),
        ('.', vec![',', 'p', 'o', 'e']),
        ('p', vec!['.', 'y', 'e', 'u']),
        ('y', vec!['p', 'f', 'u', 'i']),
        ('f', vec!['y', 'g', 'i', 'd']),
        ('g', vec!['f', 'c', 'd', 'h']),
        ('c', vec!['g', 'r', 'h', 't']),
        ('r', vec!['c', 'l', 't', 'n']),
        ('l', vec!['r', '/', 'n', 's']),
        ('/', vec!['l', '=', 's']),
        ('a', vec!['\'', ',', 'o', ';']),
        ('o', vec![',', '.', 'a', 'e', ';', 'q']),
        ('e', vec!['.', 'p', 'o', 'u', 'q', 'j']),
        ('u', vec!['p', 'y', 'e', 'i', 'j', 'k']),
        ('i', vec!['y', 'f', 'u', 'd', 'k', 'x']),
        ('d', vec!['f', 'g', 'i', 'h', 'x', 'b']),
        ('h', vec!['g', 'c', 'd', 't', 'b', 'm']),
        ('t', vec!['c', 'r', 'h', 'n', 'm', 'w']),
        ('n', vec!['r', 'l', 't', 's', 'w', 'v']),
        ('s', vec!['l', '/', 'n', '-', 'v', 'z']),
        (';', vec!['a', 'o', 'q']),
        ('q', vec!['o', 'e', ';', 'j']),
        ('j', vec!['e', 'u', 'q', 'k']),
        ('k', vec!['u', 'i', 'j', 'x']),
        ('x', vec!['i', 'd', 'k', 'b']),
        ('b', vec!['d', 'h', 'x', 'm']),
        ('m', vec!['h', 't', 'b', 'w']),
        ('w', vec!['t', 'n', 'm', 'v']),
        ('v', vec!['n', 's', 'w', 'z']),
        ('z', vec!['s', '-', 'v']),
        ('-', vec!['s', 'z']),
    ];

    for (key, adjacent) in adjacency_map {
        layout.insert(key, adjacent);
    }

    layout
}

pub enum KeyboardLayoutType {
    Qwerty,
    Dvorak,
    Custom(HashMap<char, Vec<char>>),
}

impl KeyboardLayoutType {
    pub fn get_layout(&self) -> HashMap<char, Vec<char>> {
        match self {
            KeyboardLayoutType::Qwerty => create_qwerty_layout(),
            KeyboardLayoutType::Dvorak => create_dvorak_layout(),
            KeyboardLayoutType::Custom(layout) => layout.clone(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            KeyboardLayoutType::Qwerty => "QWERTY",
            KeyboardLayoutType::Dvorak => "Dvorak",
            KeyboardLayoutType::Custom(_) => "Custom",
        }
    }
}