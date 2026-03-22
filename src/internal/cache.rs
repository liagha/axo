pub trait Encode {
    fn encode(&self, buffer: &mut Vec<u8>);
}

pub trait Decode<'element>: Sized {
    fn decode(buffer: &'element [u8], cursor: &mut usize) -> Self;
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