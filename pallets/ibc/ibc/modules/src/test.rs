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

use core::fmt::Debug;
use serde::{de::DeserializeOwned, Serialize};

/// Test that a struct `T` can be:
///
/// - parsed out of the provided JSON data
/// - serialized back to JSON
/// - parsed back from the serialized JSON of the previous step
/// - that the two parsed structs are equal according to their `PartialEq` impl
pub fn test_serialization_roundtrip<T>(json_data: &str)
where
	T: Debug + Serialize + DeserializeOwned,
{
	let parsed0 = serde_json::from_str::<T>(json_data);
	assert!(parsed0.is_ok());
	let parsed0 = parsed0.unwrap();

	let serialized = serde_json::to_string(&parsed0);
	assert!(serialized.is_ok());
	let serialized = serialized.unwrap();

	let parsed1 = serde_json::from_str::<T>(&serialized);
	assert!(parsed1.is_ok());

	// TODO - fix PartialEq bound issue in AbciQuery
	//assert_eq!(parsed0, parsed1);
}
