use std::slice::Iter;

use kofile::errors::{ConstantReadError, ReadError, ReadResult};

pub mod kofile;

pub trait ToBytes {
    fn to_bytes(&self, buf: &mut Vec<u8>);
}

pub trait FromBytes {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self>
    where
        Self: Sized;
}

#[derive(Debug, PartialEq, Clone)]
pub enum KOSValue {
    Null,
    Bool(bool),
    Byte(i8),
    Int16(i16),
    Int32(i32),
    Float(f32),
    Double(f64),
    String(String),
    ArgMarker,
    ScalarInt(i32),
    ScalarDouble(f64),
    BoolValue(bool),
    StringValue(String),
}

impl KOSValue {
    pub fn size_bytes(&self) -> usize {
        match &self {
            Self::Null | Self::ArgMarker => 1,
            Self::Bool(_) | Self::Byte(_) | Self::BoolValue(_) => 2,
            Self::Int16(_) => 3,
            Self::Int32(_) | Self::Float(_) | Self::ScalarInt(_) => 5,
            Self::Double(_) | Self::ScalarDouble(_) => 9,
            Self::String(s) | Self::StringValue(s) => {
                2 + s.len() // 1 byte for the type, 1 byte for the length, and then the string
            }
        }
    }
}

impl ToBytes for KOSValue {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Null => {
                buf.push(0);
            }
            Self::Bool(b) => {
                buf.push(1);
                b.to_bytes(buf);
            }
            Self::Byte(b) => {
                buf.push(2);
                b.to_bytes(buf);
            }
            Self::Int16(i) => {
                buf.push(3);
                i.to_bytes(buf);
            }
            Self::Int32(i) => {
                buf.push(4);
                i.to_bytes(buf);
            }
            Self::Float(f) => {
                buf.push(5);
                f.to_bytes(buf);
            }
            Self::Double(f) => {
                buf.push(6);
                f.to_bytes(buf);
            }
            Self::String(s) => {
                buf.push(7);
                buf.push(s.len() as u8);
                s.to_bytes(buf);
            }
            Self::ArgMarker => {
                buf.push(8);
            }
            Self::ScalarInt(i) => {
                buf.push(9);
                i.to_bytes(buf);
            }
            Self::ScalarDouble(f) => {
                buf.push(10);
                f.to_bytes(buf);
            }
            Self::BoolValue(b) => {
                buf.push(11);
                b.to_bytes(buf);
            }
            Self::StringValue(s) => {
                buf.push(12);
                buf.push(s.len() as u8);
                s.to_bytes(buf);
            }
        }
    }
}

impl FromBytes for KOSValue {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let kos_type_value = *source.next().ok_or(ReadError::KOSValueReadError)?;
        let kos_read_error = ReadError::KOSValueReadError;

        Ok(match kos_type_value {
            0 => KOSValue::Null,
            1 => {
                let b = bool::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Bool(b)
            }
            2 => {
                let b = i8::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Byte(b)
            }
            3 => {
                let i = i16::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Int16(i)
            }
            4 => {
                let i = i32::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Int32(i)
            }
            5 => {
                let f = f32::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Float(f)
            }
            6 => {
                let d = f64::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::Double(d)
            }
            7 => {
                let s = String::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::String(s)
            }
            8 => KOSValue::ArgMarker,
            9 => {
                let i = i32::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::ScalarInt(i)
            }
            10 => {
                let d = f64::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::ScalarDouble(d)
            }
            11 => {
                let b = bool::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::BoolValue(b)
            }
            12 => {
                let s = String::from_bytes(source).map_err(|_| kos_read_error)?;
                KOSValue::StringValue(s)
            }
            _ => {
                return Err(ReadError::KOSValueTypeReadError(kos_type_value));
            }
        })
    }
}

impl ToBytes for bool {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push(if *self { 1 } else { 0 });
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl ToBytes for i8 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self) as u8);
    }
}

impl ToBytes for u16 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for i16 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for i32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for f32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for &str {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl ToBytes for String {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl FromBytes for bool {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        source
            .next()
            .map(|&x| x == 1)
            .ok_or(ReadError::ConstantReadError(
                ConstantReadError::BoolReadError,
            ))
    }
}

impl FromBytes for u8 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        source
            .next()
            .map(|&x| x)
            .ok_or(ReadError::ConstantReadError(ConstantReadError::U8ReadError))
    }
}

