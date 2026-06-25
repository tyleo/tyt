use crate::Error;
use serde::{
    Deserialize,
    de::{
        DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
        value::StrDeserializer,
    },
    forward_to_deserialize_any,
};
use std::slice;
use voxcore::{VoxMap, VoxValue};

/// Deserializes a value out of a [`VoxValue`] tree, the inverse of
/// [`to_vox_value`](crate::to_vox_value).
pub(crate) fn from_vox_value<'de, T: Deserialize<'de>>(value: &'de VoxValue) -> Result<T, Error> {
    T::deserialize(VoxValueDeserializer(value))
}

/// A [`Deserializer`] over a borrowed [`VoxValue`]. The newtype keeps the impl
/// local so the orphan rule allows it.
struct VoxValueDeserializer<'de>(&'de VoxValue);

/// Implements an integer/float deserialize method by reading a
/// [`VoxValue::Number`] and falling back to `deserialize_any` otherwise.
macro_rules! deserialize_number {
    ($method:ident, $visit:ident, $t:ty) => {
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self.0 {
                VoxValue::Number(number) => visitor.$visit(*number as $t),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

impl<'de> Deserializer<'de> for VoxValueDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Null => visitor.visit_unit(),
            VoxValue::Bool(value) => visitor.visit_bool(*value),
            // An integral value visits as an integer so a buffering consumer (an
            // untagged enum) can still read an integer variant; only fractional
            // values fall back to floating point.
            VoxValue::Number(value)
                if value.fract() == 0.0
                    && *value >= i64::MIN as f64
                    && *value <= i64::MAX as f64 =>
            {
                visitor.visit_i64(*value as i64)
            }
            VoxValue::Number(value) => visitor.visit_f64(*value),
            VoxValue::Text(value) => visitor.visit_str(value),
            VoxValue::Array(values) => visitor.visit_seq(SeqRef(values.iter())),
            VoxValue::Object(VoxMap(entries)) => visitor.visit_map(MapRef {
                iter: entries.iter(),
                value: None,
            }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Text(variant) => visitor.visit_enum(EnumRef {
                variant,
                value: None,
            }),
            VoxValue::Object(VoxMap(entries)) if entries.len() == 1 => {
                let (variant, value) = &entries[0];
                visitor.visit_enum(EnumRef {
                    variant,
                    value: Some(value),
                })
            }
            _ => Err(Error::invalid("invalid enum value")),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Bool(value) => visitor.visit_bool(*value),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Text(text) if text.chars().count() == 1 => {
                visitor.visit_char(text.chars().next().expect("one char"))
            }
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Text(text) => visitor.visit_str(text),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_str(visitor)
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            VoxValue::Null => visitor.visit_unit(),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_unit(visitor)
    }

    deserialize_number!(deserialize_i8, visit_i8, i8);
    deserialize_number!(deserialize_i16, visit_i16, i16);
    deserialize_number!(deserialize_i32, visit_i32, i32);
    deserialize_number!(deserialize_i64, visit_i64, i64);
    deserialize_number!(deserialize_i128, visit_i128, i128);
    deserialize_number!(deserialize_u8, visit_u8, u8);
    deserialize_number!(deserialize_u16, visit_u16, u16);
    deserialize_number!(deserialize_u32, visit_u32, u32);
    deserialize_number!(deserialize_u64, visit_u64, u64);
    deserialize_number!(deserialize_u128, visit_u128, u128);
    deserialize_number!(deserialize_f32, visit_f32, f32);
    deserialize_number!(deserialize_f64, visit_f64, f64);

    forward_to_deserialize_any! {
        bytes byte_buf seq tuple tuple_struct map struct identifier ignored_any
    }
}

/// A [`SeqAccess`] over the elements of a [`VoxValue::Array`].
struct SeqRef<'de>(slice::Iter<'de, VoxValue>);

impl<'de> SeqAccess<'de> for SeqRef<'de> {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        match self.0.next() {
            Some(value) => seed.deserialize(VoxValueDeserializer(value)).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

/// A [`MapAccess`] over the entries of a [`VoxValue::Object`].
struct MapRef<'de> {
    iter: slice::Iter<'de, (String, VoxValue)>,
    value: Option<&'de VoxValue>,
}

impl<'de> MapAccess<'de> for MapRef<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key: StrDeserializer<Error> = StrDeserializer::new(key);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| Error::invalid("map value requested before its key"))?;
        seed.deserialize(VoxValueDeserializer(value))
    }
}

/// An [`EnumAccess`] reading a variant name and its optional payload.
struct EnumRef<'de> {
    variant: &'de str,
    value: Option<&'de VoxValue>,
}

impl<'de> EnumAccess<'de> for EnumRef<'de> {
    type Error = Error;
    type Variant = VariantRef<'de>;

    fn variant_seed<V: DeserializeSeed<'de>>(
        self,
        seed: V,
    ) -> Result<(V::Value, VariantRef<'de>), Error> {
        let variant: StrDeserializer<Error> = StrDeserializer::new(self.variant);
        let value = seed.deserialize(variant)?;
        Ok((value, VariantRef(self.value)))
    }
}

/// A [`VariantAccess`] over an enum variant's payload.
struct VariantRef<'de>(Option<&'de VoxValue>);

impl<'de> VariantAccess<'de> for VariantRef<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.0 {
            None => Ok(()),
            Some(_) => Err(Error::invalid("expected a unit variant")),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        match self.0 {
            Some(value) => seed.deserialize(VoxValueDeserializer(value)),
            None => Err(Error::invalid("expected a newtype variant")),
        }
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Some(VoxValue::Array(values)) => visitor.visit_seq(SeqRef(values.iter())),
            _ => Err(Error::invalid("expected a tuple variant")),
        }
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Some(VoxValue::Object(VoxMap(entries))) => visitor.visit_map(MapRef {
                iter: entries.iter(),
                value: None,
            }),
            _ => Err(Error::invalid("expected a struct variant")),
        }
    }
}
