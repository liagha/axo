use crate::internal::hash::{Hash, Map, Set};

pub trait Encode {
    fn encode(&self, buffer: &mut Vec<u8>);
}

pub trait Decode<'element>: Sized {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self;
}

impl Encode for u8 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(*self);
    }
}

impl<'element> Decode<'element> for u8 {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let value = buffer[*cursor];
        *cursor += 1;
        value
    }
}

impl Encode for bool {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(if *self { 1 } else { 0 });
    }
}

impl<'element> Decode<'element> for bool {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let value = buffer[*cursor] == 1;
        *cursor += 1;
        value
    }
}

impl Encode for u64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl<'element> Decode<'element> for u64 {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let mut bytes = [0; 8];
        bytes.copy_from_slice(&buffer[*cursor..*cursor + 8]);
        *cursor += 8;
        u64::from_le_bytes(bytes)
    }
}

impl Encode for usize {
    fn encode(&self, buffer: &mut Vec<u8>) {
        (*self as u64).encode(buffer);
    }
}

impl<'element> Decode<'element> for usize {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        u64::decode(buffer, cursor) as usize
    }
}

impl<Target: Encode> Encode for Option<Target> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        match self {
            Some(value) => {
                buffer.push(1);
                value.encode(buffer);
            }
            None => buffer.push(0),
        }
    }
}

impl<'element, Target: Decode<'element>> Decode<'element> for Option<Target> {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let tag = buffer[*cursor];
        *cursor += 1;
        match tag {
            1 => Some(Target::decode(buffer, cursor)),
            _ => None,
        }
    }
}

impl<Target: Encode> Encode for Vec<Target> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);
        for item in self {
            item.encode(buffer);
        }
    }
}

impl<'element, Target: Decode<'element>> Decode<'element> for Vec<Target> {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self {
        let length = usize::decode(buffer, cursor);
        let mut items = Vec::with_capacity(length);
        for _ in 0..length {
            items.push(Target::decode(buffer, cursor));
        }
        items
    }
}

impl Encode for f64 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl<'a> Decode<'a> for f64 {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&buffer[*cursor..*cursor + 8]);
        *cursor += 8;
        f64::from_le_bytes(bytes)
    }
}

impl Encode for char {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&(*self as u32).to_le_bytes());
    }
}

impl<'a> Decode<'a> for char {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&buffer[*cursor..*cursor + 4]);
        *cursor += 4;
        char::from_u32(u32::from_le_bytes(bytes)).unwrap()
    }
}

impl<T: Encode> Encode for Box<T> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.as_ref().encode(buffer);
    }
}

impl<'a, T: Decode<'a>> Decode<'a> for Box<T> {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        Box::new(T::decode(buffer, cursor))
    }
}

impl Encode for i128 {
    fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(&self.to_le_bytes());
    }
}

impl<'a> Decode<'a> for i128 {
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&buffer[*cursor..*cursor + 16]);
        *cursor += 16;
        i128::from_le_bytes(bytes)
    }
}

impl<K: Encode, V: Encode> Encode for Map<K, V> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);
        for (key, value) in self.iter() {
            key.encode(buffer);
            value.encode(buffer);
        }
    }
}

impl<'a, K, V> Decode<'a> for Map<K, V>
where
    K: Decode<'a> + Eq + Hash,
    V: Decode<'a>,
{
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let len = usize::decode(buffer, cursor);
        let mut map = Map::with_capacity(len);
        for _ in 0..len {
            let key = K::decode(buffer, cursor);
            let value = V::decode(buffer, cursor);
            map.insert(key, value);
        }
        map
    }
}

impl<T: Encode> Encode for Set<T> {
    fn encode(&self, buffer: &mut Vec<u8>) {
        self.len().encode(buffer);
        for item in self.iter() {
            item.encode(buffer);
        }
    }
}

impl<'a, T> Decode<'a> for Set<T>
where
    T: Decode<'a> + Eq + Hash,
{
    fn decode(buffer: &'a [u8], cursor: &mut usize) -> Self {
        let len = usize::decode(buffer, cursor);
        let mut set = Set::with_capacity(len);
        for _ in 0..len {
            set.insert(T::decode(buffer, cursor));
        }
        set
    }
}
