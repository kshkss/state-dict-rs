use serde::{ser, Serialize};
use std::collections::HashMap;

use crate::error::{Error, Result};

pub struct Serializer {
    // This string starts empty and JSON is appended as values are serialized.
    counter: usize,
    pos: Vec<String>,
    output: HashMap<String, f64>,
}

impl Serializer {
    fn new(root: String) -> Self {
        Self {
            counter: 0,
            pos: vec![root],
            output: HashMap::new(),
        }
    }

    fn is_root(&self) -> bool {
        self.pos.len() == 0
    }

    fn push_key(&mut self, key: &str) {
        let len = self.pos.len();
        let new_pos = if len == 0 {
            key.to_string()
        } else {
            self.pos[len - 1].to_owned() + "." + key
        };
        self.pos.push(new_pos);
    }

    fn push_index(&mut self, i: i32) {
        let len = self.pos.len();
        let new_pos = if len == 0 {
            format!("[{}]", i)
        } else {
            self.pos[len - 1].to_owned() + &format!("[{}]", i)
        };
        self.pos.push(new_pos);
    }

    fn pop(&mut self) {
        self.pos.pop();
    }

    fn insert(&mut self, value: f64) {
        assert_ne!(self.pos.len(), 0);
        self.output
            .insert(self.pos[self.pos.len() - 1].to_owned(), value);
    }
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
//
// This basic serializer supports only `to_string`.
pub fn to_hashmap<T>(value: &T) -> Result<HashMap<String, f64>>
where
    T: Serialize,
{
    let mut serializer = Serializer::new("$".to_string());
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_f64(if v { 1. } else { 0. })
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.insert(v);
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, _v: char) -> Result<()> {
        Err(Error::Unsupported)
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(Error::Unsupported)
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(Error::Unsupported)
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        self.serialize_f64(f64::NAN)
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_u32(variant_index)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_u32(variant_index)?;
        self.push_index(0);
        value.serialize(&mut *self)?;
        self.pop();
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_u32(variant_index)?;
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    // Structs look just like maps in JSON. In particular, JSON requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        if self.is_root() {
            self.push_key("_");
        }
        self.serialize_u32(variant_index)?;
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_index(self.counter as i32);
        self.counter += 1;
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        self.counter = 0;
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_index(self.counter as i32);
        self.counter += 1;
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.counter = 0;
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_index(self.counter as i32);
        self.counter += 1;
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.counter = 0;
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_index(self.counter as i32);
        self.counter += 1;
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.counter = 0;
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = key.serialize(StringExtractor)?;
        self.push_key(&key);
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_key(key);
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_key(key);
        value.serialize(&mut **self)?;
        self.pop();
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

struct StringExtractor;

impl ser::Serializer for StringExtractor {
    type Ok = String;
    type Error = Error;
    type SerializeSeq = ser::Impossible<String, Error>;
    type SerializeTuple = ser::Impossible<String, Error>;
    type SerializeTupleStruct = ser::Impossible<String, Error>;
    type SerializeTupleVariant = ser::Impossible<String, Error>;
    type SerializeMap = ser::Impossible<String, Error>;
    type SerializeStruct = ser::Impossible<String, Error>;
    type SerializeStructVariant = ser::Impossible<String, Error>;

    fn serialize_bool(self, _v: bool) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_i8(self, _v: i8) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_i16(self, _v: i16) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_i32(self, _v: i32) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_i64(self, _v: i64) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_u8(self, _v: u8) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_u16(self, _v: u16) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_u32(self, _v: u32) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_u64(self, _v: u64) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_f32(self, _v: f32) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_f64(self, _v: f64) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_char(self, _v: char) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_none(self) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_some<T: ?Sized>(self, _value: &T) -> Result<String>
    where
        T: ser::Serialize,
    {
        Err(Error::KeyNotString)
    }

