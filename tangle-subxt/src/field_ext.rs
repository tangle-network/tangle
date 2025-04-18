use crate::tangle_testnet_runtime::api::runtime_types::{
	bounded_collections::bounded_vec::BoundedVec,
	tangle_primitives::services::field::{Field, FieldType},
};
use subxt_core::utils::AccountId32;

pub trait FieldExt {
	fn field_type(&self) -> FieldType;
}

impl FieldExt for Field<AccountId32> {
	fn field_type(&self) -> FieldType {
		match self {
			Field::Optional(ty, _) => FieldType::Optional(Box::new(ty.clone())),
			Field::Bool(_) => FieldType::Bool,
			Field::Uint8(_) => FieldType::Uint8,
			Field::Int8(_) => FieldType::Int8,
			Field::Uint16(_) => FieldType::Uint16,
			Field::Int16(_) => FieldType::Int16,
			Field::Uint32(_) => FieldType::Uint32,
			Field::Int32(_) => FieldType::Int32,
			Field::Uint64(_) => FieldType::Uint64,
			Field::Int64(_) => FieldType::Int64,
			Field::String(_) => FieldType::String,
			Field::Array(ty, values) => {
				FieldType::Array(values.0.len() as u64, Box::new(ty.clone()))
			},
			Field::List(ty, _) => FieldType::List(Box::new(ty.clone())),
			Field::Struct(_, fields) => {
				let mut type_fields = Vec::with_capacity(fields.0.len());
				for (_, field) in &fields.0 {
					type_fields.push(field.field_type());
				}

				FieldType::Struct(Box::new(BoundedVec(type_fields)))
			},
			Field::AccountId(_) => FieldType::AccountId,
		}
	}
}
