use schema::{Schema, Field, Document};
use fastfield::FastFieldSerializer;
use std::io;
use schema::Value;
use DocId;

pub struct U64FastFieldsWriter {
    field_writers: Vec<U64FastFieldWriter>,
}

impl U64FastFieldsWriter {

    pub fn from_schema(schema: &Schema) -> U64FastFieldsWriter {
        let u64_fields: Vec<Field> = schema.fields()
            .iter()
            .enumerate()
            .filter(|&(_, field_entry)| field_entry.is_u64_fast()) 
            .map(|(field_id, _)| Field(field_id as u32))
            .collect();
        U64FastFieldsWriter::new(u64_fields)
    }

    pub fn new(fields: Vec<Field>) -> U64FastFieldsWriter {
        U64FastFieldsWriter {
            field_writers: fields
                .into_iter()
                .map(U64FastFieldWriter::new)
                .collect(),
        }
    }
    
    pub fn get_field_writer(&mut self, field: Field) -> Option<&mut U64FastFieldWriter> {
        self.field_writers
            .iter_mut()
            .find(|field_writer| field_writer.field == field)
    }
    
    pub fn add_document(&mut self, doc: &Document) {
        for field_writer in &mut self.field_writers {
            field_writer.add_document(doc);
        }
    }

    pub fn serialize(&self, serializer: &mut FastFieldSerializer) -> io::Result<()> {
        for field_writer in &self.field_writers {
            try!(field_writer.serialize(serializer));
        }
        Ok(())
    }
    
    /// Ensures all of the fast field writers have
    /// reached `doc`. (included)
    /// 
    /// The missing values will be filled with 0.
    pub fn fill_val_up_to(&mut self, doc: DocId) {
        for field_writer in &mut self.field_writers {
            field_writer.fill_val_up_to(doc);
        }
            
    }
}

pub struct U64FastFieldWriter {
    field: Field,
    vals: Vec<u64>,
}

impl U64FastFieldWriter {
    pub fn new(field: Field) -> U64FastFieldWriter {
        U64FastFieldWriter {
            field: field,
            vals: Vec::new(),
        }
    }
    
    /// Ensures all of the fast field writer have
    /// reached `doc`. (included)
    /// 
    /// The missing values will be filled with 0.
    fn fill_val_up_to(&mut self, doc: DocId) {
        let target = doc as usize + 1;
        debug_assert!(self.vals.len() <= target);
        while self.vals.len() < target {
            self.add_val(0u64)
        }
    }
    
    pub fn add_val(&mut self, val: u64) {
        self.vals.push(val);
    }
    
    fn extract_val(&self, doc: &Document) -> u64 {
        match doc.get_first(self.field) {
            Some(v) => {
                match *v {
                    Value::U64(ref val) => { *val }
                    _ => { panic!("Expected a u64field, got {:?} ", v) }
                }
            },
            None => {
                0u64
            }            
        }
    }
    
    pub fn add_document(&mut self, doc: &Document) {
        let val = self.extract_val(doc);
        self.add_val(val);
    }

    pub fn serialize(&self, serializer: &mut FastFieldSerializer) -> io::Result<()> {
        let zero = 0;
        let min = *self.vals.iter().min().unwrap_or(&zero);
        let max = *self.vals.iter().max().unwrap_or(&min);
        try!(serializer.new_u64_fast_field(self.field, min, max));
        for &val in &self.vals {
            try!(serializer.add_val(val));
        }
        serializer.close_field()
    }
}