impl FromBytes for i8 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        source
            .next()
            .map(|&x| x as i8)
            .ok_or(ReadError::ConstantReadError(ConstantReadError::I8ReadError))
    }
}

impl FromBytes for u16 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 2];
        for i in 0..2 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::U16ReadError,
                ));
            }
        }
        Ok(u16::from_le_bytes(slice))
    }
}

impl FromBytes for i16 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 2];
        for i in 0..2 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::I16ReadError,
                ));
            }
        }
        Ok(i16::from_le_bytes(slice))
    }
}

impl FromBytes for u32 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::U32ReadError,
                ));
            }
        }
        Ok(u32::from_le_bytes(slice))
    }
}

impl FromBytes for i32 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::I32ReadError,
                ));
            }
        }
        Ok(i32::from_le_bytes(slice))
    }
}

impl FromBytes for f32 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::F32ReadError,
                ));
            }
        }
        Ok(f32::from_le_bytes(slice))
    }
}

impl FromBytes for f64 {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let mut slice = [0u8; 8];
        for i in 0..8 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::F64ReadError,
                ));
            }
        }
        Ok(f64::from_le_bytes(slice))
    }
}

impl FromBytes for String {
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self> {
        let len = match source.next() {
            Some(v) => *v,
            None => {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::StringReadError,
                ));
            }
        };
        let mut s = String::with_capacity(len as usize);
        for _ in 0..len {
            if let Some(&byte) = source.next() {
                s.push(byte as char);
            } else {
                return Err(ReadError::ConstantReadError(
                    ConstantReadError::StringReadError,
                ));
            }
        }
        Ok(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_to_bytes() {
        let v = KOSValue::Null;

        let mut buf = Vec::with_capacity(1);

        v.to_bytes(&mut buf);

        assert_eq!(buf, vec![0]);
    }

    #[test]
    fn bool_to_bytes() {
        let v1 = KOSValue::Bool(true);
        let v2 = KOSValue::Bool(false);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![1, 1]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![1, 0]);
    }

    #[test]
    fn byte_to_bytes() {
        let v1 = KOSValue::Byte(0);
        let v2 = KOSValue::Byte(-128);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![2, 0]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![2, (-128 as i8) as u8]);
    }

    #[test]
    fn int16_to_bytes() {
        let v1 = KOSValue::Int16(526);

        let mut buf = Vec::with_capacity(3);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![3, 0b00001110, 0b00000010]);
    }

    #[test]
    fn int32_to_bytes() {
        let v1 = KOSValue::Int32(-764);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![4, 0b00000100, 0b11111101, 0b11111111, 0b11111111]);
    }

    #[test]
    fn float_to_bytes() {
        let v1 = KOSValue::Float(3.14159);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![5, 208, 15, 73, 64]);
    }

    #[test]
    fn double_to_bytes() {
        let v1 = KOSValue::Double(3.14159);

        let mut buf = Vec::with_capacity(9);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![6, 110, 134, 27, 240, 249, 33, 9, 64]);
    }

    #[test]
    fn string_to_bytes() {
        let v1 = KOSValue::String(String::from("test str"));

        let mut buf = Vec::with_capacity(10);

        v1.to_bytes(&mut buf);

        assert_eq!(
            buf,
            vec![7, 8, b't', b'e', b's', b't', b' ', b's', b't', b'r']
        );
    }

    #[test]
    fn argmarker_to_bytes() {
        let v1 = KOSValue::ArgMarker;

        let mut buf = Vec::with_capacity(1);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![8]);
    }

    #[test]
    fn scalarint_to_bytes() {
        let v1 = KOSValue::ScalarInt(-1267);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![9, 0b00001101, 0b11111011, 0b11111111, 0b11111111]);
    }

    #[test]
    fn scalardouble_to_bytes() {
        let v1 = KOSValue::ScalarDouble(3.14159);

        let mut buf = Vec::with_capacity(9);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![10, 110, 134, 27, 240, 249, 33, 9, 64]);
    }

    #[test]
    fn boolvalue_to_bytes() {
        let v1 = KOSValue::BoolValue(true);
        let v2 = KOSValue::BoolValue(false);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![11, 1]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![11, 0]);
    }

    #[test]
    fn stringvalue_to_bytes() {
        let v1 = KOSValue::StringValue(String::from("hello"));

        let mut buf = Vec::with_capacity(7);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![12, 5, b'h', b'e', b'l', b'l', b'o']);
    }
}
