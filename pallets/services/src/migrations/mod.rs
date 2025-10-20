// This file is part of Tangle.
// Copyright (C) 2022-2024 Tangle Foundation.
//
// Tangle is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Tangle is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Tangle.  If not, see <http://www.gnu.org/licenses/>.

//! # Services Pallet Migrations
//!
//! This module contains all storage migrations for the Services pallet.
//! Each migration is in a separate dated file for clarity and maintainability.

// No migrations needed yet - ServiceMetadata profiling_data field is backward compatible
// See analysis in /tmp/services-metadata-migration-analysis.md

pub use frame_support::{traits::StorageVersion, weights::Weight};