    fn serialize_unit(self) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<String> {
        Err(Error::KeyNotString)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ser::Serialize,
    {
        Err(Error::KeyNotString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(Error::KeyNotString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(Error::KeyNotString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(Error::KeyNotString)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(Error::KeyNotString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(Error::KeyNotString)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[test]
fn test_struct() {
    #[derive(Serialize)]
    struct Test {
        int: u32,
        seq: Vec<f32>,
    }

    let test = Test {
        int: 1,
        seq: vec![2., 3.],
    };
    let dict = to_hashmap(&test).unwrap();

    assert_eq!(dict.get("$.int"), Some(&1.));
    assert_eq!(dict.get("$.seq[0]"), Some(&2.));
    assert_eq!(dict.get("$.seq[1]"), Some(&3.));
    assert_eq!(dict.iter().count(), 3);
}

#[test]
fn test_enum() {
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let u = E::Unit;
    let dict = to_hashmap(&u).unwrap();
    assert_eq!(dict.iter().count(), 1);
    assert_eq!(dict.get("$"), Some(&0.));

    let n = E::Newtype(1);
    let dict = to_hashmap(&n).unwrap();
    assert_eq!(dict.iter().count(), 2);
    assert_eq!(dict.get("$"), Some(&1.));
    assert_eq!(dict.get("$[0]"), Some(&1.));

    let t = E::Tuple(1, 2);
    let dict = to_hashmap(&t).unwrap();
    assert_eq!(dict.iter().count(), 3);
    assert_eq!(dict.get("$"), Some(&2.));
    assert_eq!(dict.get("$[0]"), Some(&1.));
    assert_eq!(dict.get("$[1]"), Some(&2.));

    let s = E::Struct { a: 1 };
    let dict = to_hashmap(&s).unwrap();
    assert_eq!(dict.iter().count(), 2);
    assert_eq!(dict.get("$"), Some(&3.));
    assert_eq!(dict.get("$.a"), Some(&1.));
}

#[test]
fn test_nested() {
    #[derive(Serialize, Clone)]
    struct Test {
        int: u32,
        seq: Vec<f32>,
    }
    #[derive(Serialize)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }
    #[derive(Serialize)]
    struct Test2 {
        a: Test,
        b: E,
    }

    let test_a = Test {
        int: 1,
        seq: vec![2., 3.],
    };

    let u = Test2 {
        a: test_a.clone(),
        b: E::Unit,
    };
    let dict = to_hashmap(&u).unwrap();
    assert_eq!(dict.iter().count(), 4);
    assert_eq!(dict.get("$.a.int"), Some(&1.));
    assert_eq!(dict.get("$.a.seq[0]"), Some(&2.));
    assert_eq!(dict.get("$.a.seq[1]"), Some(&3.));
    assert_eq!(dict.get("$.b"), Some(&0.));

    let n = Test2 {
        a: test_a.clone(),
        b: E::Newtype(1),
    };
    let dict = to_hashmap(&n).unwrap();
    assert_eq!(dict.iter().count(), 5);
    assert_eq!(dict.get("$.a.int"), Some(&1.));
    assert_eq!(dict.get("$.a.seq[0]"), Some(&2.));
    assert_eq!(dict.get("$.a.seq[1]"), Some(&3.));
    assert_eq!(dict.get("$.b"), Some(&1.));
    assert_eq!(dict.get("$.b[0]"), Some(&1.));

    let t = Test2 {
        a: test_a.clone(),
        b: E::Tuple(1, 2),
    };
    let dict = to_hashmap(&t).unwrap();
    assert_eq!(dict.iter().count(), 6);
    assert_eq!(dict.get("$.a.int"), Some(&1.));
    assert_eq!(dict.get("$.a.seq[0]"), Some(&2.));
    assert_eq!(dict.get("$.a.seq[1]"), Some(&3.));
    assert_eq!(dict.get("$.b"), Some(&2.));
    assert_eq!(dict.get("$.b[0]"), Some(&1.));
    assert_eq!(dict.get("$.b[1]"), Some(&2.));

    let s = Test2 {
        a: test_a.clone(),
        b: E::Struct { a: 1 },
    };
    let dict = to_hashmap(&s).unwrap();
    assert_eq!(dict.iter().count(), 5);
    assert_eq!(dict.get("$.a.int"), Some(&1.));
    assert_eq!(dict.get("$.a.seq[0]"), Some(&2.));
    assert_eq!(dict.get("$.a.seq[1]"), Some(&3.));
    assert_eq!(dict.get("$.b"), Some(&3.));
    assert_eq!(dict.get("$.b.a"), Some(&1.));
}
