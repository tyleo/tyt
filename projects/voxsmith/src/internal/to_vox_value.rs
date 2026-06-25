use crate::Error;
use serde::{
    Serialize,
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant, Serializer,
    },
};
use voxcore::{VoxMap, VoxValue};

/// Serializes a value into a [`VoxValue`] tree, the inverse of
/// [`from_vox_value`](crate::from_vox_value). Numbers of any width become
/// [`VoxValue::Number`], structs and maps become [`VoxValue::Object`], and
/// sequences become [`VoxValue::Array`].
pub(crate) fn to_vox_value<T: Serialize + ?Sized>(value: &T) -> Result<VoxValue, Error> {
    value.serialize(VoxValueSerializer)
}

/// Serializes a map key, which must resolve to a string.
fn key_to_string<T: Serialize + ?Sized>(key: &T) -> Result<String, Error> {
    match to_vox_value(key)? {
        VoxValue::Text(text) => Ok(text),
        VoxValue::Number(number) => Ok(number.to_string()),
        VoxValue::Bool(bool) => Ok(bool.to_string()),
        _ => Err(Error::invalid("map key must serialize to a string")),
    }
}

/// A [`Serializer`] that builds a [`VoxValue`].
struct VoxValueSerializer;

impl Serializer for VoxValueSerializer {
    type Ok = VoxValue;
    type Error = Error;
    type SerializeSeq = SerializeArray;
    type SerializeTuple = SerializeArray;
    type SerializeTupleStruct = SerializeArray;
    type SerializeTupleVariant = SerializeVariantArray;
    type SerializeMap = SerializeObject;
    type SerializeStruct = SerializeObject;
    type SerializeStructVariant = SerializeVariantObject;

    fn serialize_bool(self, value: bool) -> Result<VoxValue, Error> {
        Ok(VoxValue::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_i16(self, value: i16) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_i32(self, value: i32) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_i64(self, value: i64) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_u8(self, value: u8) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_u16(self, value: u16) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_u32(self, value: u32) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_u64(self, value: u64) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_f32(self, value: f32) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value as f64))
    }

    fn serialize_f64(self, value: f64) -> Result<VoxValue, Error> {
        Ok(VoxValue::Number(value))
    }

    fn serialize_char(self, value: char) -> Result<VoxValue, Error> {
        Ok(VoxValue::Text(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<VoxValue, Error> {
        Ok(VoxValue::Text(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<VoxValue, Error> {
        Ok(VoxValue::Array(
            value.iter().map(|&b| VoxValue::Number(b as f64)).collect(),
        ))
    }

    fn serialize_none(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Null)
    }

    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<VoxValue, Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<VoxValue, Error> {
        Ok(VoxValue::Null)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
    ) -> Result<VoxValue, Error> {
        Ok(VoxValue::Text(variant.to_owned()))
    }

    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<VoxValue, Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<VoxValue, Error> {
        Ok(VoxValue::Object(VoxMap(vec![(
            variant.to_owned(),
            to_vox_value(value)?,
        )])))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<SerializeArray, Error> {
        Ok(SerializeArray { items: Vec::new() })
    }

    fn serialize_tuple(self, len: usize) -> Result<SerializeArray, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<SerializeArray, Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeVariantArray, Error> {
        Ok(SerializeVariantArray {
            name: variant.to_owned(),
            items: Vec::new(),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<SerializeObject, Error> {
        Ok(SerializeObject {
            entries: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<SerializeObject, Error> {
        Ok(SerializeObject {
            entries: Vec::new(),
            next_key: None,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<SerializeVariantObject, Error> {
        Ok(SerializeVariantObject {
            name: variant.to_owned(),
            entries: Vec::new(),
        })
    }
}

/// Collects sequence and tuple elements into a [`VoxValue::Array`].
struct SerializeArray {
    items: Vec<VoxValue>,
}

impl SerializeSeq for SerializeArray {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(to_vox_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Array(self.items))
    }
}

impl SerializeTuple for SerializeArray {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(to_vox_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Array(self.items))
    }
}

impl SerializeTupleStruct for SerializeArray {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(to_vox_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Array(self.items))
    }
}

/// Collects a tuple variant into a single-entry [`VoxValue::Object`] keyed by the
/// variant name.
struct SerializeVariantArray {
    name: String,
    items: Vec<VoxValue>,
}

impl SerializeTupleVariant for SerializeVariantArray {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        self.items.push(to_vox_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Object(VoxMap(vec![(
            self.name,
            VoxValue::Array(self.items),
        )])))
    }
}

/// Collects map and struct entries into a [`VoxValue::Object`].
struct SerializeObject {
    entries: Vec<(String, VoxValue)>,
    next_key: Option<String>,
}

impl SerializeMap for SerializeObject {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), Error> {
        self.next_key = Some(key_to_string(key)?);
        Ok(())
    }

    fn serialize_value<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Error> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| Error::invalid("map value serialized before its key"))?;
        self.entries.push((key, to_vox_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Object(VoxMap(self.entries)))
    }
}

impl SerializeStruct for SerializeObject {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.entries.push((key.to_owned(), to_vox_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Object(VoxMap(self.entries)))
    }
}

/// Collects a struct variant into a single-entry [`VoxValue::Object`] keyed by
/// the variant name.
struct SerializeVariantObject {
    name: String,
    entries: Vec<(String, VoxValue)>,
}

impl SerializeStructVariant for SerializeVariantObject {
    type Ok = VoxValue;
    type Error = Error;

    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Error> {
        self.entries.push((key.to_owned(), to_vox_value(value)?));
        Ok(())
    }

    fn end(self) -> Result<VoxValue, Error> {
        Ok(VoxValue::Object(VoxMap(vec![(
            self.name,
            VoxValue::Object(VoxMap(self.entries)),
        )])))
    }
}
