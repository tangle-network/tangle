use crate::tests::{ConstraintsOf, Runtime};
use sp_core::{bounded_vec, crypto::AccountId32};
use tangle_primitives::services::{Field as PrimitivesField, FieldType};

type Field = PrimitivesField<ConstraintsOf<Runtime>, AccountId32>;

#[test]
fn field_type_check() {
	let f = Field::Optional(FieldType::Uint8, Some(Box::new(Field::Uint8(0))));
	assert_eq!(f, FieldType::Optional(Box::new(FieldType::Uint8)));

	let f = Field::Optional(FieldType::Uint8, None);
	assert_eq!(f, FieldType::Optional(Box::new(FieldType::Uint8)));

	let f = Field::List(FieldType::Uint8, bounded_vec![Field::Uint8(0), Field::Uint8(1)]);
	assert_eq!(f, FieldType::List(Box::new(FieldType::Uint8)));

	let f = Field::Array(FieldType::Uint8, bounded_vec![Field::Uint8(0), Field::Uint8(1)]);
	assert_eq!(f, FieldType::Array(2, Box::new(FieldType::Uint8)));

	// == Should fail ==

	// Optional lying about its contents
	let f =
		Field::Optional(FieldType::Uint8, Some(Box::new(Field::String("a".try_into().unwrap()))));
	assert_ne!(f, FieldType::Optional(Box::new(FieldType::Uint8)));

	// List lying about its contents
	let f = Field::List(
		FieldType::Uint8,
		bounded_vec![
			Field::String("a".try_into().unwrap()),
			Field::String("b".try_into().unwrap())
		],
	);
	assert_ne!(f, FieldType::List(Box::new(FieldType::Uint8)));

	// List with mixed field types
	let f = Field::List(
		FieldType::Uint8,
		bounded_vec![Field::Uint8(0), Field::String("b".try_into().unwrap())],
	);
	assert_ne!(f, FieldType::List(Box::new(FieldType::Uint8)));

	// Array lying about its contents
	let f = Field::Array(
		FieldType::Uint8,
		bounded_vec![
			Field::String("a".try_into().unwrap()),
			Field::String("b".try_into().unwrap())
		],
	);
	assert_ne!(f, FieldType::Array(2, Box::new(FieldType::Uint8)));

	// Array lying mixed field types
	let f = Field::Array(
		FieldType::Uint8,
		bounded_vec![Field::Uint8(0), Field::String("b".try_into().unwrap())],
	);
	assert_ne!(f, FieldType::Array(2, Box::new(FieldType::Uint8)));

	// Array with a bad length
	let f = Field::Array(FieldType::String, bounded_vec![Field::String("a".try_into().unwrap())]);
	assert_ne!(f, FieldType::Array(2, Box::new(FieldType::String)));
}
