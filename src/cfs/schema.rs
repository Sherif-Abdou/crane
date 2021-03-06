use std::{convert::TryInto, fmt::Debug};

use super::buffer::{self, Buffer};


#[derive(Clone, PartialEq, Debug)]
pub enum DataValue {
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt64(u64),
    Varchar(String),
    Fixchar(String, u64),
}

impl DataValue {
    pub fn to_bytes(&self) -> Vec<u8> {
        match &self {
            Self::Int16(i) => ((*i).to_be_bytes().to_vec()),
            Self::Int32(i) => ((*i).to_be_bytes().to_vec()),
            Self::Int64(i) => ((*i).to_be_bytes().to_vec()),
            Self::UInt64(i) => ((*i).to_be_bytes().to_vec()),
            Self::Varchar(s) => (s.as_bytes().to_vec()),
            Self::Fixchar(s, i) => {
                let mut v = s.as_bytes().to_vec();
                while v.len() < ((*i) as usize) {
                    v.push(0u8);
                }
                v.append(&mut i.to_be_bytes().to_vec());
                v
            },
            _ => (vec![]),
        }
    }

    pub fn len(&self) -> Option<u64> {
        return match &self {
            Self::Int8(_) => Some(1),
            Self::Int16(_) => Some(2),
            Self::Int32(_) => Some(4),
            Self::Int64(_) => Some(8),
            Self::UInt64(_) => Some(8),
            Self::Fixchar(_, i) => Some(*i + 8),
            Self::Bool(_) => Some(1),
            Self::Varchar(_) => None,
        }
    }

    pub fn id(&self) -> u16 {
        match &self {
            Self::Int8(_) => 1,
            Self::Int16(_) => 2,
            Self::Int32(_) => 3,
            Self::Int64(_) => 4,
            Self::UInt64(_) => 5,
            Self::Fixchar(_, _) => 6,
            Self::Bool(_) => 7,
            Self::Varchar(_) => 8,
        }
    }

    pub fn from_id(id: u16, metadata: u64) -> Self {
        match id {
            1 => Self::Int8(0),
            2 => Self::Int16(0),
            3 => Self::Int32(0),
            4 => Self::Int64(0),
            5 => Self::UInt64(0),
            6 => Self::Fixchar("".to_string(), metadata),
            7 => Self::Bool(false),
            _ => unimplemented!(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, d_type: &mut DataValue) {
        let parse_err = "Couldn't parse value from bytes";
        let new_val = match d_type {
            Self::Int8(_) => Self::Int8(i8::from_be_bytes(bytes[..].try_into().expect(parse_err))),
            Self::Int16(_) => Self::Int16(i16::from_be_bytes(bytes[..].try_into().expect(parse_err))),
            Self::Int32(_) => Self::Int32(i32::from_be_bytes(bytes[..].try_into().expect(parse_err))),
            Self::Int64(_) => Self::Int64(i64::from_be_bytes(bytes[..].try_into().expect(parse_err))),
            Self::UInt64(_) => Self::UInt64(u64::from_be_bytes(bytes[..].try_into().expect(parse_err))),
            Self::Bool(_) => unimplemented!(),
            Self::Varchar(_) => unimplemented!(),
            Self::Fixchar(_, _) => {
                let (s, e) = (bytes.len()-8, bytes.len());
                let len_bytes = &bytes[s..e];
                let len = u64::from_be_bytes(len_bytes.try_into().expect(parse_err)) as usize;
                let str_bytes = &bytes[0..len];
                let str = String::from_utf8_lossy(str_bytes).to_string();
                let str = str.replace("\0", "");

                Self::Fixchar(str, len as u64)
            },
        };

        *d_type = new_val;
    }
}

#[derive(Clone)]
pub struct CraneSchema {
    pub types: Vec<DataValue>,
    pub names: Vec<String>,
}

impl CraneSchema {
    pub fn new(types: Vec<DataValue>) -> Self {
        CraneSchema {
            types,
            names: vec![],
        }
    }

    pub fn parse_bytes(&self, bytes: &mut Buffer) -> Vec<DataValue> {
        let mut values = self.types.clone();


        values.iter_mut()
            // .rev() // Last types written are first types parsed
            .for_each(|v| DataValue::from_bytes(bytes.consume(v.len().unwrap()), v));

        values
    }

    pub fn len(&self) -> u64 {
        self.types.iter().map(|v| v.len().unwrap()).sum()
    }

    pub fn produce_bytes(&self, values: &Vec<DataValue>) -> Vec<u8> {
        values.iter()
            .map(|v| v.to_bytes().iter().copied().collect::<Vec<u8>>())
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_schema() {
        let schema = vec![
            DataValue::Int16(21),
            DataValue::Int32(5000),
        ];

        let values = schema.clone();
        
        let schema = CraneSchema::new(schema);

        let mut bytes = Buffer::new(schema.produce_bytes(&values));

        let back_to_values = schema.parse_bytes(&mut bytes);

        assert_eq!(values, back_to_values);
    }
}