// This file is part of Tangle.
// Copyright (C) 2022-2024 Webb Technologies Inc.
//
// This file is part of Utils package, originally developed by Purestake Inc.
// Utils package used in Tangle Network in terms of GPLv3.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use syn::{GenericArgument, Type};

pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemType);

	let ItemType { attrs, vis, type_token, ident, generics, eq_token, ty, semi_token } = item;

	if let Type::Tuple(ref type_tuple) = *ty {
		let variants: Vec<(Ident, u64)> =
			type_tuple.elems.iter().filter_map(extract_precompile_name_and_prefix).collect();

		let ident_expressions: Vec<&Ident> = variants.iter().map(|(ident, _)| ident).collect();
		let variant_expressions: Vec<&u64> = variants.iter().map(|(_, id)| id).collect();

		(quote! {
			#(#attrs)*
			#vis #type_token #ident #generics #eq_token #ty #semi_token

			#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug)]
			#[repr(u64)]
			pub enum PrecompileName {
				#(
					#ident_expressions = #variant_expressions,
				)*
			}

			impl PrecompileName {
				pub fn from_address(address: sp_core::H160) -> Option<Self> {
					let _u64 = address.to_low_u64_be();
					if address == sp_core::H160::from_low_u64_be(_u64) {
						use num_enum::TryFromPrimitive;
						Self::try_from_primitive(_u64).ok()
					} else {
						None
					}
				}
			}
		})
		.into()
	} else {
		quote_spanned! {
			ty.span() => compile_error!("Expected tuple");
		}
		.into()
	}
}

fn extract_precompile_name_and_prefix(type_: &Type) -> Option<(Ident, u64)> {
	match type_ {
		Type::Path(type_path) => {
			if let Some(path_segment) = type_path.path.segments.last() {
				match path_segment.ident.to_string().as_ref() {
					"PrecompileAt" => {
						extract_precompile_name_and_prefix_for_precompile_at(path_segment)
					},
					_ => None,
				}
			} else {
				None
			}
		},
		_ => None,
	}
}

fn extract_precompile_name_and_prefix_for_precompile_at(
	path_segment: &syn::PathSegment,
) -> Option<(Ident, u64)> {
	if let syn::PathArguments::AngleBracketed(generics) = &path_segment.arguments {
		let mut iter = generics.args.iter();
		if let (
			Some(GenericArgument::Type(Type::Path(type_path_1))),
			Some(GenericArgument::Type(Type::Path(type_path_2))),
		) = (iter.next(), iter.next())
		{
			if let (Some(path_segment_1), Some(path_segment_2)) =
				(type_path_1.path.segments.last(), type_path_2.path.segments.last())
			{
				if let syn::PathArguments::AngleBracketed(generics_) = &path_segment_1.arguments {
					if let Some(GenericArgument::Const(Expr::Lit(lit))) = generics_.args.first() {
						if let Lit::Int(int) = &lit.lit {
							if let Ok(precompile_id) = int.base10_parse() {
								if &path_segment_2.ident.to_string() == "CollectivePrecompile" {
									if let Some(instance_ident) =
										precompile_instance_ident(path_segment_2)
									{
										return Some((instance_ident, precompile_id));
									}
								} else {
									return Some((path_segment_2.ident.clone(), precompile_id));
								}
							}
						}
					}
				}
			}
		}
	}

	None
}

fn precompile_instance_ident(path_segment: &syn::PathSegment) -> Option<Ident> {
	if let syn::PathArguments::AngleBracketed(generics_) = &path_segment.arguments {
		if let Some(GenericArgument::Type(Type::Path(instance_type_path))) = generics_.args.last() {
			if let Some(instance_type) = instance_type_path.path.segments.last() {
				return Some(instance_type.ident.clone());
			}
		}
	}

	None
}
