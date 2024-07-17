// Copyright 2022 ComposableFi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::State;
use quote::quote;

impl State {
	pub(crate) fn impl_fn_downcast(&self) -> proc_macro2::TokenStream {
		let cases = self.clients.iter().map(|client| {
			let variant_ident = &client.variant_ident;
			let attrs = &client.attrs;
			quote! {
				#(#attrs)*
				Self::#variant_ident(state) => state.downcast::<T>(),
			}
		});

		quote! {
			fn downcast<T: ::core::clone::Clone + ::core::any::Any>(self) -> ::core::option::Option<T> {
				match self {
					#(#cases)*
				}
			}
		}
	}

	pub(crate) fn impl_fn_wrap(&self) -> proc_macro2::TokenStream {
		let cases = self.clients.iter().map(|client| {
			let variant_ident = &client.variant_ident;
			let attrs = &client.attrs;
			let client_state_type = &client.inner_ty_path;
			quote! {
				#(#attrs)*
				if let Some(state) = sub_state.downcast_ref::<#client_state_type>() {
					return Some(Self::#variant_ident(state.clone()));
				}
			}
		});

		quote! {
			fn wrap(sub_state: &dyn ::core::any::Any) -> ::core::option::Option<Self> {
				#(#cases)*
				None
			}
		}
	}

	pub(crate) fn impl_fn_encode_to_vec(&self) -> proc_macro2::TokenStream {
		let crate_ = &self.crate_ident;
		quote! {
			fn encode_to_vec(&self) -> ::core::result::Result<::alloc::vec::Vec<u8>, ::tendermint_proto::Error> {
				#crate_::protobuf::Protobuf::encode_vec(self)
			}
		}
	}
}
