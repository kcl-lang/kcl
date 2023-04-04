use prost_wkt::MessageSerde;
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, SerializeStruct, Serializer};

include!(concat!(env!("OUT_DIR"), "/pbany/google.protobuf.rs"));

use prost::{DecodeError, Message};

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnyError {
    description: Cow<'static, str>,
}

impl AnyError {
    pub fn new<S>(description: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        AnyError {
            description: description.into(),
        }
    }
}

impl std::error::Error for AnyError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl std::fmt::Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("failed to convert Value: ")?;
        f.write_str(&self.description)
    }
}

impl From<prost::DecodeError> for AnyError {
    fn from(error: DecodeError) -> Self {
        AnyError::new(format!("Error decoding message: {error:?}"))
    }
}

impl From<prost::EncodeError> for AnyError {
    fn from(error: prost::EncodeError) -> Self {
        AnyError::new(format!("Error encoding message: {error:?}"))
    }
}

impl Any {
    /// Packs a message into an `Any` containing a `type_url` which will take the format
    /// of `type.googleapis.com/package_name.struct_name`, and a value containing the
    /// encoded message.
    #[deprecated(since = "0.3.0", note = "please use `try_pack` instead")]
    pub fn pack<T>(message: T) -> Self
    where
        T: Message + MessageSerde + Default,
    {
        let type_url = MessageSerde::type_url(&message).to_string();
        // Serialize the message into a value
        let mut buf = Vec::new();
        buf.reserve(message.encoded_len());
        message.encode(&mut buf).expect("Failed to encode message");
        Any {
            type_url,
            value: buf,
        }
    }

    /// Packs a message into an `Any` containing a `type_url` which will take the format
    /// of `type.googleapis.com/package_name.struct_name`, and a value containing the
    /// encoded message.
    pub fn try_pack<T>(message: T) -> Result<Self, AnyError>
    where
        T: Message + MessageSerde + Default,
    {
        let type_url = MessageSerde::type_url(&message).to_string();
        // Serialize the message into a value
        let mut buf = Vec::new();
        buf.reserve(message.encoded_len());
        message.encode(&mut buf)?;
        let encoded = Any {
            type_url,
            value: buf,
        };
        Ok(encoded)
    }

    /// Unpacks the contents of the `Any` into the provided message type. Example usage:
    ///
    /// ```ignore
    /// let back: Foo = any.unpack_as(Foo::default())?;
    /// ```
    pub fn unpack_as<T: Message>(self, mut target: T) -> Result<T, AnyError> {
        let instance = target.merge(self.value.as_slice()).map(|_| target)?;
        Ok(instance)
    }

    #[deprecated(since = "0.3.0", note = "Method renamed to `try_unpack`")]
    pub fn unpack(self) -> Result<Box<dyn prost_wkt::MessageSerde>, AnyError> {
        self.try_unpack()
    }

    /// Unpacks the contents of the `Any` into the `MessageSerde` trait object. Example
    /// usage:
    ///
    /// ```ignore
    /// let back: Box<dyn MessageSerde> = any.try_unpack()?;
    /// ```
    pub fn try_unpack(self) -> Result<Box<dyn prost_wkt::MessageSerde>, AnyError> {
        ::prost_wkt::inventory::iter::<::prost_wkt::MessageSerdeDecoderEntry>
            .into_iter()
            .find(|entry| self.type_url == entry.type_url)
            .ok_or_else(|| format!("Failed to deserialize {}. Make sure prost-wkt-build is executed.", self.type_url))
            .and_then(|entry| {
                (entry.decoder)(&self.value).map_err(|error| {
                    format!(
                        "Failed to deserialize {}. Make sure it implements prost::Message. Error reported: {}",
                        self.type_url,
                        error
                    )
                })
            })
            .map_err(AnyError::new)
    }
}

impl Serialize for Any {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match self.clone().try_unpack() {
            Ok(result) => serde::ser::Serialize::serialize(result.as_ref(), serializer),
            Err(_) => {
                let mut state = serializer.serialize_struct("Any", 3)?;
                state.serialize_field("@type", &self.type_url)?;
                state.serialize_field("value", &self.value)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Any {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let erased: Box<dyn prost_wkt::MessageSerde> =
            serde::de::Deserialize::deserialize(deserializer)?;
        let type_url = erased.type_url().to_string();
        let value = erased.try_encoded().map_err(|err| {
            serde::de::Error::custom(format!("Failed to encode message: {err:?}"))
        })?;
        Ok(Any { type_url, value })
    }
}

#[cfg(test)]
mod tests {
    use crate::pbany::*;
    use prost::{DecodeError, EncodeError, Message};
    use prost_wkt::*;
    use serde::*;
    use serde_json::json;

    #[derive(Clone, Eq, PartialEq, ::prost::Message, Serialize, Deserialize)]
    #[serde(default, rename_all = "camelCase")]
    pub struct Foo {
        #[prost(string, tag = "1")]
        pub string: std::string::String,
    }

    #[typetag::serde(name = "type.googleapis.com/any.test.Foo")]
    impl prost_wkt::MessageSerde for Foo {
        fn message_name(&self) -> &'static str {
            "Foo"
        }

        fn package_name(&self) -> &'static str {
            "any.test"
        }

        fn type_url(&self) -> &'static str {
            "type.googleapis.com/any.test.Foo"
        }
        fn new_instance(&self, data: Vec<u8>) -> Result<Box<dyn MessageSerde>, DecodeError> {
            let mut target = Self::default();
            Message::merge(&mut target, data.as_slice())?;
            let erased: Box<dyn MessageSerde> = Box::new(target);
            Ok(erased)
        }

        fn try_encoded(&self) -> Result<Vec<u8>, EncodeError> {
            let mut buf = Vec::new();
            buf.reserve(Message::encoded_len(self));
            Message::encode(self, &mut buf)?;
            Ok(buf)
        }
    }

    #[test]
    fn pack_unpack_test() {
        let msg = Foo {
            string: "Hello World!".to_string(),
        };
        let any = Any::try_pack(msg.clone()).unwrap();
        println!("{any:?}");
        let unpacked = any.unpack_as(Foo::default()).unwrap();
        println!("{unpacked:?}");
        assert_eq!(unpacked, msg)
    }

    #[test]
    fn pack_unpack_with_downcast_test() {
        let msg = Foo {
            string: "Hello World!".to_string(),
        };
        let any = Any::try_pack(msg.clone()).unwrap();
        println!("{any:?}");
        let unpacked: &dyn MessageSerde = &any.unpack_as(Foo::default()).unwrap();
        let downcast = unpacked.downcast_ref::<Foo>().unwrap();
        assert_eq!(downcast, &msg);
    }

    #[test]
    fn deserialize_default_test() {
        let type_url = "type.googleapis.com/any.test.Foo";
        let data = json!({
            "@type": type_url,
            "value": {}
        });
        let erased: Box<dyn MessageSerde> = serde_json::from_value(data).unwrap();
        let foo: &Foo = erased.downcast_ref::<Foo>().unwrap();
        println!("Deserialize default: {foo:?}");
        assert_eq!(foo, &Foo::default())
    }
}
