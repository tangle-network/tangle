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

use sha3::{Digest, Keccak256};

#[test]
fn test_keccak256() {
	assert_eq!(&precompile_utils_macro::keccak256!(""), Keccak256::digest(b"").as_slice(),);
	assert_eq!(
		&precompile_utils_macro::keccak256!("toto()"),
		Keccak256::digest(b"toto()").as_slice(),
	);
	assert_ne!(
		&precompile_utils_macro::keccak256!("toto()"),
		Keccak256::digest(b"tata()").as_slice(),
	);
}
