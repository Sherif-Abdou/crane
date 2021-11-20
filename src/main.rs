mod cfs;

use std::path::{PathBuf};
use std::fs::{File, OpenOptions};
use std::io::Write;

pub enum DataTypes {
    Bool(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Varchar(String),
    Fixchar(String, u32),
}

impl DataTypes {
    pub fn to_bytes(&self) -> Vec<u8> {
        return match &self {
            Self::Int16(i) => ((*i).to_ne_bytes().to_vec()),
            Self::Int32(i) => ((*i).to_ne_bytes().to_vec()),
            Self::Int64(i) => ((*i).to_ne_bytes().to_vec()),
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

    pub fn len(&self) -> Option<u32> {
        return match &self {
            Self::Int8(_) => Some(8),
            Self::Int16(_) => Some(16),
            Self::Int32(_) => Some(32),
            Self::Int64(_) => Some(64),
            Self::Fixchar(_, i) => Some(*i),
            Self::Bool(_) => Some(1),
            Self::Varchar(_) => None,
        }
    }
}

type Column = Vec<DataTypes>;
pub struct Schema {
    pub name: String,
    pub slots: usize,
    pub align: Option<u32>,
    pub types: Column,
    pub values: Vec<Column>,
}

impl Schema {
    pub fn new(name: String, types: Vec<DataTypes>) -> Schema {
        let align = self::Schema::calc_align(&types);
        Schema {
            name,
            slots: 0,
            align,
            types,
            values: vec![]
        }
    }

    pub fn calc_align(types: &Vec<DataTypes>) -> Option<u32> {
        types.iter()
            .map(|v| v.len())
            .sum()
    }
}

pub struct Crane {
    pub schema: Schema,
    pub file_path: PathBuf,
}

impl Crane {
    pub fn new(schema: Schema, file_path: PathBuf) -> Crane {
        if !file_path.exists() {
            File::create(file_path.clone()).expect("Couldn't write to db file");
        }
        Crane {
            schema,
            file_path
        }
    }

    pub fn write_memory(&self) {
        let mut file = OpenOptions::new().write(true).open(&self.file_path).expect("Couldn't open file for writing");

        let size = self.schema.values.iter()
            .flatten()
            .map(|v| file.write(v.to_bytes().as_slice()).unwrap())
            .sum::<usize>();

        file.flush().unwrap();
    }
}


#[cfg(test)]
mod test {
    use std::{io::{Read, Seek, SeekFrom}, vec};

    use super::*;

    fn gen_schema() -> Schema {
        let slots: Vec<Column> = vec![
            vec![DataTypes::Int16(257), DataTypes::Int8(0)]];

        let mut res = Schema::new("stuff".to_string(), vec![
            DataTypes::Int16(0),
            DataTypes::Int8(0),
        ]);

        res.values = slots;

        res
    }

    #[test]
    fn test_thing() {
        let path = PathBuf::from("./test/test.db");
        let schema = gen_schema();

        let crane = Crane::new(schema, path);

        crane.write_memory();
    }

    #[test]
    fn test_double_file() {
        let path = PathBuf::from("./.gitignore");
        let mut f1 = File::open(&path).unwrap();

        f1.seek(SeekFrom::Start(0)).unwrap();
        let v  = (&f1).bytes().next().unwrap().unwrap().clone();

        f1.seek(SeekFrom::Start(1)).unwrap();
        let h  = (&f1).bytes().next().unwrap().unwrap().clone();

        assert!(v != h);
    }
}


fn main() {
    println!("Hello, world!");
}
