/// (De)serializes byte arrays as hex-string for human-readable formats (like
/// json) or as raw bytes otherwise
///
/// # Motivation
/// We want byte arrays to be serialized as bytes for binary formats, to keep
/// size of serialized data minimal. For that purpose, [`serde_with::Bytes`]
/// should suffice. However, serializing data as bytes on human-readable formats
/// (like json) is not efficient nor is it human-readable: bytes are serialized
/// as list of integers. Hex encoding is preferred for such formats as it's more
/// compact and readable.
///
/// # Private API
/// `HexOrBin` is shared between several crates in the project, however we do not
/// publicly expose it. Although it works perfectly fine in our case, it may not
/// work well sometimes due to limitations.
///
/// # Limitations
/// * Only works with byte arrays that have staticly-known size. `Array::default()` should return
///   `Array` filled with zeroes
/// * Using `HexOrBin` compiles, but deseralization basically always fails except for deserializing
///   empty arrays
/// * Only defined for arrays that implement [`Default`] trait. Note that `[u8; N]` implements this
///   trait only for limited amount of `N`.
pub struct HexOrBin;

impl<T> serde_with::SerializeAs<T> for HexOrBin
where
	T: AsRef<[u8]>,
{
	fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		if serializer.is_human_readable() {
			serializer.serialize_str(&hex::encode(source))
		} else {
			serializer.serialize_bytes(source.as_ref())
		}
	}
}

impl<'de, T> serde_with::DeserializeAs<'de, T> for HexOrBin
where
	T: Default + AsMut<[u8]>,
{
	fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		struct Visitor<T> {
			expect_hex: bool,
			out: T,
			_ph: core::marker::PhantomData<T>,
		}
		impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
		where
			T: AsMut<[u8]>,
		{
			type Value = T;

			fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
				if self.expect_hex {
					formatter.write_str("hex-encoded byte string")
				} else {
					formatter.write_str("byte string")
				}
			}

			fn visit_bytes<E>(mut self, v: &[u8]) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				if self.expect_hex {
					return Err(E::invalid_value(
						serde::de::Unexpected::Bytes(v),
						&"expected hex-encoded bytes",
					));
				}
				let out_len = self.out.as_mut().len();
				if out_len != v.len() {
					return Err(E::invalid_length(v.len(), &ExpectedLen(out_len)));
				}
				self.out.as_mut().copy_from_slice(v);
				Ok(self.out)
			}

			fn visit_str<E>(mut self, v: &str) -> Result<Self::Value, E>
			where
				E: serde::de::Error,
			{
				if !self.expect_hex {
					return Err(E::invalid_value(
						serde::de::Unexpected::Str(v),
						&"expected raw bytes",
					));
				}

				hex::decode_to_slice(v, self.out.as_mut()).map_err(E::custom)?;

				Ok(self.out)
			}
		}

		if deserializer.is_human_readable() {
			deserializer.deserialize_str(Visitor {
				expect_hex: true,
				out: Default::default(),
				_ph: Default::default(),
			})
		} else {
			deserializer.deserialize_bytes(Visitor {
				expect_hex: false,
				out: Default::default(),
				_ph: Default::default(),
			})
		}
	}
}

struct ExpectedLen(usize);
impl serde::de::Expected for ExpectedLen {
	fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
		write!(formatter, "{}", self.0)
	}
}
