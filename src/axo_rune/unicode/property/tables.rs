use crate::axo_rune::CharRange;
use crate::chars;

#[derive(Copy, Clone, Debug)]
pub enum CharDataTable<V: 'static> {
    #[doc(hidden)]
    Direct(&'static [(char, V)]),
    #[doc(hidden)]
    Range(&'static [(CharRange, V)]),
}

impl<V> Default for CharDataTable<V> {
    fn default() -> Self {
        CharDataTable::Direct(&[])
    }
}

impl<V> CharDataTable<V> {
    pub fn contains(&self, needle: char) -> bool {
        match *self {
            CharDataTable::Direct(table) => {
                table.binary_search_by_key(&needle, |&(k, _)| k).is_ok()
            }
            CharDataTable::Range(table) => table
                .binary_search_by(|&(range, _)| range.cmp_char(needle))
                .is_ok(),
        }
    }
}

impl<V: Copy> CharDataTable<V> {
    pub fn find(&self, needle: char) -> Option<V> {
        match *self {
            CharDataTable::Direct(table) => table
                .binary_search_by_key(&needle, |&(k, _)| k)
                .map(|idx| table[idx].1)
                .ok(),
            CharDataTable::Range(table) => table
                .binary_search_by(|&(range, _)| range.cmp_char(needle))
                .map(|idx| table[idx].1)
                .ok(),
        }
    }

    pub fn find_with_range(&self, needle: char) -> Option<(CharRange, V)> {
        match *self {
            CharDataTable::Direct(_) => None,
            CharDataTable::Range(table) => table
                .binary_search_by(|&(range, _)| range.cmp_char(needle))
                .map(|idx| table[idx])
                .ok(),
        }
    }
}

impl<V: Copy + Default> CharDataTable<V> {
    pub fn find_or_default(&self, needle: char) -> V {
        self.find(needle).unwrap_or_else(Default::default)
    }
}

#[derive(Debug)]
pub struct CharDataTableIter<'a, V: 'static>(&'a CharDataTable<V>, usize);

impl<'a, V: Copy> Iterator for CharDataTableIter<'a, V> {
    type Item = (CharRange, V);

    fn next(&mut self) -> Option<Self::Item> {
        match *self.0 {
            CharDataTable::Direct(arr) => {
                if self.1 >= arr.len() {
                    None
                } else {
                    let idx = self.1;
                    self.1 += 1;
                    let (ch, v) = arr[idx];
                    Some((chars!(ch..=ch), v))
                }
            }
            CharDataTable::Range(arr) => {
                if self.1 >= arr.len() {
                    None
                } else {
                    let idx = self.1;
                    self.1 += 1;
                    Some(arr[idx])
                }
            }
        }
    }
}

impl<V> CharDataTable<V> {
    pub fn iter(&self) -> CharDataTableIter<'_, V> {
        CharDataTableIter(self, 0)
    }
}
