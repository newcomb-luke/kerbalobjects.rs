pub mod kofile;

pub trait ToBytes {
    fn to_bytes(&self, buf: &mut Vec<u8>);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KOSValue<'a> {
    Null,
    Bool(bool),
    Byte(i8),
    Int16(i16),
    Int32(i32),
    Float(f32),
    Double(f64),
    String(&'a str),
    ArgMarker,
    ScalarInt(i32),
    ScalarDouble(f64),
    BoolValue(bool),
    StringValue(&'a str),
}

impl<'a> ToBytes for KOSValue<'a> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Null => {
                buf.push(0);
            }
            Self::Bool(b) => {
                buf.push(1);
                buf.push(if *b { 1 } else { 0 });
            }
            Self::Byte(b) => {
                buf.push(2);
                buf.push(*b as u8);
            }
            Self::Int16(i) => {
                buf.push(3);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            Self::Int32(i) => {
                buf.push(4);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            Self::Float(f) => {
                buf.push(5);
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Self::Double(f) => {
                buf.push(6);
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Self::String(s) => {
                buf.push(7);
                buf.push(s.len() as u8);
                buf.extend_from_slice(s.as_bytes());
            }
            Self::ArgMarker => {
                buf.push(8);
            }
            Self::ScalarInt(i) => {
                buf.push(9);
                buf.extend_from_slice(&i.to_le_bytes());
            }
            Self::ScalarDouble(f) => {
                buf.push(10);
                buf.extend_from_slice(&f.to_le_bytes());
            }
            Self::BoolValue(b) => {
                buf.push(11);
                buf.push(if *b { 1 } else { 0 });
            }
            Self::StringValue(s) => {
                buf.push(12);
                buf.push(s.len() as u8);
                buf.extend_from_slice(s.as_bytes());
            }
        }
    }
}

#[macro_export]
macro_rules! push_32 {
    ($value: expr => $buf: ident) => {
        $buf.push((($value >> 24) & 0xff) as u8);
        $buf.push((($value >> 16) & 0xff) as u8);
        $buf.push((($value >> 8) & 0xff) as u8);
        $buf.push(($value & 0xff) as u8);
    };
}

#[macro_export]
macro_rules! push_16 {
    ($value: expr => $buf: ident) => {
        $buf.push((($value >> 8) & 0xff) as u8);
        $buf.push(($value & 0xff) as u8);
    };
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
        let s = "test str";

        let v1 = KOSValue::String(s);

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
        let s = "hello";

        let v1 = KOSValue::StringValue(s);

        let mut buf = Vec::with_capacity(7);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![12, 5, b'h', b'e', b'l', b'l', b'o']);
    }
}
