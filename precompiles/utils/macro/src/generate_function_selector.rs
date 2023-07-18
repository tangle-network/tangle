// Copyright 2022 Webb Technologies Inc.
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

pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
	let item = parse_macro_input!(input as ItemEnum);

	let ItemEnum {
		attrs,
		vis,
		enum_token,
		ident,
		variants,
		..
	} = item;

	let mut ident_expressions: Vec<Ident> = vec![];
	let mut variant_expressions: Vec<Expr> = vec![];
	let mut variant_attrs: Vec<Vec<Attribute>> = vec![];
	for variant in variants {
		match variant.discriminant {
			Some((_, Expr::Lit(ExprLit { lit, .. }))) => {
				if let Lit::Str(lit_str) = lit {
					let digest = Keccak256::digest(lit_str.value().as_bytes());
					let selector = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]);
					ident_expressions.push(variant.ident);
					variant_expressions.push(Expr::Lit(ExprLit {
						lit: Lit::Verbatim(Literal::u32_suffixed(selector)),
						attrs: Default::default(),
					}));
					variant_attrs.push(variant.attrs);
				} else {
					return quote_spanned! {
						lit.span() => compile_error!("Expected literal string");
					}
					.into();
				}
			}
			Some((_eg, expr)) => {
				return quote_spanned! {
					expr.span() => compile_error!("Expected literal");
				}
				.into()
			}
			None => {
				return quote_spanned! {
					variant.span() => compile_error!("Each variant must have a discriminant");
				}
				.into()
			}
		}
	}

	(quote! {
		#(#attrs)*
		#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive)]
		#[repr(u32)]
		#vis #enum_token #ident {
			#(
				#(#variant_attrs)*
				#ident_expressions = #variant_expressions,
			)*
		}
	})
	.into()
}
