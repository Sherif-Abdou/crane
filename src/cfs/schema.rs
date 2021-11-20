use std::convert::TryInto;

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
                v.push(0u8);
                v.append(&mut i.to_ne_bytes().to_vec());
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
            Self::Fixchar(_, i) => Some(*i),
            Self::Bool(_) => Some(1),
            Self::Varchar(_) => None,
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, d_type: &mut DataValue) {
        let new_val = match d_type {
            Self::Int8(_) => Self::Int8(i8::from_be_bytes(bytes[..].try_into().unwrap())),
            Self::Int16(_) => Self::Int16(i16::from_be_bytes(bytes[..].try_into().unwrap())),
            Self::Int32(_) => Self::Int32(i32::from_be_bytes(bytes[..].try_into().unwrap())),
            Self::Int64(_) => Self::Int64(i64::from_be_bytes(bytes[..].try_into().unwrap())),
            Self::UInt64(_) => Self::UInt64(u64::from_be_bytes(bytes[..].try_into().unwrap())),
            Self::Bool(_) => unimplemented!(),
            Self::Varchar(_) => unimplemented!(),
            Self::Fixchar(_, l) => Self::Fixchar(String::from_utf8_lossy(&bytes).to_string(), *l),
        };

        *d_type = new_val;
    }
}

pub struct CraneSchema {
    pub types: Vec<DataValue>
}

impl CraneSchema {
    pub fn new(types: Vec<DataValue>) -> Self {
        CraneSchema {
            types
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