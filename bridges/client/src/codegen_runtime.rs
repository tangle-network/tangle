#[allow(dead_code, unused_imports, non_camel_case_types)]
#[allow(clippy::all)]
#[allow(rustdoc::broken_intra_doc_links)]
pub mod api {
	#[allow(unused_imports)]
	mod root_mod {
		pub use super::*;
	}
	pub mod runtime_types {
		use super::runtime_types;
		pub mod bounded_collections {
			use super::runtime_types;
			pub mod bounded_btree_map {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BoundedBTreeMap<_0, _1>(
					pub ::subxt::ext::subxt_core::utils::KeyedVec<_0, _1>,
				);
			}
			pub mod bounded_vec {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BoundedVec<_0>(pub ::subxt::ext::subxt_core::alloc::vec::Vec<_0>);
			}
			pub mod weak_bounded_vec {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct WeakBoundedVec<_0>(pub ::subxt::ext::subxt_core::alloc::vec::Vec<_0>);
			}
		}
		pub mod finality_grandpa {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Equivocation<_0, _1, _2> {
				pub round_number: ::core::primitive::u64,
				pub identity: _0,
				pub first: (_1, _2),
				pub second: (_1, _2),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Precommit<_0, _1> {
				pub target_hash: _0,
				pub target_number: _1,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Prevote<_0, _1> {
				pub target_hash: _0,
				pub target_number: _1,
			}
		}
		pub mod frame_metadata_hash_extension {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct CheckMetadataHash {
				pub mode: runtime_types::frame_metadata_hash_extension::Mode,
			}
			#[derive(
				::codec::Decode, ::codec::Encode, Clone, Debug, Eq, PartialEq, scale_info::TypeInfo,
			)]
			pub enum Mode {
				#[codec(index = 0)]
				Disabled,
				#[codec(index = 1)]
				Enabled,
			}
		}
		pub mod frame_support {
			use super::runtime_types;
			pub mod dispatch {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum DispatchClass {
					#[codec(index = 0)]
					Normal,
					#[codec(index = 1)]
					Operational,
					#[codec(index = 2)]
					Mandatory,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct DispatchInfo {
					pub weight: ::sp_weights::Weight,
					pub class: runtime_types::frame_support::dispatch::DispatchClass,
					pub pays_fee: runtime_types::frame_support::dispatch::Pays,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Pays {
					#[codec(index = 0)]
					Yes,
					#[codec(index = 1)]
					No,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PerDispatchClass<_0> {
					pub normal: _0,
					pub operational: _0,
					pub mandatory: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PostDispatchInfo {
					pub actual_weight: ::core::option::Option<::sp_weights::Weight>,
					pub pays_fee: runtime_types::frame_support::dispatch::Pays,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum RawOrigin<_0> {
					#[codec(index = 0)]
					Root,
					#[codec(index = 1)]
					Signed(_0),
					#[codec(index = 2)]
					None,
				}
			}
			pub mod traits {
				use super::runtime_types;
				pub mod messages {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum ProcessMessageError {
						#[codec(index = 0)]
						BadFormat,
						#[codec(index = 1)]
						Corrupt,
						#[codec(index = 2)]
						Unsupported,
						#[codec(index = 3)]
						Overweight(::sp_weights::Weight),
						#[codec(index = 4)]
						Yield,
						#[codec(index = 5)]
						StackLimitReached,
					}
				}
				pub mod preimages {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Bounded<_0, _1> {
						#[codec(index = 0)]
						Legacy {
							hash: ::subxt::ext::subxt_core::utils::H256,
						},
						#[codec(index = 1)]
						Inline(
							runtime_types::bounded_collections::bounded_vec::BoundedVec<
								::core::primitive::u8,
							>,
						),
						#[codec(index = 2)]
						Lookup {
							hash: ::subxt::ext::subxt_core::utils::H256,
							len: ::core::primitive::u32,
						},
						__Ignore(::core::marker::PhantomData<(_0, _1)>),
					}
				}
				pub mod schedule {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum DispatchTime<_0> {
						#[codec(index = 0)]
						At(_0),
						#[codec(index = 1)]
						After(_0),
					}
				}
				pub mod tokens {
					use super::runtime_types;
					pub mod fungible {
						use super::runtime_types;
						#[derive(
							::codec::Decode,
							::codec::Encode,
							::subxt::ext::subxt_core::ext::codec::CompactAs,
							Clone,
							Debug,
							PartialEq,
						)]
						pub struct HoldConsideration(pub ::core::primitive::u128);
					}
					pub mod misc {
						use super::runtime_types;
						#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
						pub enum BalanceStatus {
							#[codec(index = 0)]
							Free,
							#[codec(index = 1)]
							Reserved,
						}
						#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
						pub struct IdAmount<_0, _1> {
							pub id: _0,
							pub amount: _1,
						}
					}
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct PalletId(pub [::core::primitive::u8; 8usize]);
		}
		pub mod frame_system {
			use super::runtime_types;
			pub mod extensions {
				use super::runtime_types;
				pub mod check_genesis {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckGenesis;
				}
				pub mod check_mortality {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckMortality(pub ::sp_runtime::generic::Era);
				}
				pub mod check_non_zero_sender {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckNonZeroSender;
				}
				pub mod check_nonce {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckNonce(#[codec(compact)] pub ::core::primitive::u32);
				}
				pub mod check_spec_version {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckSpecVersion;
				}
				pub mod check_tx_version {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckTxVersion;
				}
				pub mod check_weight {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CheckWeight;
				}
			}
			pub mod limits {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BlockLength {
					pub max: runtime_types::frame_support::dispatch::PerDispatchClass<
						::core::primitive::u32,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BlockWeights {
					pub base_block: ::sp_weights::Weight,
					pub max_block: ::sp_weights::Weight,
					pub per_class: runtime_types::frame_support::dispatch::PerDispatchClass<
						runtime_types::frame_system::limits::WeightsPerClass,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct WeightsPerClass {
					pub base_extrinsic: ::sp_weights::Weight,
					pub max_extrinsic: ::core::option::Option<::sp_weights::Weight>,
					pub max_total: ::core::option::Option<::sp_weights::Weight>,
					pub reserved: ::core::option::Option<::sp_weights::Weight>,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					remark {
						remark: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					set_heap_pages { pages: ::core::primitive::u64 },
					#[codec(index = 2)]
					set_code {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 3)]
					set_code_without_checks {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 4)]
					set_storage {
						items: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						)>,
					},
					#[codec(index = 5)]
					kill_storage {
						keys: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					},
					#[codec(index = 6)]
					kill_prefix {
						prefix: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						subkeys: ::core::primitive::u32,
					},
					#[codec(index = 7)]
					remark_with_event {
						remark: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 9)]
					authorize_upgrade { code_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 10)]
					authorize_upgrade_without_checks {
						code_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 11)]
					apply_authorized_upgrade {
						code: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					InvalidSpecName,
					#[codec(index = 1)]
					SpecVersionNeedsToIncrease,
					#[codec(index = 2)]
					FailedToExtractRuntimeVersion,
					#[codec(index = 3)]
					NonDefaultComposite,
					#[codec(index = 4)]
					NonZeroRefCount,
					#[codec(index = 5)]
					CallFiltered,
					#[codec(index = 6)]
					MultiBlockMigrationsOngoing,
					#[codec(index = 7)]
					NothingAuthorized,
					#[codec(index = 8)]
					Unauthorized,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					ExtrinsicSuccess {
						dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
					},
					#[codec(index = 1)]
					ExtrinsicFailed {
						dispatch_error: runtime_types::sp_runtime::DispatchError,
						dispatch_info: runtime_types::frame_support::dispatch::DispatchInfo,
					},
					#[codec(index = 2)]
					CodeUpdated,
					#[codec(index = 3)]
					NewAccount { account: ::sp_core::crypto::AccountId32 },
					#[codec(index = 4)]
					KilledAccount { account: ::sp_core::crypto::AccountId32 },
					#[codec(index = 5)]
					Remarked {
						sender: ::sp_core::crypto::AccountId32,
						hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 6)]
					UpgradeAuthorized {
						code_hash: ::subxt::ext::subxt_core::utils::H256,
						check_version: ::core::primitive::bool,
					},
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct AccountInfo<_0, _1> {
				pub nonce: _0,
				pub consumers: ::core::primitive::u32,
				pub providers: ::core::primitive::u32,
				pub sufficients: ::core::primitive::u32,
				pub data: _1,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct CodeUpgradeAuthorization {
				pub code_hash: ::subxt::ext::subxt_core::utils::H256,
				pub check_version: ::core::primitive::bool,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct EventRecord<_0, _1> {
				pub phase: runtime_types::frame_system::Phase,
				pub event: _0,
				pub topics: ::subxt::ext::subxt_core::alloc::vec::Vec<_1>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct LastRuntimeUpgradeInfo {
				#[codec(compact)]
				pub spec_version: ::core::primitive::u32,
				pub spec_name: ::subxt::ext::subxt_core::alloc::string::String,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Phase {
				#[codec(index = 0)]
				ApplyExtrinsic(::core::primitive::u32),
				#[codec(index = 1)]
				Finalization,
				#[codec(index = 2)]
				Initialization,
			}
		}
		pub mod pallet_asset_rate {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					create {
						asset_kind: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						>,
						rate: runtime_types::sp_arithmetic::fixed_point::FixedU128,
					},
					#[codec(index = 1)]
					update {
						asset_kind: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						>,
						rate: runtime_types::sp_arithmetic::fixed_point::FixedU128,
					},
					#[codec(index = 2)]
					remove {
						asset_kind: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					UnknownAssetKind,
					#[codec(index = 1)]
					AlreadyExists,
					#[codec(index = 2)]
					Overflow,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					AssetRateCreated {
						asset_kind:
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						rate: runtime_types::sp_arithmetic::fixed_point::FixedU128,
					},
					#[codec(index = 1)]
					AssetRateRemoved {
						asset_kind:
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
					},
					#[codec(index = 2)]
					AssetRateUpdated {
						asset_kind:
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						old: runtime_types::sp_arithmetic::fixed_point::FixedU128,
						new: runtime_types::sp_arithmetic::fixed_point::FixedU128,
					},
				}
			}
		}
		pub mod pallet_babe {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					report_equivocation {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_slots::EquivocationProof<
								::sp_runtime::generic::Header<
									::core::primitive::u32,
									::sp_runtime::traits::BlakeTwo256,
								>,
								runtime_types::sp_consensus_babe::app::Public,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 1)]
					report_equivocation_unsigned {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_slots::EquivocationProof<
								::sp_runtime::generic::Header<
									::core::primitive::u32,
									::sp_runtime::traits::BlakeTwo256,
								>,
								runtime_types::sp_consensus_babe::app::Public,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 2)]
					plan_config_change {
						config: runtime_types::sp_consensus_babe::digests::NextConfigDescriptor,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					InvalidEquivocationProof,
					#[codec(index = 1)]
					InvalidKeyOwnershipProof,
					#[codec(index = 2)]
					DuplicateOffenceReport,
					#[codec(index = 3)]
					InvalidConfiguration,
				}
			}
		}
		pub mod pallet_bags_list {
			use super::runtime_types;
			pub mod list {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Bag {
					pub head: ::core::option::Option<::sp_core::crypto::AccountId32>,
					pub tail: ::core::option::Option<::sp_core::crypto::AccountId32>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum ListError {
					#[codec(index = 0)]
					Duplicate,
					#[codec(index = 1)]
					NotHeavier,
					#[codec(index = 2)]
					NotInSameBag,
					#[codec(index = 3)]
					NodeNotFound,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Node {
					pub id: ::sp_core::crypto::AccountId32,
					pub prev: ::core::option::Option<::sp_core::crypto::AccountId32>,
					pub next: ::core::option::Option<::sp_core::crypto::AccountId32>,
					pub bag_upper: ::core::primitive::u64,
					pub score: ::core::primitive::u64,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					rebag {
						dislocated: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 1)]
					put_in_front_of {
						lighter: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 2)]
					put_in_front_of_other {
						heavier: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						lighter: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					List(runtime_types::pallet_bags_list::list::ListError),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Rebagged {
						who: ::sp_core::crypto::AccountId32,
						from: ::core::primitive::u64,
						to: ::core::primitive::u64,
					},
					#[codec(index = 1)]
					ScoreUpdated {
						who: ::sp_core::crypto::AccountId32,
						new_score: ::core::primitive::u64,
					},
				}
			}
		}
		pub mod pallet_balances {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					transfer_allow_death {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					force_transfer {
						source: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					transfer_keep_alive {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						#[codec(compact)]
						value: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					transfer_all {
						dest: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						keep_alive: ::core::primitive::bool,
					},
					#[codec(index = 5)]
					force_unreserve {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 6)]
					upgrade_accounts {
						who: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
					},
					#[codec(index = 8)]
					force_set_balance {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						#[codec(compact)]
						new_free: ::core::primitive::u128,
					},
					#[codec(index = 9)]
					force_adjust_total_issuance {
						direction: runtime_types::pallet_balances::types::AdjustmentDirection,
						#[codec(compact)]
						delta: ::core::primitive::u128,
					},
					#[codec(index = 10)]
					burn {
						#[codec(compact)]
						value: ::core::primitive::u128,
						keep_alive: ::core::primitive::bool,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					VestingBalance,
					#[codec(index = 1)]
					LiquidityRestrictions,
					#[codec(index = 2)]
					InsufficientBalance,
					#[codec(index = 3)]
					ExistentialDeposit,
					#[codec(index = 4)]
					Expendability,
					#[codec(index = 5)]
					ExistingVestingSchedule,
					#[codec(index = 6)]
					DeadAccount,
					#[codec(index = 7)]
					TooManyReserves,
					#[codec(index = 8)]
					TooManyHolds,
					#[codec(index = 9)]
					TooManyFreezes,
					#[codec(index = 10)]
					IssuanceDeactivated,
					#[codec(index = 11)]
					DeltaZero,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Endowed {
						account: ::sp_core::crypto::AccountId32,
						free_balance: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					DustLost {
						account: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					Transfer {
						from: ::sp_core::crypto::AccountId32,
						to: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					BalanceSet {
						who: ::sp_core::crypto::AccountId32,
						free: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					Reserved {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					Unreserved {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 6)]
					ReserveRepatriated {
						from: ::sp_core::crypto::AccountId32,
						to: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
						destination_status:
							runtime_types::frame_support::traits::tokens::misc::BalanceStatus,
					},
					#[codec(index = 7)]
					Deposit { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 8)]
					Withdraw {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 9)]
					Slashed { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 10)]
					Minted { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 11)]
					Burned { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 12)]
					Suspended {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 13)]
					Restored {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 14)]
					Upgraded { who: ::sp_core::crypto::AccountId32 },
					#[codec(index = 15)]
					Issued { amount: ::core::primitive::u128 },
					#[codec(index = 16)]
					Rescinded { amount: ::core::primitive::u128 },
					#[codec(index = 17)]
					Locked { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 18)]
					Unlocked {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 19)]
					Frozen { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 20)]
					Thawed { who: ::sp_core::crypto::AccountId32, amount: ::core::primitive::u128 },
					#[codec(index = 21)]
					TotalIssuanceForced {
						old: ::core::primitive::u128,
						new: ::core::primitive::u128,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AccountData<_0> {
					pub free: _0,
					pub reserved: _0,
					pub frozen: _0,
					pub flags: runtime_types::pallet_balances::types::ExtraFlags,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum AdjustmentDirection {
					#[codec(index = 0)]
					Increase,
					#[codec(index = 1)]
					Decrease,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BalanceLock<_0> {
					pub id: [::core::primitive::u8; 8usize],
					pub amount: _0,
					pub reasons: runtime_types::pallet_balances::types::Reasons,
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct ExtraFlags(pub ::core::primitive::u128);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Reasons {
					#[codec(index = 0)]
					Fee,
					#[codec(index = 1)]
					Misc,
					#[codec(index = 2)]
					All,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ReserveData<_0, _1> {
					pub id: _0,
					pub amount: _1,
				}
			}
		}
		pub mod pallet_beefy {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					report_double_voting {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::DoubleVotingProof<
								::core::primitive::u32,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Signature,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 1)]
					report_double_voting_unsigned {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::DoubleVotingProof<
								::core::primitive::u32,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Signature,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 2)]
					set_new_genesis { delay_in_blocks: ::core::primitive::u32 },
					#[codec(index = 3)]
					report_fork_voting {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::ForkVotingProof<
								::sp_runtime::generic::Header<
									::core::primitive::u32,
									::sp_runtime::traits::BlakeTwo256,
								>,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
								runtime_types::sp_mmr_primitives::AncestryProof<
									::subxt::ext::subxt_core::utils::H256,
								>,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 4)]
					report_fork_voting_unsigned {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::ForkVotingProof<
								::sp_runtime::generic::Header<
									::core::primitive::u32,
									::sp_runtime::traits::BlakeTwo256,
								>,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
								runtime_types::sp_mmr_primitives::AncestryProof<
									::subxt::ext::subxt_core::utils::H256,
								>,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 5)]
					report_future_block_voting {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::FutureBlockVotingProof<
								::core::primitive::u32,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 6)]
					report_future_block_voting_unsigned {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::sp_consensus_beefy::FutureBlockVotingProof<
								::core::primitive::u32,
								runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					InvalidKeyOwnershipProof,
					#[codec(index = 1)]
					InvalidDoubleVotingProof,
					#[codec(index = 2)]
					InvalidForkVotingProof,
					#[codec(index = 3)]
					InvalidFutureBlockVotingProof,
					#[codec(index = 4)]
					InvalidEquivocationProofSession,
					#[codec(index = 5)]
					DuplicateOffenceReport,
					#[codec(index = 6)]
					InvalidConfiguration,
				}
			}
		}
		pub mod pallet_broker {
			use super::runtime_types;
			pub mod coretime_interface {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum CoreAssignment {
					#[codec(index = 0)]
					Idle,
					#[codec(index = 1)]
					Pool,
					#[codec(index = 2)]
					Task(::core::primitive::u32),
				}
			}
		}
		pub mod pallet_conviction_voting {
			use super::runtime_types;
			pub mod conviction {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Conviction {
					#[codec(index = 0)]
					None,
					#[codec(index = 1)]
					Locked1x,
					#[codec(index = 2)]
					Locked2x,
					#[codec(index = 3)]
					Locked3x,
					#[codec(index = 4)]
					Locked4x,
					#[codec(index = 5)]
					Locked5x,
					#[codec(index = 6)]
					Locked6x,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					vote {
						#[codec(compact)]
						poll_index: ::core::primitive::u32,
						vote: runtime_types::pallet_conviction_voting::vote::AccountVote<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 1)]
					delegate {
						class: ::core::primitive::u16,
						to: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						conviction: runtime_types::pallet_conviction_voting::conviction::Conviction,
						balance: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					undelegate { class: ::core::primitive::u16 },
					#[codec(index = 3)]
					unlock {
						class: ::core::primitive::u16,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 4)]
					remove_vote {
						class: ::core::option::Option<::core::primitive::u16>,
						index: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					remove_other_vote {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						class: ::core::primitive::u16,
						index: ::core::primitive::u32,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotOngoing,
					#[codec(index = 1)]
					NotVoter,
					#[codec(index = 2)]
					NoPermission,
					#[codec(index = 3)]
					NoPermissionYet,
					#[codec(index = 4)]
					AlreadyDelegating,
					#[codec(index = 5)]
					AlreadyVoting,
					#[codec(index = 6)]
					InsufficientFunds,
					#[codec(index = 7)]
					NotDelegating,
					#[codec(index = 8)]
					Nonsense,
					#[codec(index = 9)]
					MaxVotesReached,
					#[codec(index = 10)]
					ClassNeeded,
					#[codec(index = 11)]
					BadClass,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Delegated(::sp_core::crypto::AccountId32, ::sp_core::crypto::AccountId32),
					#[codec(index = 1)]
					Undelegated(::sp_core::crypto::AccountId32),
					#[codec(index = 2)]
					Voted {
						who: ::sp_core::crypto::AccountId32,
						vote: runtime_types::pallet_conviction_voting::vote::AccountVote<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 3)]
					VoteRemoved {
						who: ::sp_core::crypto::AccountId32,
						vote: runtime_types::pallet_conviction_voting::vote::AccountVote<
							::core::primitive::u128,
						>,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Delegations<_0> {
					pub votes: _0,
					pub capital: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Tally<_0> {
					pub ayes: _0,
					pub nays: _0,
					pub support: _0,
				}
			}
			pub mod vote {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum AccountVote<_0> {
					#[codec(index = 0)]
					Standard {
						vote: runtime_types::pallet_conviction_voting::vote::Vote,
						balance: _0,
					},
					#[codec(index = 1)]
					Split { aye: _0, nay: _0 },
					#[codec(index = 2)]
					SplitAbstain { aye: _0, nay: _0, abstain: _0 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Casting<_0, _1, _2> {
					pub votes: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						_1,
						runtime_types::pallet_conviction_voting::vote::AccountVote<_0>,
					)>,
					pub delegations:
						runtime_types::pallet_conviction_voting::types::Delegations<_0>,
					pub prior: runtime_types::pallet_conviction_voting::vote::PriorLock<_1, _0>,
					#[codec(skip)]
					pub __ignore: ::core::marker::PhantomData<_2>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Delegating<_0, _1, _2> {
					pub balance: _0,
					pub target: _1,
					pub conviction: runtime_types::pallet_conviction_voting::conviction::Conviction,
					pub delegations:
						runtime_types::pallet_conviction_voting::types::Delegations<_0>,
					pub prior: runtime_types::pallet_conviction_voting::vote::PriorLock<_2, _0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PriorLock<_0, _1>(pub _0, pub _1);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Vote(pub ::core::primitive::u8);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Voting<_0, _1, _2, _3> {
					#[codec(index = 0)]
					Casting(runtime_types::pallet_conviction_voting::vote::Casting<_0, _2, _2>),
					#[codec(index = 1)]
					Delegating(
						runtime_types::pallet_conviction_voting::vote::Delegating<_0, _1, _2>,
					),
					__Ignore(::core::marker::PhantomData<_3>),
				}
			}
		}
		pub mod pallet_delegated_staking {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotAllowed,
					#[codec(index = 1)]
					AlreadyStaking,
					#[codec(index = 2)]
					InvalidRewardDestination,
					#[codec(index = 3)]
					InvalidDelegation,
					#[codec(index = 4)]
					NotEnoughFunds,
					#[codec(index = 5)]
					NotAgent,
					#[codec(index = 6)]
					NotDelegator,
					#[codec(index = 7)]
					BadState,
					#[codec(index = 8)]
					UnappliedSlash,
					#[codec(index = 9)]
					NothingToSlash,
					#[codec(index = 10)]
					WithdrawFailed,
					#[codec(index = 11)]
					NotSupported,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Delegated {
						agent: ::sp_core::crypto::AccountId32,
						delegator: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					Released {
						agent: ::sp_core::crypto::AccountId32,
						delegator: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					Slashed {
						agent: ::sp_core::crypto::AccountId32,
						delegator: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					MigratedDelegation {
						agent: ::sp_core::crypto::AccountId32,
						delegator: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum HoldReason {
					#[codec(index = 0)]
					StakingDelegation,
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AgentLedger {
					pub payee: ::sp_core::crypto::AccountId32,
					#[codec(compact)]
					pub total_delegated: ::core::primitive::u128,
					#[codec(compact)]
					pub unclaimed_withdrawals: ::core::primitive::u128,
					#[codec(compact)]
					pub pending_slash: ::core::primitive::u128,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Delegation {
					pub agent: ::sp_core::crypto::AccountId32,
					pub amount: ::core::primitive::u128,
				}
			}
		}
		pub mod pallet_election_provider_multi_phase {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
                    submit_unsigned {
                        raw_solution: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::pallet_election_provider_multi_phase::RawSolution<
                                runtime_types::westend_runtime::NposCompactSolution16,
                            >,
                        >,
                        witness: runtime_types::pallet_election_provider_multi_phase::SolutionOrSnapshotSize,
                    },
                    #[codec(index = 1)]
                    set_minimum_untrusted_score {
                        maybe_next_score: ::core::option::Option<
                            runtime_types::sp_npos_elections::ElectionScore,
                        >,
                    },
                    #[codec(index = 2)]
                    set_emergency_election_result {
                        supports: ::subxt::ext::subxt_core::alloc::vec::Vec<
                            (
                                ::sp_core::crypto::AccountId32,
                                runtime_types::sp_npos_elections::Support<
                                    ::sp_core::crypto::AccountId32,
                                >,
                            ),
                        >,
                    },
                    #[codec(index = 3)]
                    submit {
                        raw_solution: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::pallet_election_provider_multi_phase::RawSolution<
                                runtime_types::westend_runtime::NposCompactSolution16,
                            >,
                        >,
                    },
                    #[codec(index = 4)]
                    governance_fallback {
                        maybe_max_voters: ::core::option::Option<::core::primitive::u32>,
                        maybe_max_targets: ::core::option::Option<::core::primitive::u32>,
                    },
                }
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					PreDispatchEarlySubmission,
					#[codec(index = 1)]
					PreDispatchWrongWinnerCount,
					#[codec(index = 2)]
					PreDispatchWeakSubmission,
					#[codec(index = 3)]
					SignedQueueFull,
					#[codec(index = 4)]
					SignedCannotPayDeposit,
					#[codec(index = 5)]
					SignedInvalidWitness,
					#[codec(index = 6)]
					SignedTooMuchWeight,
					#[codec(index = 7)]
					OcwCallWrongEra,
					#[codec(index = 8)]
					MissingSnapshotMetadata,
					#[codec(index = 9)]
					InvalidSubmissionIndex,
					#[codec(index = 10)]
					CallNotAllowed,
					#[codec(index = 11)]
					FallbackFailed,
					#[codec(index = 12)]
					BoundNotMet,
					#[codec(index = 13)]
					TooManyWinners,
					#[codec(index = 14)]
					PreDispatchDifferentRound,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					SolutionStored {
						compute:
							runtime_types::pallet_election_provider_multi_phase::ElectionCompute,
						origin: ::core::option::Option<::sp_core::crypto::AccountId32>,
						prev_ejected: ::core::primitive::bool,
					},
					#[codec(index = 1)]
					ElectionFinalized {
						compute:
							runtime_types::pallet_election_provider_multi_phase::ElectionCompute,
						score: runtime_types::sp_npos_elections::ElectionScore,
					},
					#[codec(index = 2)]
					ElectionFailed,
					#[codec(index = 3)]
					Rewarded {
						account: ::sp_core::crypto::AccountId32,
						value: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					Slashed {
						account: ::sp_core::crypto::AccountId32,
						value: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					PhaseTransitioned {
						from: runtime_types::pallet_election_provider_multi_phase::Phase<
							::core::primitive::u32,
						>,
						to: runtime_types::pallet_election_provider_multi_phase::Phase<
							::core::primitive::u32,
						>,
						round: ::core::primitive::u32,
					},
				}
			}
			pub mod signed {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SignedSubmission<_0, _1, _2> {
					pub who: _0,
					pub deposit: _1,
					pub raw_solution:
						runtime_types::pallet_election_provider_multi_phase::RawSolution<_2>,
					pub call_fee: _1,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ElectionCompute {
				#[codec(index = 0)]
				OnChain,
				#[codec(index = 1)]
				Signed,
				#[codec(index = 2)]
				Unsigned,
				#[codec(index = 3)]
				Fallback,
				#[codec(index = 4)]
				Emergency,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Phase<_0> {
				#[codec(index = 0)]
				Off,
				#[codec(index = 1)]
				Signed,
				#[codec(index = 2)]
				Unsigned((::core::primitive::bool, _0)),
				#[codec(index = 3)]
				Emergency,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RawSolution<_0> {
				pub solution: _0,
				pub score: runtime_types::sp_npos_elections::ElectionScore,
				pub round: ::core::primitive::u32,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ReadySolution {
				pub supports: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
					::sp_core::crypto::AccountId32,
					runtime_types::sp_npos_elections::Support<::sp_core::crypto::AccountId32>,
				)>,
				pub score: runtime_types::sp_npos_elections::ElectionScore,
				pub compute: runtime_types::pallet_election_provider_multi_phase::ElectionCompute,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RoundSnapshot<_0, _1> {
				pub voters: ::subxt::ext::subxt_core::alloc::vec::Vec<_1>,
				pub targets: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct SolutionOrSnapshotSize {
				#[codec(compact)]
				pub voters: ::core::primitive::u32,
				#[codec(compact)]
				pub targets: ::core::primitive::u32,
			}
		}
		pub mod pallet_fast_unstake {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					register_fast_unstake,
					#[codec(index = 1)]
					deregister,
					#[codec(index = 2)]
					control { eras_to_check: ::core::primitive::u32 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotController,
					#[codec(index = 1)]
					AlreadyQueued,
					#[codec(index = 2)]
					NotFullyBonded,
					#[codec(index = 3)]
					NotQueued,
					#[codec(index = 4)]
					AlreadyHead,
					#[codec(index = 5)]
					CallNotAllowed,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Unstaked {
						stash: ::sp_core::crypto::AccountId32,
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 1)]
					Slashed {
						stash: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					BatchChecked {
						eras: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u32>,
					},
					#[codec(index = 3)]
					BatchFinished { size: ::core::primitive::u32 },
					#[codec(index = 4)]
					InternalError,
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct UnstakeRequest {
					pub stashes: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						::sp_core::crypto::AccountId32,
						::core::primitive::u128,
					)>,
					pub checked: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u32,
					>,
				}
			}
		}
		pub mod pallet_grandpa {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					report_equivocation {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							::sp_consensus_grandpa::EquivocationProof<
								::subxt::ext::subxt_core::utils::H256,
								::core::primitive::u32,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 1)]
					report_equivocation_unsigned {
						equivocation_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
							::sp_consensus_grandpa::EquivocationProof<
								::subxt::ext::subxt_core::utils::H256,
								::core::primitive::u32,
							>,
						>,
						key_owner_proof: ::sp_session::MembershipProof,
					},
					#[codec(index = 2)]
					note_stalled {
						delay: ::core::primitive::u32,
						best_finalized_block_number: ::core::primitive::u32,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					PauseFailed,
					#[codec(index = 1)]
					ResumeFailed,
					#[codec(index = 2)]
					ChangePending,
					#[codec(index = 3)]
					TooSoon,
					#[codec(index = 4)]
					InvalidKeyOwnershipProof,
					#[codec(index = 5)]
					InvalidEquivocationProof,
					#[codec(index = 6)]
					DuplicateOffenceReport,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					NewAuthorities {
						authority_set: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							runtime_types::sp_consensus_grandpa::app::Public,
							::core::primitive::u64,
						)>,
					},
					#[codec(index = 1)]
					Paused,
					#[codec(index = 2)]
					Resumed,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct StoredPendingChange<_0> {
				pub scheduled_at: _0,
				pub delay: _0,
				pub next_authorities:
					runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<(
						runtime_types::sp_consensus_grandpa::app::Public,
						::core::primitive::u64,
					)>,
				pub forced: ::core::option::Option<_0>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum StoredState<_0> {
				#[codec(index = 0)]
				Live,
				#[codec(index = 1)]
				PendingPause { scheduled_at: _0, delay: _0 },
				#[codec(index = 2)]
				Paused,
				#[codec(index = 3)]
				PendingResume { scheduled_at: _0, delay: _0 },
			}
		}
		pub mod pallet_identity {
			use super::runtime_types;
			pub mod legacy {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct IdentityInfo {
					pub additional: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						runtime_types::pallet_identity::types::Data,
						runtime_types::pallet_identity::types::Data,
					)>,
					pub display: runtime_types::pallet_identity::types::Data,
					pub legal: runtime_types::pallet_identity::types::Data,
					pub web: runtime_types::pallet_identity::types::Data,
					pub riot: runtime_types::pallet_identity::types::Data,
					pub email: runtime_types::pallet_identity::types::Data,
					pub pgp_fingerprint: ::core::option::Option<[::core::primitive::u8; 20usize]>,
					pub image: runtime_types::pallet_identity::types::Data,
					pub twitter: runtime_types::pallet_identity::types::Data,
				}
			}
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					add_registrar {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 1)]
					set_identity {
						info: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::pallet_identity::legacy::IdentityInfo,
						>,
					},
					#[codec(index = 2)]
					set_subs {
						subs: ::subxt::ext::subxt_core::alloc::vec::Vec<(
							::sp_core::crypto::AccountId32,
							runtime_types::pallet_identity::types::Data,
						)>,
					},
					#[codec(index = 3)]
					clear_identity,
					#[codec(index = 4)]
					request_judgement {
						#[codec(compact)]
						reg_index: ::core::primitive::u32,
						#[codec(compact)]
						max_fee: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					cancel_request { reg_index: ::core::primitive::u32 },
					#[codec(index = 6)]
					set_fee {
						#[codec(compact)]
						index: ::core::primitive::u32,
						#[codec(compact)]
						fee: ::core::primitive::u128,
					},
					#[codec(index = 7)]
					set_account_id {
						#[codec(compact)]
						index: ::core::primitive::u32,
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 8)]
					set_fields {
						#[codec(compact)]
						index: ::core::primitive::u32,
						fields: ::core::primitive::u64,
					},
					#[codec(index = 9)]
					provide_judgement {
						#[codec(compact)]
						reg_index: ::core::primitive::u32,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						judgement: runtime_types::pallet_identity::types::Judgement<
							::core::primitive::u128,
						>,
						identity: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 10)]
					kill_identity {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 11)]
					add_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						data: runtime_types::pallet_identity::types::Data,
					},
					#[codec(index = 12)]
					rename_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						data: runtime_types::pallet_identity::types::Data,
					},
					#[codec(index = 13)]
					remove_sub {
						sub: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 14)]
					quit_sub,
					#[codec(index = 15)]
					add_username_authority {
						authority: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						suffix: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						allocation: ::core::primitive::u32,
					},
					#[codec(index = 16)]
					remove_username_authority {
						authority: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 17)]
					set_username_for {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						username: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						signature:
							::core::option::Option<runtime_types::sp_runtime::MultiSignature>,
					},
					#[codec(index = 18)]
					accept_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 19)]
					remove_expired_approval {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 20)]
					set_primary_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 21)]
					remove_dangling_username {
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					TooManySubAccounts,
					#[codec(index = 1)]
					NotFound,
					#[codec(index = 2)]
					NotNamed,
					#[codec(index = 3)]
					EmptyIndex,
					#[codec(index = 4)]
					FeeChanged,
					#[codec(index = 5)]
					NoIdentity,
					#[codec(index = 6)]
					StickyJudgement,
					#[codec(index = 7)]
					JudgementGiven,
					#[codec(index = 8)]
					InvalidJudgement,
					#[codec(index = 9)]
					InvalidIndex,
					#[codec(index = 10)]
					InvalidTarget,
					#[codec(index = 11)]
					TooManyRegistrars,
					#[codec(index = 12)]
					AlreadyClaimed,
					#[codec(index = 13)]
					NotSub,
					#[codec(index = 14)]
					NotOwned,
					#[codec(index = 15)]
					JudgementForDifferentIdentity,
					#[codec(index = 16)]
					JudgementPaymentFailed,
					#[codec(index = 17)]
					InvalidSuffix,
					#[codec(index = 18)]
					NotUsernameAuthority,
					#[codec(index = 19)]
					NoAllocation,
					#[codec(index = 20)]
					InvalidSignature,
					#[codec(index = 21)]
					RequiresSignature,
					#[codec(index = 22)]
					InvalidUsername,
					#[codec(index = 23)]
					UsernameTaken,
					#[codec(index = 24)]
					NoUsername,
					#[codec(index = 25)]
					NotExpired,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					IdentitySet { who: ::sp_core::crypto::AccountId32 },
					#[codec(index = 1)]
					IdentityCleared {
						who: ::sp_core::crypto::AccountId32,
						deposit: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					IdentityKilled {
						who: ::sp_core::crypto::AccountId32,
						deposit: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					JudgementRequested {
						who: ::sp_core::crypto::AccountId32,
						registrar_index: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					JudgementUnrequested {
						who: ::sp_core::crypto::AccountId32,
						registrar_index: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					JudgementGiven {
						target: ::sp_core::crypto::AccountId32,
						registrar_index: ::core::primitive::u32,
					},
					#[codec(index = 6)]
					RegistrarAdded { registrar_index: ::core::primitive::u32 },
					#[codec(index = 7)]
					SubIdentityAdded {
						sub: ::sp_core::crypto::AccountId32,
						main: ::sp_core::crypto::AccountId32,
						deposit: ::core::primitive::u128,
					},
					#[codec(index = 8)]
					SubIdentityRemoved {
						sub: ::sp_core::crypto::AccountId32,
						main: ::sp_core::crypto::AccountId32,
						deposit: ::core::primitive::u128,
					},
					#[codec(index = 9)]
					SubIdentityRevoked {
						sub: ::sp_core::crypto::AccountId32,
						main: ::sp_core::crypto::AccountId32,
						deposit: ::core::primitive::u128,
					},
					#[codec(index = 10)]
					AuthorityAdded { authority: ::sp_core::crypto::AccountId32 },
					#[codec(index = 11)]
					AuthorityRemoved { authority: ::sp_core::crypto::AccountId32 },
					#[codec(index = 12)]
					UsernameSet {
						who: ::sp_core::crypto::AccountId32,
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 13)]
					UsernameQueued {
						who: ::sp_core::crypto::AccountId32,
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
						expiration: ::core::primitive::u32,
					},
					#[codec(index = 14)]
					PreapprovalExpired { whose: ::sp_core::crypto::AccountId32 },
					#[codec(index = 15)]
					PrimaryUsernameSet {
						who: ::sp_core::crypto::AccountId32,
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
					#[codec(index = 16)]
					DanglingUsernameRemoved {
						who: ::sp_core::crypto::AccountId32,
						username: runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AuthorityProperties<_0> {
					pub suffix: _0,
					pub allocation: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Data {
					#[codec(index = 0)]
					None,
					#[codec(index = 1)]
					Raw0([::core::primitive::u8; 0usize]),
					#[codec(index = 2)]
					Raw1([::core::primitive::u8; 1usize]),
					#[codec(index = 3)]
					Raw2([::core::primitive::u8; 2usize]),
					#[codec(index = 4)]
					Raw3([::core::primitive::u8; 3usize]),
					#[codec(index = 5)]
					Raw4([::core::primitive::u8; 4usize]),
					#[codec(index = 6)]
					Raw5([::core::primitive::u8; 5usize]),
					#[codec(index = 7)]
					Raw6([::core::primitive::u8; 6usize]),
					#[codec(index = 8)]
					Raw7([::core::primitive::u8; 7usize]),
					#[codec(index = 9)]
					Raw8([::core::primitive::u8; 8usize]),
					#[codec(index = 10)]
					Raw9([::core::primitive::u8; 9usize]),
					#[codec(index = 11)]
					Raw10([::core::primitive::u8; 10usize]),
					#[codec(index = 12)]
					Raw11([::core::primitive::u8; 11usize]),
					#[codec(index = 13)]
					Raw12([::core::primitive::u8; 12usize]),
					#[codec(index = 14)]
					Raw13([::core::primitive::u8; 13usize]),
					#[codec(index = 15)]
					Raw14([::core::primitive::u8; 14usize]),
					#[codec(index = 16)]
					Raw15([::core::primitive::u8; 15usize]),
					#[codec(index = 17)]
					Raw16([::core::primitive::u8; 16usize]),
					#[codec(index = 18)]
					Raw17([::core::primitive::u8; 17usize]),
					#[codec(index = 19)]
					Raw18([::core::primitive::u8; 18usize]),
					#[codec(index = 20)]
					Raw19([::core::primitive::u8; 19usize]),
					#[codec(index = 21)]
					Raw20([::core::primitive::u8; 20usize]),
					#[codec(index = 22)]
					Raw21([::core::primitive::u8; 21usize]),
					#[codec(index = 23)]
					Raw22([::core::primitive::u8; 22usize]),
					#[codec(index = 24)]
					Raw23([::core::primitive::u8; 23usize]),
					#[codec(index = 25)]
					Raw24([::core::primitive::u8; 24usize]),
					#[codec(index = 26)]
					Raw25([::core::primitive::u8; 25usize]),
					#[codec(index = 27)]
					Raw26([::core::primitive::u8; 26usize]),
					#[codec(index = 28)]
					Raw27([::core::primitive::u8; 27usize]),
					#[codec(index = 29)]
					Raw28([::core::primitive::u8; 28usize]),
					#[codec(index = 30)]
					Raw29([::core::primitive::u8; 29usize]),
					#[codec(index = 31)]
					Raw30([::core::primitive::u8; 30usize]),
					#[codec(index = 32)]
					Raw31([::core::primitive::u8; 31usize]),
					#[codec(index = 33)]
					Raw32([::core::primitive::u8; 32usize]),
					#[codec(index = 34)]
					BlakeTwo256([::core::primitive::u8; 32usize]),
					#[codec(index = 35)]
					Sha256([::core::primitive::u8; 32usize]),
					#[codec(index = 36)]
					Keccak256([::core::primitive::u8; 32usize]),
					#[codec(index = 37)]
					ShaThree256([::core::primitive::u8; 32usize]),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Judgement<_0> {
					#[codec(index = 0)]
					Unknown,
					#[codec(index = 1)]
					FeePaid(_0),
					#[codec(index = 2)]
					Reasonable,
					#[codec(index = 3)]
					KnownGood,
					#[codec(index = 4)]
					OutOfDate,
					#[codec(index = 5)]
					LowQuality,
					#[codec(index = 6)]
					Erroneous,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct RegistrarInfo<_0, _1, _2> {
					pub account: _1,
					pub fee: _0,
					pub fields: _2,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Registration<_0, _2> {
					pub judgements: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						::core::primitive::u32,
						runtime_types::pallet_identity::types::Judgement<_0>,
					)>,
					pub deposit: _0,
					pub info: _2,
				}
			}
		}
		pub mod pallet_indices {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					claim { index: ::core::primitive::u32 },
					#[codec(index = 1)]
					transfer {
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						index: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					free { index: ::core::primitive::u32 },
					#[codec(index = 3)]
					force_transfer {
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						index: ::core::primitive::u32,
						freeze: ::core::primitive::bool,
					},
					#[codec(index = 4)]
					freeze { index: ::core::primitive::u32 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotAssigned,
					#[codec(index = 1)]
					NotOwner,
					#[codec(index = 2)]
					InUse,
					#[codec(index = 3)]
					NotTransfer,
					#[codec(index = 4)]
					Permanent,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					IndexAssigned {
						who: ::sp_core::crypto::AccountId32,
						index: ::core::primitive::u32,
					},
					#[codec(index = 1)]
					IndexFreed { index: ::core::primitive::u32 },
					#[codec(index = 2)]
					IndexFrozen {
						index: ::core::primitive::u32,
						who: ::sp_core::crypto::AccountId32,
					},
				}
			}
		}
		pub mod pallet_message_queue {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
                    reap_page {
                        message_origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        page_index: ::core::primitive::u32,
                    },
                    #[codec(index = 1)]
                    execute_overweight {
                        message_origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        page: ::core::primitive::u32,
                        index: ::core::primitive::u32,
                        weight_limit: ::sp_weights::Weight,
                    },
                }
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotReapable,
					#[codec(index = 1)]
					NoPage,
					#[codec(index = 2)]
					NoMessage,
					#[codec(index = 3)]
					AlreadyProcessed,
					#[codec(index = 4)]
					Queued,
					#[codec(index = 5)]
					InsufficientWeight,
					#[codec(index = 6)]
					TemporarilyUnprocessable,
					#[codec(index = 7)]
					QueuePaused,
					#[codec(index = 8)]
					RecursiveDisallowed,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
                    ProcessingFailed {
                        id: ::subxt::ext::subxt_core::utils::H256,
                        origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        error: runtime_types::frame_support::traits::messages::ProcessMessageError,
                    },
                    #[codec(index = 1)]
                    Processed {
                        id: ::subxt::ext::subxt_core::utils::H256,
                        origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        weight_used: ::sp_weights::Weight,
                        success: ::core::primitive::bool,
                    },
                    #[codec(index = 2)]
                    OverweightEnqueued {
                        id: [::core::primitive::u8; 32usize],
                        origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        page_index: ::core::primitive::u32,
                        message_index: ::core::primitive::u32,
                    },
                    #[codec(index = 3)]
                    PageReaped {
                        origin: runtime_types::polkadot_runtime_parachains::inclusion::AggregateMessageOrigin,
                        index: ::core::primitive::u32,
                    },
                }
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct BookState<_0> {
				pub begin: ::core::primitive::u32,
				pub end: ::core::primitive::u32,
				pub count: ::core::primitive::u32,
				pub ready_neighbours:
					::core::option::Option<runtime_types::pallet_message_queue::Neighbours<_0>>,
				pub message_count: ::core::primitive::u64,
				pub size: ::core::primitive::u64,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Neighbours<_0> {
				pub prev: _0,
				pub next: _0,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Page<_0> {
				pub remaining: _0,
				pub remaining_size: _0,
				pub first_index: _0,
				pub first: _0,
				pub last: _0,
				pub heap: runtime_types::bounded_collections::bounded_vec::BoundedVec<
					::core::primitive::u8,
				>,
			}
		}
		pub mod pallet_multisig {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					as_multi_threshold_1 {
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
						maybe_timepoint: ::core::option::Option<
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
						max_weight: ::sp_weights::Weight,
					},
					#[codec(index = 2)]
					approve_as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
						maybe_timepoint: ::core::option::Option<
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						>,
						call_hash: [::core::primitive::u8; 32usize],
						max_weight: ::sp_weights::Weight,
					},
					#[codec(index = 3)]
					cancel_as_multi {
						threshold: ::core::primitive::u16,
						other_signatories: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
						timepoint:
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						call_hash: [::core::primitive::u8; 32usize],
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					MinimumThreshold,
					#[codec(index = 1)]
					AlreadyApproved,
					#[codec(index = 2)]
					NoApprovalsNeeded,
					#[codec(index = 3)]
					TooFewSignatories,
					#[codec(index = 4)]
					TooManySignatories,
					#[codec(index = 5)]
					SignatoriesOutOfOrder,
					#[codec(index = 6)]
					SenderInSignatories,
					#[codec(index = 7)]
					NotFound,
					#[codec(index = 8)]
					NotOwner,
					#[codec(index = 9)]
					NoTimepoint,
					#[codec(index = 10)]
					WrongTimepoint,
					#[codec(index = 11)]
					UnexpectedTimepoint,
					#[codec(index = 12)]
					MaxWeightTooLow,
					#[codec(index = 13)]
					AlreadyStored,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					NewMultisig {
						approving: ::sp_core::crypto::AccountId32,
						multisig: ::sp_core::crypto::AccountId32,
						call_hash: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 1)]
					MultisigApproval {
						approving: ::sp_core::crypto::AccountId32,
						timepoint:
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						multisig: ::sp_core::crypto::AccountId32,
						call_hash: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 2)]
					MultisigExecuted {
						approving: ::sp_core::crypto::AccountId32,
						timepoint:
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						multisig: ::sp_core::crypto::AccountId32,
						call_hash: [::core::primitive::u8; 32usize],
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 3)]
					MultisigCancelled {
						cancelling: ::sp_core::crypto::AccountId32,
						timepoint:
							runtime_types::pallet_multisig::Timepoint<::core::primitive::u32>,
						multisig: ::sp_core::crypto::AccountId32,
						call_hash: [::core::primitive::u8; 32usize],
					},
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Multisig<_0, _1, _2> {
				pub when: runtime_types::pallet_multisig::Timepoint<_0>,
				pub deposit: _1,
				pub depositor: _2,
				pub approvals: runtime_types::bounded_collections::bounded_vec::BoundedVec<_2>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Timepoint<_0> {
				pub height: _0,
				pub index: ::core::primitive::u32,
			}
		}
		pub mod pallet_nomination_pools {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					join {
						#[codec(compact)]
						amount: ::core::primitive::u128,
						pool_id: ::core::primitive::u32,
					},
					#[codec(index = 1)]
					bond_extra {
						extra: runtime_types::pallet_nomination_pools::BondExtra<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 2)]
					claim_payout,
					#[codec(index = 3)]
					unbond {
						member_account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						#[codec(compact)]
						unbonding_points: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					pool_withdraw_unbonded {
						pool_id: ::core::primitive::u32,
						num_slashing_spans: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					withdraw_unbonded {
						member_account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						num_slashing_spans: ::core::primitive::u32,
					},
					#[codec(index = 6)]
					create {
						#[codec(compact)]
						amount: ::core::primitive::u128,
						root: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						nominator: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						bouncer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 7)]
					create_with_pool_id {
						#[codec(compact)]
						amount: ::core::primitive::u128,
						root: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						nominator: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						bouncer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						pool_id: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					nominate {
						pool_id: ::core::primitive::u32,
						validators: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
					},
					#[codec(index = 9)]
					set_state {
						pool_id: ::core::primitive::u32,
						state: runtime_types::pallet_nomination_pools::PoolState,
					},
					#[codec(index = 10)]
					set_metadata {
						pool_id: ::core::primitive::u32,
						metadata: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 11)]
					set_configs {
						min_join_bond: runtime_types::pallet_nomination_pools::ConfigOp<
							::core::primitive::u128,
						>,
						min_create_bond: runtime_types::pallet_nomination_pools::ConfigOp<
							::core::primitive::u128,
						>,
						max_pools: runtime_types::pallet_nomination_pools::ConfigOp<
							::core::primitive::u32,
						>,
						max_members: runtime_types::pallet_nomination_pools::ConfigOp<
							::core::primitive::u32,
						>,
						max_members_per_pool: runtime_types::pallet_nomination_pools::ConfigOp<
							::core::primitive::u32,
						>,
						global_max_commission: runtime_types::pallet_nomination_pools::ConfigOp<
							runtime_types::sp_arithmetic::per_things::Perbill,
						>,
					},
					#[codec(index = 12)]
					update_roles {
						pool_id: ::core::primitive::u32,
						new_root: runtime_types::pallet_nomination_pools::ConfigOp<
							::sp_core::crypto::AccountId32,
						>,
						new_nominator: runtime_types::pallet_nomination_pools::ConfigOp<
							::sp_core::crypto::AccountId32,
						>,
						new_bouncer: runtime_types::pallet_nomination_pools::ConfigOp<
							::sp_core::crypto::AccountId32,
						>,
					},
					#[codec(index = 13)]
					chill { pool_id: ::core::primitive::u32 },
					#[codec(index = 14)]
					bond_extra_other {
						member: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						extra: runtime_types::pallet_nomination_pools::BondExtra<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 15)]
					set_claim_permission {
						permission: runtime_types::pallet_nomination_pools::ClaimPermission,
					},
					#[codec(index = 16)]
					claim_payout_other { other: ::sp_core::crypto::AccountId32 },
					#[codec(index = 17)]
					set_commission {
						pool_id: ::core::primitive::u32,
						new_commission: ::core::option::Option<(
							runtime_types::sp_arithmetic::per_things::Perbill,
							::sp_core::crypto::AccountId32,
						)>,
					},
					#[codec(index = 18)]
					set_commission_max {
						pool_id: ::core::primitive::u32,
						max_commission: runtime_types::sp_arithmetic::per_things::Perbill,
					},
					#[codec(index = 19)]
					set_commission_change_rate {
						pool_id: ::core::primitive::u32,
						change_rate: runtime_types::pallet_nomination_pools::CommissionChangeRate<
							::core::primitive::u32,
						>,
					},
					#[codec(index = 20)]
					claim_commission { pool_id: ::core::primitive::u32 },
					#[codec(index = 21)]
					adjust_pool_deposit { pool_id: ::core::primitive::u32 },
					#[codec(index = 22)]
					set_commission_claim_permission {
						pool_id: ::core::primitive::u32,
						permission: ::core::option::Option<
							runtime_types::pallet_nomination_pools::CommissionClaimPermission<
								::sp_core::crypto::AccountId32,
							>,
						>,
					},
					#[codec(index = 23)]
					apply_slash {
						member_account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 24)]
					migrate_delegation {
						member_account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 25)]
					migrate_pool_to_delegate_stake { pool_id: ::core::primitive::u32 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum DefensiveError {
					#[codec(index = 0)]
					NotEnoughSpaceInUnbondPool,
					#[codec(index = 1)]
					PoolNotFound,
					#[codec(index = 2)]
					RewardPoolNotFound,
					#[codec(index = 3)]
					SubPoolsNotFound,
					#[codec(index = 4)]
					BondedStashKilledPrematurely,
					#[codec(index = 5)]
					DelegationUnsupported,
					#[codec(index = 6)]
					SlashNotApplied,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					PoolNotFound,
					#[codec(index = 1)]
					PoolMemberNotFound,
					#[codec(index = 2)]
					RewardPoolNotFound,
					#[codec(index = 3)]
					SubPoolsNotFound,
					#[codec(index = 4)]
					AccountBelongsToOtherPool,
					#[codec(index = 5)]
					FullyUnbonding,
					#[codec(index = 6)]
					MaxUnbondingLimit,
					#[codec(index = 7)]
					CannotWithdrawAny,
					#[codec(index = 8)]
					MinimumBondNotMet,
					#[codec(index = 9)]
					OverflowRisk,
					#[codec(index = 10)]
					NotDestroying,
					#[codec(index = 11)]
					NotNominator,
					#[codec(index = 12)]
					NotKickerOrDestroying,
					#[codec(index = 13)]
					NotOpen,
					#[codec(index = 14)]
					MaxPools,
					#[codec(index = 15)]
					MaxPoolMembers,
					#[codec(index = 16)]
					CanNotChangeState,
					#[codec(index = 17)]
					DoesNotHavePermission,
					#[codec(index = 18)]
					MetadataExceedsMaxLen,
					#[codec(index = 19)]
					Defensive(runtime_types::pallet_nomination_pools::pallet::DefensiveError),
					#[codec(index = 20)]
					PartialUnbondNotAllowedPermissionlessly,
					#[codec(index = 21)]
					MaxCommissionRestricted,
					#[codec(index = 22)]
					CommissionExceedsMaximum,
					#[codec(index = 23)]
					CommissionExceedsGlobalMaximum,
					#[codec(index = 24)]
					CommissionChangeThrottled,
					#[codec(index = 25)]
					CommissionChangeRateNotAllowed,
					#[codec(index = 26)]
					NoPendingCommission,
					#[codec(index = 27)]
					NoCommissionCurrentSet,
					#[codec(index = 28)]
					PoolIdInUse,
					#[codec(index = 29)]
					InvalidPoolId,
					#[codec(index = 30)]
					BondExtraRestricted,
					#[codec(index = 31)]
					NothingToAdjust,
					#[codec(index = 32)]
					NothingToSlash,
					#[codec(index = 33)]
					AlreadyMigrated,
					#[codec(index = 34)]
					NotMigrated,
					#[codec(index = 35)]
					NotSupported,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Created {
						depositor: ::sp_core::crypto::AccountId32,
						pool_id: ::core::primitive::u32,
					},
					#[codec(index = 1)]
					Bonded {
						member: ::sp_core::crypto::AccountId32,
						pool_id: ::core::primitive::u32,
						bonded: ::core::primitive::u128,
						joined: ::core::primitive::bool,
					},
					#[codec(index = 2)]
					PaidOut {
						member: ::sp_core::crypto::AccountId32,
						pool_id: ::core::primitive::u32,
						payout: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					Unbonded {
						member: ::sp_core::crypto::AccountId32,
						pool_id: ::core::primitive::u32,
						balance: ::core::primitive::u128,
						points: ::core::primitive::u128,
						era: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					Withdrawn {
						member: ::sp_core::crypto::AccountId32,
						pool_id: ::core::primitive::u32,
						balance: ::core::primitive::u128,
						points: ::core::primitive::u128,
					},
					#[codec(index = 5)]
					Destroyed { pool_id: ::core::primitive::u32 },
					#[codec(index = 6)]
					StateChanged {
						pool_id: ::core::primitive::u32,
						new_state: runtime_types::pallet_nomination_pools::PoolState,
					},
					#[codec(index = 7)]
					MemberRemoved {
						pool_id: ::core::primitive::u32,
						member: ::sp_core::crypto::AccountId32,
						released_balance: ::core::primitive::u128,
					},
					#[codec(index = 8)]
					RolesUpdated {
						root: ::core::option::Option<::sp_core::crypto::AccountId32>,
						bouncer: ::core::option::Option<::sp_core::crypto::AccountId32>,
						nominator: ::core::option::Option<::sp_core::crypto::AccountId32>,
					},
					#[codec(index = 9)]
					PoolSlashed {
						pool_id: ::core::primitive::u32,
						balance: ::core::primitive::u128,
					},
					#[codec(index = 10)]
					UnbondingPoolSlashed {
						pool_id: ::core::primitive::u32,
						era: ::core::primitive::u32,
						balance: ::core::primitive::u128,
					},
					#[codec(index = 11)]
					PoolCommissionUpdated {
						pool_id: ::core::primitive::u32,
						current: ::core::option::Option<(
							runtime_types::sp_arithmetic::per_things::Perbill,
							::sp_core::crypto::AccountId32,
						)>,
					},
					#[codec(index = 12)]
					PoolMaxCommissionUpdated {
						pool_id: ::core::primitive::u32,
						max_commission: runtime_types::sp_arithmetic::per_things::Perbill,
					},
					#[codec(index = 13)]
					PoolCommissionChangeRateUpdated {
						pool_id: ::core::primitive::u32,
						change_rate: runtime_types::pallet_nomination_pools::CommissionChangeRate<
							::core::primitive::u32,
						>,
					},
					#[codec(index = 14)]
					PoolCommissionClaimPermissionUpdated {
						pool_id: ::core::primitive::u32,
						permission: ::core::option::Option<
							runtime_types::pallet_nomination_pools::CommissionClaimPermission<
								::sp_core::crypto::AccountId32,
							>,
						>,
					},
					#[codec(index = 15)]
					PoolCommissionClaimed {
						pool_id: ::core::primitive::u32,
						commission: ::core::primitive::u128,
					},
					#[codec(index = 16)]
					MinBalanceDeficitAdjusted {
						pool_id: ::core::primitive::u32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 17)]
					MinBalanceExcessAdjusted {
						pool_id: ::core::primitive::u32,
						amount: ::core::primitive::u128,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum FreezeReason {
					#[codec(index = 0)]
					PoolMinBalance,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum BondExtra<_0> {
				#[codec(index = 0)]
				FreeBalance(_0),
				#[codec(index = 1)]
				Rewards,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct BondedPoolInner {
				pub commission: runtime_types::pallet_nomination_pools::Commission,
				pub member_counter: ::core::primitive::u32,
				pub points: ::core::primitive::u128,
				pub roles: runtime_types::pallet_nomination_pools::PoolRoles<
					::sp_core::crypto::AccountId32,
				>,
				pub state: runtime_types::pallet_nomination_pools::PoolState,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ClaimPermission {
				#[codec(index = 0)]
				Permissioned,
				#[codec(index = 1)]
				PermissionlessCompound,
				#[codec(index = 2)]
				PermissionlessWithdraw,
				#[codec(index = 3)]
				PermissionlessAll,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Commission {
				pub current: ::core::option::Option<(
					runtime_types::sp_arithmetic::per_things::Perbill,
					::sp_core::crypto::AccountId32,
				)>,
				pub max: ::core::option::Option<runtime_types::sp_arithmetic::per_things::Perbill>,
				pub change_rate: ::core::option::Option<
					runtime_types::pallet_nomination_pools::CommissionChangeRate<
						::core::primitive::u32,
					>,
				>,
				pub throttle_from: ::core::option::Option<::core::primitive::u32>,
				pub claim_permission: ::core::option::Option<
					runtime_types::pallet_nomination_pools::CommissionClaimPermission<
						::sp_core::crypto::AccountId32,
					>,
				>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct CommissionChangeRate<_0> {
				pub max_increase: runtime_types::sp_arithmetic::per_things::Perbill,
				pub min_delay: _0,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum CommissionClaimPermission<_0> {
				#[codec(index = 0)]
				Permissionless,
				#[codec(index = 1)]
				Account(_0),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ConfigOp<_0> {
				#[codec(index = 0)]
				Noop,
				#[codec(index = 1)]
				Set(_0),
				#[codec(index = 2)]
				Remove,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct PoolMember {
				pub pool_id: ::core::primitive::u32,
				pub points: ::core::primitive::u128,
				pub last_recorded_reward_counter:
					runtime_types::sp_arithmetic::fixed_point::FixedU128,
				pub unbonding_eras:
					runtime_types::bounded_collections::bounded_btree_map::BoundedBTreeMap<
						::core::primitive::u32,
						::core::primitive::u128,
					>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct PoolRoles<_0> {
				pub depositor: _0,
				pub root: ::core::option::Option<_0>,
				pub nominator: ::core::option::Option<_0>,
				pub bouncer: ::core::option::Option<_0>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum PoolState {
				#[codec(index = 0)]
				Open,
				#[codec(index = 1)]
				Blocked,
				#[codec(index = 2)]
				Destroying,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RewardPool {
				pub last_recorded_reward_counter:
					runtime_types::sp_arithmetic::fixed_point::FixedU128,
				pub last_recorded_total_payouts: ::core::primitive::u128,
				pub total_rewards_claimed: ::core::primitive::u128,
				pub total_commission_pending: ::core::primitive::u128,
				pub total_commission_claimed: ::core::primitive::u128,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct SubPools {
				pub no_era: runtime_types::pallet_nomination_pools::UnbondPool,
				pub with_era:
					runtime_types::bounded_collections::bounded_btree_map::BoundedBTreeMap<
						::core::primitive::u32,
						runtime_types::pallet_nomination_pools::UnbondPool,
					>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct UnbondPool {
				pub points: ::core::primitive::u128,
				pub balance: ::core::primitive::u128,
			}
		}
		pub mod pallet_offences {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Offence {
						kind: [::core::primitive::u8; 16usize],
						timeslot: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
				}
			}
		}
		pub mod pallet_parameters {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					set_parameter { key_value: runtime_types::westend_runtime::RuntimeParameters },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Updated {
						key: runtime_types::westend_runtime::RuntimeParametersKey,
						old_value: ::core::option::Option<
							runtime_types::westend_runtime::RuntimeParametersValue,
						>,
						new_value: ::core::option::Option<
							runtime_types::westend_runtime::RuntimeParametersValue,
						>,
					},
				}
			}
		}
		pub mod pallet_preimage {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					note_preimage {
						bytes: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					unnote_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					request_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 3)]
					unrequest_preimage { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 4)]
					ensure_updated {
						hashes: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::subxt::ext::subxt_core::utils::H256,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					TooBig,
					#[codec(index = 1)]
					AlreadyNoted,
					#[codec(index = 2)]
					NotAuthorized,
					#[codec(index = 3)]
					NotNoted,
					#[codec(index = 4)]
					Requested,
					#[codec(index = 5)]
					NotRequested,
					#[codec(index = 6)]
					TooMany,
					#[codec(index = 7)]
					TooFew,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Noted { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					Requested { hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					Cleared { hash: ::subxt::ext::subxt_core::utils::H256 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum HoldReason {
					#[codec(index = 0)]
					Preimage,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum OldRequestStatus<_0, _1> {
				#[codec(index = 0)]
				Unrequested { deposit: (_0, _1), len: ::core::primitive::u32 },
				#[codec(index = 1)]
				Requested {
					deposit: ::core::option::Option<(_0, _1)>,
					count: ::core::primitive::u32,
					len: ::core::option::Option<::core::primitive::u32>,
				},
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RequestStatus<_0, _1> {
				#[codec(index = 0)]
				Unrequested { ticket: (_0, _1), len: ::core::primitive::u32 },
				#[codec(index = 1)]
				Requested {
					maybe_ticket: ::core::option::Option<(_0, _1)>,
					count: ::core::primitive::u32,
					maybe_len: ::core::option::Option<::core::primitive::u32>,
				},
			}
		}
		pub mod pallet_proxy {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					proxy {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						force_proxy_type:
							::core::option::Option<runtime_types::westend_runtime::ProxyType>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					add_proxy {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					remove_proxy {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					remove_proxies,
					#[codec(index = 4)]
					create_pure {
						proxy_type: runtime_types::westend_runtime::ProxyType,
						delay: ::core::primitive::u32,
						index: ::core::primitive::u16,
					},
					#[codec(index = 5)]
					kill_pure {
						spawner: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						index: ::core::primitive::u16,
						#[codec(compact)]
						height: ::core::primitive::u32,
						#[codec(compact)]
						ext_index: ::core::primitive::u32,
					},
					#[codec(index = 6)]
					announce {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 7)]
					remove_announcement {
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 8)]
					reject_announcement {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 9)]
					proxy_announced {
						delegate: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						real: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						force_proxy_type:
							::core::option::Option<runtime_types::westend_runtime::ProxyType>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					TooMany,
					#[codec(index = 1)]
					NotFound,
					#[codec(index = 2)]
					NotProxy,
					#[codec(index = 3)]
					Unproxyable,
					#[codec(index = 4)]
					Duplicate,
					#[codec(index = 5)]
					NoPermission,
					#[codec(index = 6)]
					Unannounced,
					#[codec(index = 7)]
					NoSelfProxy,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					ProxyExecuted {
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 1)]
					PureCreated {
						pure: ::sp_core::crypto::AccountId32,
						who: ::sp_core::crypto::AccountId32,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						disambiguation_index: ::core::primitive::u16,
					},
					#[codec(index = 2)]
					Announced {
						real: ::sp_core::crypto::AccountId32,
						proxy: ::sp_core::crypto::AccountId32,
						call_hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 3)]
					ProxyAdded {
						delegator: ::sp_core::crypto::AccountId32,
						delegatee: ::sp_core::crypto::AccountId32,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					ProxyRemoved {
						delegator: ::sp_core::crypto::AccountId32,
						delegatee: ::sp_core::crypto::AccountId32,
						proxy_type: runtime_types::westend_runtime::ProxyType,
						delay: ::core::primitive::u32,
					},
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Announcement<_0, _1, _2> {
				pub real: _0,
				pub call_hash: _1,
				pub height: _2,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ProxyDefinition<_0, _1, _2> {
				pub delegate: _0,
				pub proxy_type: _1,
				pub delay: _2,
			}
		}
		pub mod pallet_recovery {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					as_recovered {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					set_recovered {
						lost: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						rescuer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 2)]
					create_recovery {
						friends: ::subxt::ext::subxt_core::alloc::vec::Vec<
							::sp_core::crypto::AccountId32,
						>,
						threshold: ::core::primitive::u16,
						delay_period: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					initiate_recovery {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 4)]
					vouch_recovery {
						lost: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						rescuer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 5)]
					claim_recovery {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 6)]
					close_recovery {
						rescuer: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 7)]
					remove_recovery,
					#[codec(index = 8)]
					cancel_recovered {
						account: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotAllowed,
					#[codec(index = 1)]
					ZeroThreshold,
					#[codec(index = 2)]
					NotEnoughFriends,
					#[codec(index = 3)]
					MaxFriends,
					#[codec(index = 4)]
					NotSorted,
					#[codec(index = 5)]
					NotRecoverable,
					#[codec(index = 6)]
					AlreadyRecoverable,
					#[codec(index = 7)]
					AlreadyStarted,
					#[codec(index = 8)]
					NotStarted,
					#[codec(index = 9)]
					NotFriend,
					#[codec(index = 10)]
					DelayPeriod,
					#[codec(index = 11)]
					AlreadyVouched,
					#[codec(index = 12)]
					Threshold,
					#[codec(index = 13)]
					StillActive,
					#[codec(index = 14)]
					AlreadyProxy,
					#[codec(index = 15)]
					BadState,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					RecoveryCreated { account: ::sp_core::crypto::AccountId32 },
					#[codec(index = 1)]
					RecoveryInitiated {
						lost_account: ::sp_core::crypto::AccountId32,
						rescuer_account: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 2)]
					RecoveryVouched {
						lost_account: ::sp_core::crypto::AccountId32,
						rescuer_account: ::sp_core::crypto::AccountId32,
						sender: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 3)]
					RecoveryClosed {
						lost_account: ::sp_core::crypto::AccountId32,
						rescuer_account: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 4)]
					AccountRecovered {
						lost_account: ::sp_core::crypto::AccountId32,
						rescuer_account: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 5)]
					RecoveryRemoved { lost_account: ::sp_core::crypto::AccountId32 },
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ActiveRecovery<_0, _1, _2> {
				pub created: _0,
				pub deposit: _1,
				pub friends: _2,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RecoveryConfig<_0, _1, _2> {
				pub delay_period: _0,
				pub deposit: _1,
				pub friends: _2,
				pub threshold: ::core::primitive::u16,
			}
		}
		pub mod pallet_referenda {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					submit {
						proposal_origin: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::OriginCaller,
						>,
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::westend_runtime::RuntimeCall,
							::sp_runtime::traits::BlakeTwo256,
						>,
						enactment_moment:
							runtime_types::frame_support::traits::schedule::DispatchTime<
								::core::primitive::u32,
							>,
					},
					#[codec(index = 1)]
					place_decision_deposit { index: ::core::primitive::u32 },
					#[codec(index = 2)]
					refund_decision_deposit { index: ::core::primitive::u32 },
					#[codec(index = 3)]
					cancel { index: ::core::primitive::u32 },
					#[codec(index = 4)]
					kill { index: ::core::primitive::u32 },
					#[codec(index = 5)]
					nudge_referendum { index: ::core::primitive::u32 },
					#[codec(index = 6)]
					one_fewer_deciding { track: ::core::primitive::u16 },
					#[codec(index = 7)]
					refund_submission_deposit { index: ::core::primitive::u32 },
					#[codec(index = 8)]
					set_metadata {
						index: ::core::primitive::u32,
						maybe_hash: ::core::option::Option<::subxt::ext::subxt_core::utils::H256>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotOngoing,
					#[codec(index = 1)]
					HasDeposit,
					#[codec(index = 2)]
					BadTrack,
					#[codec(index = 3)]
					Full,
					#[codec(index = 4)]
					QueueEmpty,
					#[codec(index = 5)]
					BadReferendum,
					#[codec(index = 6)]
					NothingToDo,
					#[codec(index = 7)]
					NoTrack,
					#[codec(index = 8)]
					Unfinished,
					#[codec(index = 9)]
					NoPermission,
					#[codec(index = 10)]
					NoDeposit,
					#[codec(index = 11)]
					BadStatus,
					#[codec(index = 12)]
					PreimageNotExist,
					#[codec(index = 13)]
					PreimageStoredWithDifferentLength,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Submitted {
						index: ::core::primitive::u32,
						track: ::core::primitive::u16,
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::westend_runtime::RuntimeCall,
							::sp_runtime::traits::BlakeTwo256,
						>,
					},
					#[codec(index = 1)]
					DecisionDepositPlaced {
						index: ::core::primitive::u32,
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 2)]
					DecisionDepositRefunded {
						index: ::core::primitive::u32,
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 3)]
					DepositSlashed {
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 4)]
					DecisionStarted {
						index: ::core::primitive::u32,
						track: ::core::primitive::u16,
						proposal: runtime_types::frame_support::traits::preimages::Bounded<
							runtime_types::westend_runtime::RuntimeCall,
							::sp_runtime::traits::BlakeTwo256,
						>,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 5)]
					ConfirmStarted { index: ::core::primitive::u32 },
					#[codec(index = 6)]
					ConfirmAborted { index: ::core::primitive::u32 },
					#[codec(index = 7)]
					Confirmed {
						index: ::core::primitive::u32,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 8)]
					Approved { index: ::core::primitive::u32 },
					#[codec(index = 9)]
					Rejected {
						index: ::core::primitive::u32,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 10)]
					TimedOut {
						index: ::core::primitive::u32,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 11)]
					Cancelled {
						index: ::core::primitive::u32,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 12)]
					Killed {
						index: ::core::primitive::u32,
						tally: runtime_types::pallet_conviction_voting::types::Tally<
							::core::primitive::u128,
						>,
					},
					#[codec(index = 13)]
					SubmissionDepositRefunded {
						index: ::core::primitive::u32,
						who: ::sp_core::crypto::AccountId32,
						amount: ::core::primitive::u128,
					},
					#[codec(index = 14)]
					MetadataSet {
						index: ::core::primitive::u32,
						hash: ::subxt::ext::subxt_core::utils::H256,
					},
					#[codec(index = 15)]
					MetadataCleared {
						index: ::core::primitive::u32,
						hash: ::subxt::ext::subxt_core::utils::H256,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Curve {
					#[codec(index = 0)]
					LinearDecreasing {
						length: runtime_types::sp_arithmetic::per_things::Perbill,
						floor: runtime_types::sp_arithmetic::per_things::Perbill,
						ceil: runtime_types::sp_arithmetic::per_things::Perbill,
					},
					#[codec(index = 1)]
					SteppedDecreasing {
						begin: runtime_types::sp_arithmetic::per_things::Perbill,
						end: runtime_types::sp_arithmetic::per_things::Perbill,
						step: runtime_types::sp_arithmetic::per_things::Perbill,
						period: runtime_types::sp_arithmetic::per_things::Perbill,
					},
					#[codec(index = 2)]
					Reciprocal {
						factor: runtime_types::sp_arithmetic::fixed_point::FixedI64,
						x_offset: runtime_types::sp_arithmetic::fixed_point::FixedI64,
						y_offset: runtime_types::sp_arithmetic::fixed_point::FixedI64,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct DecidingStatus<_0> {
					pub since: _0,
					pub confirming: ::core::option::Option<_0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Deposit<_0, _1> {
					pub who: _0,
					pub amount: _1,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum ReferendumInfo<_0, _1, _2, _3, _4, _5, _6, _7> {
					#[codec(index = 0)]
					Ongoing(
						runtime_types::pallet_referenda::types::ReferendumStatus<
							_0,
							_1,
							_2,
							_3,
							_4,
							_5,
							_6,
							_7,
						>,
					),
					#[codec(index = 1)]
					Approved(
						_2,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
					),
					#[codec(index = 2)]
					Rejected(
						_2,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
					),
					#[codec(index = 3)]
					Cancelled(
						_2,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
					),
					#[codec(index = 4)]
					TimedOut(
						_2,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
						::core::option::Option<
							runtime_types::pallet_referenda::types::Deposit<_6, _4>,
						>,
					),
					#[codec(index = 5)]
					Killed(_2),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ReferendumStatus<_0, _1, _2, _3, _4, _5, _6, _7> {
					pub track: _0,
					pub origin: _1,
					pub proposal: _3,
					pub enactment: runtime_types::frame_support::traits::schedule::DispatchTime<_2>,
					pub submitted: _2,
					pub submission_deposit: runtime_types::pallet_referenda::types::Deposit<_6, _4>,
					pub decision_deposit: ::core::option::Option<
						runtime_types::pallet_referenda::types::Deposit<_6, _4>,
					>,
					pub deciding: ::core::option::Option<
						runtime_types::pallet_referenda::types::DecidingStatus<_2>,
					>,
					pub tally: _5,
					pub in_queue: ::core::primitive::bool,
					pub alarm: ::core::option::Option<(_2, _7)>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct TrackInfo<_0, _1> {
					pub name: ::subxt::ext::subxt_core::alloc::string::String,
					pub max_deciding: ::core::primitive::u32,
					pub decision_deposit: _0,
					pub prepare_period: _1,
					pub decision_period: _1,
					pub confirm_period: _1,
					pub min_enactment_period: _1,
					pub min_approval: runtime_types::pallet_referenda::types::Curve,
					pub min_support: runtime_types::pallet_referenda::types::Curve,
				}
			}
		}
		pub mod pallet_root_testing {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					fill_block { ratio: runtime_types::sp_arithmetic::per_things::Perbill },
					#[codec(index = 1)]
					trigger_defensive,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					DefensiveTestCall,
				}
			}
		}
		pub mod pallet_scheduler {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					schedule {
						when: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					cancel { when: ::core::primitive::u32, index: ::core::primitive::u32 },
					#[codec(index = 2)]
					schedule_named {
						id: [::core::primitive::u8; 32usize],
						when: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 3)]
					cancel_named { id: [::core::primitive::u8; 32usize] },
					#[codec(index = 4)]
					schedule_after {
						after: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 5)]
					schedule_named_after {
						id: [::core::primitive::u8; 32usize],
						after: ::core::primitive::u32,
						maybe_periodic: ::core::option::Option<(
							::core::primitive::u32,
							::core::primitive::u32,
						)>,
						priority: ::core::primitive::u8,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 6)]
					set_retry {
						task: (::core::primitive::u32, ::core::primitive::u32),
						retries: ::core::primitive::u8,
						period: ::core::primitive::u32,
					},
					#[codec(index = 7)]
					set_retry_named {
						id: [::core::primitive::u8; 32usize],
						retries: ::core::primitive::u8,
						period: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					cancel_retry { task: (::core::primitive::u32, ::core::primitive::u32) },
					#[codec(index = 9)]
					cancel_retry_named { id: [::core::primitive::u8; 32usize] },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					FailedToSchedule,
					#[codec(index = 1)]
					NotFound,
					#[codec(index = 2)]
					TargetBlockNumberInPast,
					#[codec(index = 3)]
					RescheduleNoChange,
					#[codec(index = 4)]
					Named,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Scheduled { when: ::core::primitive::u32, index: ::core::primitive::u32 },
					#[codec(index = 1)]
					Canceled { when: ::core::primitive::u32, index: ::core::primitive::u32 },
					#[codec(index = 2)]
					Dispatched {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 3)]
					RetrySet {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
						period: ::core::primitive::u32,
						retries: ::core::primitive::u8,
					},
					#[codec(index = 4)]
					RetryCancelled {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
					},
					#[codec(index = 5)]
					CallUnavailable {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
					},
					#[codec(index = 6)]
					PeriodicFailed {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
					},
					#[codec(index = 7)]
					RetryFailed {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
					},
					#[codec(index = 8)]
					PermanentlyOverweight {
						task: (::core::primitive::u32, ::core::primitive::u32),
						id: ::core::option::Option<[::core::primitive::u8; 32usize]>,
					},
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RetryConfig<_0> {
				pub total_retries: ::core::primitive::u8,
				pub remaining: ::core::primitive::u8,
				pub period: _0,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Scheduled<_0, _1, _2, _3, _4> {
				pub maybe_id: ::core::option::Option<_0>,
				pub priority: ::core::primitive::u8,
				pub call: _1,
				pub maybe_periodic: ::core::option::Option<(_2, _2)>,
				pub origin: _3,
				#[codec(skip)]
				pub __ignore: ::core::marker::PhantomData<_4>,
			}
		}
		pub mod pallet_session {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					set_keys {
						keys: runtime_types::westend_runtime::SessionKeys,
						proof: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					},
					#[codec(index = 1)]
					purge_keys,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					InvalidProof,
					#[codec(index = 1)]
					NoAssociatedValidatorId,
					#[codec(index = 2)]
					DuplicatedKey,
					#[codec(index = 3)]
					NoKeys,
					#[codec(index = 4)]
					NoAccount,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					NewSession { session_index: ::core::primitive::u32 },
				}
			}
		}
		pub mod pallet_staking {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						bond {
							#[codec(compact)]
							value: ::core::primitive::u128,
							payee: runtime_types::pallet_staking::RewardDestination<
								::sp_core::crypto::AccountId32,
							>,
						},
						#[codec(index = 1)]
						bond_extra {
							#[codec(compact)]
							max_additional: ::core::primitive::u128,
						},
						#[codec(index = 2)]
						unbond {
							#[codec(compact)]
							value: ::core::primitive::u128,
						},
						#[codec(index = 3)]
						withdraw_unbonded { num_slashing_spans: ::core::primitive::u32 },
						#[codec(index = 4)]
						validate { prefs: runtime_types::pallet_staking::ValidatorPrefs },
						#[codec(index = 5)]
						nominate {
							targets: ::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::MultiAddress<
									::sp_core::crypto::AccountId32,
									(),
								>,
							>,
						},
						#[codec(index = 6)]
						chill,
						#[codec(index = 7)]
						set_payee {
							payee: runtime_types::pallet_staking::RewardDestination<
								::sp_core::crypto::AccountId32,
							>,
						},
						#[codec(index = 8)]
						set_controller,
						#[codec(index = 9)]
						set_validator_count {
							#[codec(compact)]
							new: ::core::primitive::u32,
						},
						#[codec(index = 10)]
						increase_validator_count {
							#[codec(compact)]
							additional: ::core::primitive::u32,
						},
						#[codec(index = 11)]
						scale_validator_count {
							factor: runtime_types::sp_arithmetic::per_things::Percent,
						},
						#[codec(index = 12)]
						force_no_eras,
						#[codec(index = 13)]
						force_new_era,
						#[codec(index = 14)]
						set_invulnerables {
							invulnerables: ::subxt::ext::subxt_core::alloc::vec::Vec<
								::sp_core::crypto::AccountId32,
							>,
						},
						#[codec(index = 15)]
						force_unstake {
							stash: ::sp_core::crypto::AccountId32,
							num_slashing_spans: ::core::primitive::u32,
						},
						#[codec(index = 16)]
						force_new_era_always,
						#[codec(index = 17)]
						cancel_deferred_slash {
							era: ::core::primitive::u32,
							slash_indices:
								::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u32>,
						},
						#[codec(index = 18)]
						payout_stakers {
							validator_stash: ::sp_core::crypto::AccountId32,
							era: ::core::primitive::u32,
						},
						#[codec(index = 19)]
						rebond {
							#[codec(compact)]
							value: ::core::primitive::u128,
						},
						#[codec(index = 20)]
						reap_stash {
							stash: ::sp_core::crypto::AccountId32,
							num_slashing_spans: ::core::primitive::u32,
						},
						#[codec(index = 21)]
						kick {
							who: ::subxt::ext::subxt_core::alloc::vec::Vec<
								::subxt::ext::subxt_core::utils::MultiAddress<
									::sp_core::crypto::AccountId32,
									(),
								>,
							>,
						},
						#[codec(index = 22)]
						set_staking_configs {
							min_nominator_bond:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									::core::primitive::u128,
								>,
							min_validator_bond:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									::core::primitive::u128,
								>,
							max_nominator_count:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									::core::primitive::u32,
								>,
							max_validator_count:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									::core::primitive::u32,
								>,
							chill_threshold:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									runtime_types::sp_arithmetic::per_things::Percent,
								>,
							min_commission: runtime_types::pallet_staking::pallet::pallet::ConfigOp<
								runtime_types::sp_arithmetic::per_things::Perbill,
							>,
							max_staked_rewards:
								runtime_types::pallet_staking::pallet::pallet::ConfigOp<
									runtime_types::sp_arithmetic::per_things::Percent,
								>,
						},
						#[codec(index = 23)]
						chill_other { stash: ::sp_core::crypto::AccountId32 },
						#[codec(index = 24)]
						force_apply_min_commission {
							validator_stash: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 25)]
						set_min_commission {
							new: runtime_types::sp_arithmetic::per_things::Perbill,
						},
						#[codec(index = 26)]
						payout_stakers_by_page {
							validator_stash: ::sp_core::crypto::AccountId32,
							era: ::core::primitive::u32,
							page: ::core::primitive::u32,
						},
						#[codec(index = 27)]
						update_payee { controller: ::sp_core::crypto::AccountId32 },
						#[codec(index = 28)]
						deprecate_controller_batch {
							controllers:
								runtime_types::bounded_collections::bounded_vec::BoundedVec<
									::sp_core::crypto::AccountId32,
								>,
						},
						#[codec(index = 29)]
						restore_ledger {
							stash: ::sp_core::crypto::AccountId32,
							maybe_controller:
								::core::option::Option<::sp_core::crypto::AccountId32>,
							maybe_total: ::core::option::Option<::core::primitive::u128>,
							maybe_unlocking: ::core::option::Option<
								runtime_types::bounded_collections::bounded_vec::BoundedVec<
									runtime_types::pallet_staking::UnlockChunk<
										::core::primitive::u128,
									>,
								>,
							>,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum ConfigOp<_0> {
						#[codec(index = 0)]
						Noop,
						#[codec(index = 1)]
						Set(_0),
						#[codec(index = 2)]
						Remove,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						NotController,
						#[codec(index = 1)]
						NotStash,
						#[codec(index = 2)]
						AlreadyBonded,
						#[codec(index = 3)]
						AlreadyPaired,
						#[codec(index = 4)]
						EmptyTargets,
						#[codec(index = 5)]
						DuplicateIndex,
						#[codec(index = 6)]
						InvalidSlashIndex,
						#[codec(index = 7)]
						InsufficientBond,
						#[codec(index = 8)]
						NoMoreChunks,
						#[codec(index = 9)]
						NoUnlockChunk,
						#[codec(index = 10)]
						FundedTarget,
						#[codec(index = 11)]
						InvalidEraToReward,
						#[codec(index = 12)]
						InvalidNumberOfNominations,
						#[codec(index = 13)]
						NotSortedAndUnique,
						#[codec(index = 14)]
						AlreadyClaimed,
						#[codec(index = 15)]
						InvalidPage,
						#[codec(index = 16)]
						IncorrectHistoryDepth,
						#[codec(index = 17)]
						IncorrectSlashingSpans,
						#[codec(index = 18)]
						BadState,
						#[codec(index = 19)]
						TooManyTargets,
						#[codec(index = 20)]
						BadTarget,
						#[codec(index = 21)]
						CannotChillOther,
						#[codec(index = 22)]
						TooManyNominators,
						#[codec(index = 23)]
						TooManyValidators,
						#[codec(index = 24)]
						CommissionTooLow,
						#[codec(index = 25)]
						BoundNotMet,
						#[codec(index = 26)]
						ControllerDeprecated,
						#[codec(index = 27)]
						CannotRestoreLedger,
						#[codec(index = 28)]
						RewardDestinationRestricted,
						#[codec(index = 29)]
						NotEnoughFunds,
						#[codec(index = 30)]
						VirtualStakerNotAllowed,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						EraPaid {
							era_index: ::core::primitive::u32,
							validator_payout: ::core::primitive::u128,
							remainder: ::core::primitive::u128,
						},
						#[codec(index = 1)]
						Rewarded {
							stash: ::sp_core::crypto::AccountId32,
							dest: runtime_types::pallet_staking::RewardDestination<
								::sp_core::crypto::AccountId32,
							>,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 2)]
						Slashed {
							staker: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 3)]
						SlashReported {
							validator: ::sp_core::crypto::AccountId32,
							fraction: runtime_types::sp_arithmetic::per_things::Perbill,
							slash_era: ::core::primitive::u32,
						},
						#[codec(index = 4)]
						OldSlashingReportDiscarded { session_index: ::core::primitive::u32 },
						#[codec(index = 5)]
						StakersElected,
						#[codec(index = 6)]
						Bonded {
							stash: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 7)]
						Unbonded {
							stash: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 8)]
						Withdrawn {
							stash: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 9)]
						Kicked {
							nominator: ::sp_core::crypto::AccountId32,
							stash: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 10)]
						StakingElectionFailed,
						#[codec(index = 11)]
						Chilled { stash: ::sp_core::crypto::AccountId32 },
						#[codec(index = 12)]
						PayoutStarted {
							era_index: ::core::primitive::u32,
							validator_stash: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 13)]
						ValidatorPrefsSet {
							stash: ::sp_core::crypto::AccountId32,
							prefs: runtime_types::pallet_staking::ValidatorPrefs,
						},
						#[codec(index = 14)]
						SnapshotVotersSizeExceeded { size: ::core::primitive::u32 },
						#[codec(index = 15)]
						SnapshotTargetsSizeExceeded { size: ::core::primitive::u32 },
						#[codec(index = 16)]
						ForceEra { mode: runtime_types::pallet_staking::Forcing },
						#[codec(index = 17)]
						ControllerBatchDeprecated { failures: ::core::primitive::u32 },
					}
				}
			}
			pub mod slashing {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SlashingSpans {
					pub span_index: ::core::primitive::u32,
					pub last_start: ::core::primitive::u32,
					pub last_nonzero_slash: ::core::primitive::u32,
					pub prior: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u32>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SpanRecord<_0> {
					pub slashed: _0,
					pub paid_out: _0,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ActiveEraInfo {
				pub index: ::core::primitive::u32,
				pub start: ::core::option::Option<::core::primitive::u64>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct EraRewardPoints<_0> {
				pub total: ::core::primitive::u32,
				pub individual:
					::subxt::ext::subxt_core::utils::KeyedVec<_0, ::core::primitive::u32>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Forcing {
				#[codec(index = 0)]
				NotForcing,
				#[codec(index = 1)]
				ForceNew,
				#[codec(index = 2)]
				ForceNone,
				#[codec(index = 3)]
				ForceAlways,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Nominations {
				pub targets: runtime_types::bounded_collections::bounded_vec::BoundedVec<
					::sp_core::crypto::AccountId32,
				>,
				pub submitted_in: ::core::primitive::u32,
				pub suppressed: ::core::primitive::bool,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RewardDestination<_0> {
				#[codec(index = 0)]
				Staked,
				#[codec(index = 1)]
				Stash,
				#[codec(index = 2)]
				Controller,
				#[codec(index = 3)]
				Account(_0),
				#[codec(index = 4)]
				None,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct StakingLedger {
				pub stash: ::sp_core::crypto::AccountId32,
				#[codec(compact)]
				pub total: ::core::primitive::u128,
				#[codec(compact)]
				pub active: ::core::primitive::u128,
				pub unlocking: runtime_types::bounded_collections::bounded_vec::BoundedVec<
					runtime_types::pallet_staking::UnlockChunk<::core::primitive::u128>,
				>,
				pub legacy_claimed_rewards:
					runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u32,
					>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct UnappliedSlash<_0, _1> {
				pub validator: _0,
				pub own: _1,
				pub others: ::subxt::ext::subxt_core::alloc::vec::Vec<(_0, _1)>,
				pub reporters: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
				pub payout: _1,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct UnlockChunk<_0> {
				#[codec(compact)]
				pub value: _0,
				#[codec(compact)]
				pub era: ::core::primitive::u32,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ValidatorPrefs {
				#[codec(compact)]
				pub commission: runtime_types::sp_arithmetic::per_things::Perbill,
				pub blocked: ::core::primitive::bool,
			}
		}
		pub mod pallet_sudo {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					sudo {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					sudo_unchecked_weight {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
						weight: ::sp_weights::Weight,
					},
					#[codec(index = 2)]
					set_key {
						new: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 3)]
					sudo_as {
						who: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 4)]
					remove_key,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					RequireSudo,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Sudid {
						sudo_result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
					#[codec(index = 1)]
					KeyChanged {
						old: ::core::option::Option<::sp_core::crypto::AccountId32>,
						new: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 2)]
					KeyRemoved,
					#[codec(index = 3)]
					SudoAsDone {
						sudo_result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
				}
			}
		}
		pub mod pallet_timestamp {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					set {
						#[codec(compact)]
						now: ::core::primitive::u64,
					},
				}
			}
		}
		pub mod pallet_transaction_payment {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					TransactionFeePaid {
						who: ::sp_core::crypto::AccountId32,
						actual_fee: ::core::primitive::u128,
						tip: ::core::primitive::u128,
					},
				}
			}
			pub mod types {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct FeeDetails<_0> {
					pub inclusion_fee: ::core::option::Option<
						runtime_types::pallet_transaction_payment::types::InclusionFee<_0>,
					>,
					pub tip: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct InclusionFee<_0> {
					pub base_fee: _0,
					pub len_fee: _0,
					pub adjusted_weight_fee: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct RuntimeDispatchInfo<_0, _1> {
					pub weight: _1,
					pub class: runtime_types::frame_support::dispatch::DispatchClass,
					pub partial_fee: _0,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ChargeTransactionPayment(#[codec(compact)] pub ::core::primitive::u128);
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Releases {
				#[codec(index = 0)]
				V1Ancient,
				#[codec(index = 1)]
				V2,
			}
		}
		pub mod pallet_treasury {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 3)]
					spend_local {
						#[codec(compact)]
						amount: ::core::primitive::u128,
						beneficiary: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 4)]
					remove_approval {
						#[codec(compact)]
						proposal_id: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					spend {
						asset_kind: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						>,
						#[codec(compact)]
						amount: ::core::primitive::u128,
						beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::xcm::VersionedLocation,
						>,
						valid_from: ::core::option::Option<::core::primitive::u32>,
					},
					#[codec(index = 6)]
					payout { index: ::core::primitive::u32 },
					#[codec(index = 7)]
					check_status { index: ::core::primitive::u32 },
					#[codec(index = 8)]
					void_spend { index: ::core::primitive::u32 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					InvalidIndex,
					#[codec(index = 1)]
					TooManyApprovals,
					#[codec(index = 2)]
					InsufficientPermission,
					#[codec(index = 3)]
					ProposalNotApproved,
					#[codec(index = 4)]
					FailedToConvertBalance,
					#[codec(index = 5)]
					SpendExpired,
					#[codec(index = 6)]
					EarlyPayout,
					#[codec(index = 7)]
					AlreadyAttempted,
					#[codec(index = 8)]
					PayoutError,
					#[codec(index = 9)]
					NotAttempted,
					#[codec(index = 10)]
					Inconclusive,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Spending { budget_remaining: ::core::primitive::u128 },
					#[codec(index = 1)]
					Awarded {
						proposal_index: ::core::primitive::u32,
						award: ::core::primitive::u128,
						account: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 2)]
					Burnt { burnt_funds: ::core::primitive::u128 },
					#[codec(index = 3)]
					Rollover { rollover_balance: ::core::primitive::u128 },
					#[codec(index = 4)]
					Deposit { value: ::core::primitive::u128 },
					#[codec(index = 5)]
					SpendApproved {
						proposal_index: ::core::primitive::u32,
						amount: ::core::primitive::u128,
						beneficiary: ::sp_core::crypto::AccountId32,
					},
					#[codec(index = 6)]
					UpdatedInactive {
						reactivated: ::core::primitive::u128,
						deactivated: ::core::primitive::u128,
					},
					#[codec(index = 7)]
					AssetSpendApproved {
						index: ::core::primitive::u32,
						asset_kind:
							runtime_types::polkadot_runtime_common::impls::VersionedLocatableAsset,
						amount: ::core::primitive::u128,
						beneficiary: runtime_types::xcm::VersionedLocation,
						valid_from: ::core::primitive::u32,
						expire_at: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					AssetSpendVoided { index: ::core::primitive::u32 },
					#[codec(index = 9)]
					Paid { index: ::core::primitive::u32, payment_id: ::core::primitive::u64 },
					#[codec(index = 10)]
					PaymentFailed {
						index: ::core::primitive::u32,
						payment_id: ::core::primitive::u64,
					},
					#[codec(index = 11)]
					SpendProcessed { index: ::core::primitive::u32 },
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum PaymentState<_0> {
				#[codec(index = 0)]
				Pending,
				#[codec(index = 1)]
				Attempted { id: _0 },
				#[codec(index = 2)]
				Failed,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Proposal<_0, _1> {
				pub proposer: _0,
				pub value: _1,
				pub beneficiary: _0,
				pub bond: _1,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct SpendStatus<_0, _1, _2, _3, _4> {
				pub asset_kind: _0,
				pub amount: _1,
				pub beneficiary: _2,
				pub valid_from: _3,
				pub expire_at: _3,
				pub status: runtime_types::pallet_treasury::PaymentState<_4>,
			}
		}
		pub mod pallet_utility {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					batch {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 1)]
					as_derivative {
						index: ::core::primitive::u16,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 2)]
					batch_all {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 3)]
					dispatch_as {
						as_origin: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::OriginCaller,
						>,
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 4)]
					force_batch {
						calls: ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
					#[codec(index = 5)]
					with_weight {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
						weight: ::sp_weights::Weight,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					TooManyCalls,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					BatchInterrupted {
						index: ::core::primitive::u32,
						error: runtime_types::sp_runtime::DispatchError,
					},
					#[codec(index = 1)]
					BatchCompleted,
					#[codec(index = 2)]
					BatchCompletedWithErrors,
					#[codec(index = 3)]
					ItemCompleted,
					#[codec(index = 4)]
					ItemFailed { error: runtime_types::sp_runtime::DispatchError },
					#[codec(index = 5)]
					DispatchedAs {
						result:
							::core::result::Result<(), runtime_types::sp_runtime::DispatchError>,
					},
				}
			}
		}
		pub mod pallet_vesting {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					vest,
					#[codec(index = 1)]
					vest_other {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
					},
					#[codec(index = 2)]
					vested_transfer {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						schedule: runtime_types::pallet_vesting::vesting_info::VestingInfo<
							::core::primitive::u128,
							::core::primitive::u32,
						>,
					},
					#[codec(index = 3)]
					force_vested_transfer {
						source: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						schedule: runtime_types::pallet_vesting::vesting_info::VestingInfo<
							::core::primitive::u128,
							::core::primitive::u32,
						>,
					},
					#[codec(index = 4)]
					merge_schedules {
						schedule1_index: ::core::primitive::u32,
						schedule2_index: ::core::primitive::u32,
					},
					#[codec(index = 5)]
					force_remove_vesting_schedule {
						target: ::subxt::ext::subxt_core::utils::MultiAddress<
							::sp_core::crypto::AccountId32,
							(),
						>,
						schedule_index: ::core::primitive::u32,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					NotVesting,
					#[codec(index = 1)]
					AtMaxVestingSchedules,
					#[codec(index = 2)]
					AmountLow,
					#[codec(index = 3)]
					ScheduleIndexOutOfBounds,
					#[codec(index = 4)]
					InvalidScheduleParams,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					VestingUpdated {
						account: ::sp_core::crypto::AccountId32,
						unvested: ::core::primitive::u128,
					},
					#[codec(index = 1)]
					VestingCompleted { account: ::sp_core::crypto::AccountId32 },
				}
			}
			pub mod vesting_info {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct VestingInfo<_0, _1> {
					pub locked: _0,
					pub per_block: _0,
					pub starting_block: _1,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Releases {
				#[codec(index = 0)]
				V0,
				#[codec(index = 1)]
				V1,
			}
		}
		pub mod pallet_whitelist {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
					whitelist_call { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					remove_whitelisted_call { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					dispatch_whitelisted_call {
						call_hash: ::subxt::ext::subxt_core::utils::H256,
						call_encoded_len: ::core::primitive::u32,
						call_weight_witness: ::sp_weights::Weight,
					},
					#[codec(index = 3)]
					dispatch_whitelisted_call_with_preimage {
						call: ::subxt::ext::subxt_core::alloc::boxed::Box<
							runtime_types::westend_runtime::RuntimeCall,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					UnavailablePreImage,
					#[codec(index = 1)]
					UndecodableCall,
					#[codec(index = 2)]
					InvalidCallWeightWitness,
					#[codec(index = 3)]
					CallIsNotWhitelisted,
					#[codec(index = 4)]
					CallAlreadyWhitelisted,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					CallWhitelisted { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 1)]
					WhitelistedCallRemoved { call_hash: ::subxt::ext::subxt_core::utils::H256 },
					#[codec(index = 2)]
					WhitelistedCallDispatched {
						call_hash: ::subxt::ext::subxt_core::utils::H256,
						result: ::core::result::Result<
							runtime_types::frame_support::dispatch::PostDispatchInfo,
							runtime_types::sp_runtime::DispatchErrorWithPostInfo<
								runtime_types::frame_support::dispatch::PostDispatchInfo,
							>,
						>,
					},
				}
			}
		}
		pub mod pallet_xcm {
			use super::runtime_types;
			pub mod pallet {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Call {
					#[codec(index = 0)]
                    send {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        message: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedXcm,
                        >,
                    },
                    #[codec(index = 1)]
                    teleport_assets {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        fee_asset_item: ::core::primitive::u32,
                    },
                    #[codec(index = 2)]
                    reserve_transfer_assets {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        fee_asset_item: ::core::primitive::u32,
                    },
                    #[codec(index = 3)]
                    execute {
                        message: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedXcm,
                        >,
                        max_weight: ::sp_weights::Weight,
                    },
                    #[codec(index = 4)]
                    force_xcm_version {
                        location: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::staging_xcm::v4::location::Location,
                        >,
                        version: ::core::primitive::u32,
                    },
                    #[codec(index = 5)]
                    force_default_xcm_version {
                        maybe_xcm_version: ::core::option::Option<::core::primitive::u32>,
                    },
                    #[codec(index = 6)]
                    force_subscribe_version_notify {
                        location: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                    },
                    #[codec(index = 7)]
                    force_unsubscribe_version_notify {
                        location: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                    },
                    #[codec(index = 8)]
                    limited_reserve_transfer_assets {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        fee_asset_item: ::core::primitive::u32,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 9)]
                    limited_teleport_assets {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        fee_asset_item: ::core::primitive::u32,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 10)]
                    force_suspension { suspended: ::core::primitive::bool },
                    #[codec(index = 11)]
                    transfer_assets {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        fee_asset_item: ::core::primitive::u32,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                    #[codec(index = 12)]
                    claim_assets {
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        beneficiary: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                    },
                    #[codec(index = 13)]
                    transfer_assets_using_type_and_then {
                        dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedLocation,
                        >,
                        assets: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssets,
                        >,
                        assets_transfer_type: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::staging_xcm_executor::traits::asset_transfer::TransferType,
                        >,
                        remote_fees_id: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedAssetId,
                        >,
                        fees_transfer_type: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::staging_xcm_executor::traits::asset_transfer::TransferType,
                        >,
                        custom_xcm_on_dest: ::subxt::ext::subxt_core::alloc::boxed::Box<
                            runtime_types::xcm::VersionedXcm,
                        >,
                        weight_limit: runtime_types::xcm::v3::WeightLimit,
                    },
                }
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					Unreachable,
					#[codec(index = 1)]
					SendFailure,
					#[codec(index = 2)]
					Filtered,
					#[codec(index = 3)]
					UnweighableMessage,
					#[codec(index = 4)]
					DestinationNotInvertible,
					#[codec(index = 5)]
					Empty,
					#[codec(index = 6)]
					CannotReanchor,
					#[codec(index = 7)]
					TooManyAssets,
					#[codec(index = 8)]
					InvalidOrigin,
					#[codec(index = 9)]
					BadVersion,
					#[codec(index = 10)]
					BadLocation,
					#[codec(index = 11)]
					NoSubscription,
					#[codec(index = 12)]
					AlreadySubscribed,
					#[codec(index = 13)]
					CannotCheckOutTeleport,
					#[codec(index = 14)]
					LowBalance,
					#[codec(index = 15)]
					TooManyLocks,
					#[codec(index = 16)]
					AccountNotSovereign,
					#[codec(index = 17)]
					FeesNotMet,
					#[codec(index = 18)]
					LockNotFound,
					#[codec(index = 19)]
					InUse,
					#[codec(index = 21)]
					InvalidAssetUnknownReserve,
					#[codec(index = 22)]
					InvalidAssetUnsupportedReserve,
					#[codec(index = 23)]
					TooManyReserves,
					#[codec(index = 24)]
					LocalExecutionIncomplete,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Event {
					#[codec(index = 0)]
					Attempted { outcome: runtime_types::staging_xcm::v4::traits::Outcome },
					#[codec(index = 1)]
					Sent {
						origin: runtime_types::staging_xcm::v4::location::Location,
						destination: runtime_types::staging_xcm::v4::location::Location,
						message: runtime_types::staging_xcm::v4::Xcm,
						message_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 2)]
					UnexpectedResponse {
						origin: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
					},
					#[codec(index = 3)]
					ResponseReady {
						query_id: ::core::primitive::u64,
						response: runtime_types::staging_xcm::v4::Response,
					},
					#[codec(index = 4)]
					Notified {
						query_id: ::core::primitive::u64,
						pallet_index: ::core::primitive::u8,
						call_index: ::core::primitive::u8,
					},
					#[codec(index = 5)]
					NotifyOverweight {
						query_id: ::core::primitive::u64,
						pallet_index: ::core::primitive::u8,
						call_index: ::core::primitive::u8,
						actual_weight: ::sp_weights::Weight,
						max_budgeted_weight: ::sp_weights::Weight,
					},
					#[codec(index = 6)]
					NotifyDispatchError {
						query_id: ::core::primitive::u64,
						pallet_index: ::core::primitive::u8,
						call_index: ::core::primitive::u8,
					},
					#[codec(index = 7)]
					NotifyDecodeFailed {
						query_id: ::core::primitive::u64,
						pallet_index: ::core::primitive::u8,
						call_index: ::core::primitive::u8,
					},
					#[codec(index = 8)]
					InvalidResponder {
						origin: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
						expected_location: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
					#[codec(index = 9)]
					InvalidResponderVersion {
						origin: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
					},
					#[codec(index = 10)]
					ResponseTaken { query_id: ::core::primitive::u64 },
					#[codec(index = 11)]
					AssetsTrapped {
						hash: ::subxt::ext::subxt_core::utils::H256,
						origin: runtime_types::staging_xcm::v4::location::Location,
						assets: runtime_types::xcm::VersionedAssets,
					},
					#[codec(index = 12)]
					VersionChangeNotified {
						destination: runtime_types::staging_xcm::v4::location::Location,
						result: ::core::primitive::u32,
						cost: runtime_types::staging_xcm::v4::asset::Assets,
						message_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 13)]
					SupportedVersionChanged {
						location: runtime_types::staging_xcm::v4::location::Location,
						version: ::core::primitive::u32,
					},
					#[codec(index = 14)]
					NotifyTargetSendFail {
						location: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
						error: runtime_types::xcm::v3::traits::Error,
					},
					#[codec(index = 15)]
					NotifyTargetMigrationFail {
						location: runtime_types::xcm::VersionedLocation,
						query_id: ::core::primitive::u64,
					},
					#[codec(index = 16)]
					InvalidQuerierVersion {
						origin: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
					},
					#[codec(index = 17)]
					InvalidQuerier {
						origin: runtime_types::staging_xcm::v4::location::Location,
						query_id: ::core::primitive::u64,
						expected_querier: runtime_types::staging_xcm::v4::location::Location,
						maybe_actual_querier: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
					#[codec(index = 18)]
					VersionNotifyStarted {
						destination: runtime_types::staging_xcm::v4::location::Location,
						cost: runtime_types::staging_xcm::v4::asset::Assets,
						message_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 19)]
					VersionNotifyRequested {
						destination: runtime_types::staging_xcm::v4::location::Location,
						cost: runtime_types::staging_xcm::v4::asset::Assets,
						message_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 20)]
					VersionNotifyUnrequested {
						destination: runtime_types::staging_xcm::v4::location::Location,
						cost: runtime_types::staging_xcm::v4::asset::Assets,
						message_id: [::core::primitive::u8; 32usize],
					},
					#[codec(index = 21)]
					FeesPaid {
						paying: runtime_types::staging_xcm::v4::location::Location,
						fees: runtime_types::staging_xcm::v4::asset::Assets,
					},
					#[codec(index = 22)]
					AssetsClaimed {
						hash: ::subxt::ext::subxt_core::utils::H256,
						origin: runtime_types::staging_xcm::v4::location::Location,
						assets: runtime_types::xcm::VersionedAssets,
					},
					#[codec(index = 23)]
					VersionMigrationFinished { version: ::core::primitive::u32 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Origin {
					#[codec(index = 0)]
					Xcm(runtime_types::staging_xcm::v4::location::Location),
					#[codec(index = 1)]
					Response(runtime_types::staging_xcm::v4::location::Location),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum QueryStatus<_0> {
					#[codec(index = 0)]
					Pending {
						responder: runtime_types::xcm::VersionedLocation,
						maybe_match_querier:
							::core::option::Option<runtime_types::xcm::VersionedLocation>,
						maybe_notify:
							::core::option::Option<(::core::primitive::u8, ::core::primitive::u8)>,
						timeout: _0,
					},
					#[codec(index = 1)]
					VersionNotifier {
						origin: runtime_types::xcm::VersionedLocation,
						is_active: ::core::primitive::bool,
					},
					#[codec(index = 2)]
					Ready { response: runtime_types::xcm::VersionedResponse, at: _0 },
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct RemoteLockedFungibleRecord<_0> {
					pub amount: ::core::primitive::u128,
					pub owner: runtime_types::xcm::VersionedLocation,
					pub locker: runtime_types::xcm::VersionedLocation,
					pub consumers: runtime_types::bounded_collections::bounded_vec::BoundedVec<(
						_0,
						::core::primitive::u128,
					)>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum VersionMigrationStage {
					#[codec(index = 0)]
					MigrateSupportedVersion,
					#[codec(index = 1)]
					MigrateVersionNotifiers,
					#[codec(index = 2)]
					NotifyCurrentTargets(
						::core::option::Option<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					),
					#[codec(index = 3)]
					MigrateAndNotifyOldTargets,
				}
			}
		}
		pub mod polkadot_core_primitives {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct CandidateHash(pub ::subxt::ext::subxt_core::utils::H256);
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct InboundDownwardMessage<_0> {
				pub sent_at: _0,
				pub msg: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct InboundHrmpMessage<_0> {
				pub sent_at: _0,
				pub data: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct OutboundHrmpMessage<_0> {
				pub recipient: _0,
				pub data: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			}
		}
		pub mod polkadot_parachain_primitives {
			use super::runtime_types;
			pub mod primitives {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct HeadData(
					pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct HrmpChannelId {
					pub sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
					pub recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Id(pub ::core::primitive::u32);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ValidationCode(
					pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ValidationCodeHash(pub ::subxt::ext::subxt_core::utils::H256);
			}
		}
		pub mod polkadot_primitives {
			use super::runtime_types;
			pub mod v8 {
				use super::runtime_types;
				pub mod assignment_app {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Public(pub [::core::primitive::u8; 32usize]);
				}
				pub mod async_backing {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct AsyncBackingParams {
						pub max_candidate_depth: ::core::primitive::u32,
						pub allowed_ancestry_len: ::core::primitive::u32,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct BackingState<_0, _1> {
                        pub constraints: runtime_types::polkadot_primitives::v8::async_backing::Constraints<
                            _1,
                        >,
                        pub pending_availability: ::subxt::ext::subxt_core::alloc::vec::Vec<
                            runtime_types::polkadot_primitives::v8::async_backing::CandidatePendingAvailability<
                                _0,
                                _1,
                            >,
                        >,
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CandidatePendingAvailability<_0, _1> {
						pub candidate_hash: runtime_types::polkadot_core_primitives::CandidateHash,
						pub descriptor:
							runtime_types::polkadot_primitives::v8::CandidateDescriptor<_0>,
						pub commitments:
							runtime_types::polkadot_primitives::v8::CandidateCommitments<_1>,
						pub relay_parent_number: _1,
						pub max_pov_size: ::core::primitive::u32,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Constraints<_0> {
                        pub min_relay_parent_number: _0,
                        pub max_pov_size: ::core::primitive::u32,
                        pub max_code_size: ::core::primitive::u32,
                        pub ump_remaining: ::core::primitive::u32,
                        pub ump_remaining_bytes: ::core::primitive::u32,
                        pub max_ump_num_per_candidate: ::core::primitive::u32,
                        pub dmp_remaining_messages: ::subxt::ext::subxt_core::alloc::vec::Vec<
                            _0,
                        >,
                        pub hrmp_inbound: runtime_types::polkadot_primitives::v8::async_backing::InboundHrmpLimitations<
                            _0,
                        >,
                        pub hrmp_channels_out: ::subxt::ext::subxt_core::alloc::vec::Vec<
                            (
                                runtime_types::polkadot_parachain_primitives::primitives::Id,
                                runtime_types::polkadot_primitives::v8::async_backing::OutboundHrmpChannelLimitations,
                            ),
                        >,
                        pub max_hrmp_num_per_candidate: ::core::primitive::u32,
                        pub required_parent: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                        pub validation_code_hash: runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                        pub upgrade_restriction: ::core::option::Option<
                            runtime_types::polkadot_primitives::v8::UpgradeRestriction,
                        >,
                        pub future_validation_code: ::core::option::Option<
                            (
                                _0,
                                runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                            ),
                        >,
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct InboundHrmpLimitations<_0> {
						pub valid_watermarks: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct OutboundHrmpChannelLimitations {
						pub bytes_remaining: ::core::primitive::u32,
						pub messages_remaining: ::core::primitive::u32,
					}
				}
				pub mod collator_app {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Public(pub [::core::primitive::u8; 32usize]);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Signature(pub [::core::primitive::u8; 64usize]);
				}
				pub mod executor_params {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum ExecutorParam {
						#[codec(index = 1)]
						MaxMemoryPages(::core::primitive::u32),
						#[codec(index = 2)]
						StackLogicalMax(::core::primitive::u32),
						#[codec(index = 3)]
						StackNativeMax(::core::primitive::u32),
						#[codec(index = 4)]
						PrecheckingMaxMemory(::core::primitive::u64),
						#[codec(index = 5)]
						PvfPrepTimeout(
							runtime_types::polkadot_primitives::v8::PvfPrepKind,
							::core::primitive::u64,
						),
						#[codec(index = 6)]
						PvfExecTimeout(
							runtime_types::polkadot_primitives::v8::PvfExecKind,
							::core::primitive::u64,
						),
						#[codec(index = 7)]
						WasmExtBulkMemory,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct ExecutorParams(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::polkadot_primitives::v8::executor_params::ExecutorParam,
						>,
					);
				}
				pub mod signed {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct UncheckedSigned<_0, _1> {
						pub payload: _0,
						pub validator_index: runtime_types::polkadot_primitives::v8::ValidatorIndex,
						pub signature:
							runtime_types::polkadot_primitives::v8::validator_app::Signature,
						#[codec(skip)]
						pub __ignore: ::core::marker::PhantomData<_1>,
					}
				}
				pub mod slashing {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct DisputeProof {
						pub time_slot:
							runtime_types::polkadot_primitives::v8::slashing::DisputesTimeSlot,
						pub kind:
							runtime_types::polkadot_primitives::v8::slashing::SlashingOffenceKind,
						pub validator_index: runtime_types::polkadot_primitives::v8::ValidatorIndex,
						pub validator_id:
							runtime_types::polkadot_primitives::v8::validator_app::Public,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct DisputesTimeSlot {
						pub session_index: ::core::primitive::u32,
						pub candidate_hash: runtime_types::polkadot_core_primitives::CandidateHash,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct OpaqueKeyOwnershipProof(
						pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct PendingSlashes {
						pub keys: ::subxt::ext::subxt_core::utils::KeyedVec<
							runtime_types::polkadot_primitives::v8::ValidatorIndex,
							runtime_types::polkadot_primitives::v8::validator_app::Public,
						>,
						pub kind:
							runtime_types::polkadot_primitives::v8::slashing::SlashingOffenceKind,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum SlashingOffenceKind {
						#[codec(index = 0)]
						ForInvalid,
						#[codec(index = 1)]
						AgainstValid,
					}
				}
				pub mod validator_app {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Public(pub [::core::primitive::u8; 32usize]);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Signature(pub [::core::primitive::u8; 64usize]);
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct ApprovalVotingParams {
					pub max_approval_coalesce_count: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AvailabilityBitfield(
					pub  ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
				);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BackedCandidate<_0> {
					pub candidate:
						runtime_types::polkadot_primitives::v8::CommittedCandidateReceipt<_0>,
					pub validity_votes: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::ValidityAttestation,
					>,
					pub validator_indices: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CandidateCommitments<_0> {
					pub upward_messages:
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						>,
					pub horizontal_messages:
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::polkadot_core_primitives::OutboundHrmpMessage<
								runtime_types::polkadot_parachain_primitives::primitives::Id,
							>,
						>,
					pub new_validation_code: ::core::option::Option<
						runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
					>,
					pub head_data:
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
					pub processed_downward_messages: ::core::primitive::u32,
					pub hrmp_watermark: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CandidateDescriptor<_0> {
                    pub para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                    pub relay_parent: _0,
                    pub collator: runtime_types::polkadot_primitives::v8::collator_app::Public,
                    pub persisted_validation_data_hash: ::subxt::ext::subxt_core::utils::H256,
                    pub pov_hash: ::subxt::ext::subxt_core::utils::H256,
                    pub erasure_root: ::subxt::ext::subxt_core::utils::H256,
                    pub signature: runtime_types::polkadot_primitives::v8::collator_app::Signature,
                    pub para_head: ::subxt::ext::subxt_core::utils::H256,
                    pub validation_code_hash: runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                }
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum CandidateEvent<_0> {
					#[codec(index = 0)]
					CandidateBacked(
						runtime_types::polkadot_primitives::v8::CandidateReceipt<_0>,
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
						runtime_types::polkadot_primitives::v8::CoreIndex,
						runtime_types::polkadot_primitives::v8::GroupIndex,
					),
					#[codec(index = 1)]
					CandidateIncluded(
						runtime_types::polkadot_primitives::v8::CandidateReceipt<_0>,
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
						runtime_types::polkadot_primitives::v8::CoreIndex,
						runtime_types::polkadot_primitives::v8::GroupIndex,
					),
					#[codec(index = 2)]
					CandidateTimedOut(
						runtime_types::polkadot_primitives::v8::CandidateReceipt<_0>,
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
						runtime_types::polkadot_primitives::v8::CoreIndex,
					),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CandidateReceipt<_0> {
					pub descriptor: runtime_types::polkadot_primitives::v8::CandidateDescriptor<_0>,
					pub commitments_hash: ::subxt::ext::subxt_core::utils::H256,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CommittedCandidateReceipt<_0> {
					pub descriptor: runtime_types::polkadot_primitives::v8::CandidateDescriptor<_0>,
					pub commitments: runtime_types::polkadot_primitives::v8::CandidateCommitments<
						::core::primitive::u32,
					>,
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct CoreIndex(pub ::core::primitive::u32);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum CoreState<_0, _1> {
					#[codec(index = 0)]
					Occupied(runtime_types::polkadot_primitives::v8::OccupiedCore<_0, _1>),
					#[codec(index = 1)]
					Scheduled(runtime_types::polkadot_primitives::v8::ScheduledCore),
					#[codec(index = 2)]
					Free,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct DisputeState<_0> {
					pub validators_for: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub validators_against: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub start: _0,
					pub concluded_at: ::core::option::Option<_0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum DisputeStatement {
					#[codec(index = 0)]
					Valid(runtime_types::polkadot_primitives::v8::ValidDisputeStatementKind),
					#[codec(index = 1)]
					Invalid(runtime_types::polkadot_primitives::v8::InvalidDisputeStatementKind),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct DisputeStatementSet {
					pub candidate_hash: runtime_types::polkadot_core_primitives::CandidateHash,
					pub session: ::core::primitive::u32,
					pub statements: ::subxt::ext::subxt_core::alloc::vec::Vec<(
						runtime_types::polkadot_primitives::v8::DisputeStatement,
						runtime_types::polkadot_primitives::v8::ValidatorIndex,
						runtime_types::polkadot_primitives::v8::validator_app::Signature,
					)>,
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct GroupIndex(pub ::core::primitive::u32);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct GroupRotationInfo<_0> {
					pub session_start_block: _0,
					pub group_rotation_frequency: _0,
					pub now: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct IndexedVec<_0, _1>(
					pub ::subxt::ext::subxt_core::alloc::vec::Vec<_1>,
					#[codec(skip)] pub ::core::marker::PhantomData<_0>,
				);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct InherentData<_0> {
					pub bitfields: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::signed::UncheckedSigned<
							runtime_types::polkadot_primitives::v8::AvailabilityBitfield,
							runtime_types::polkadot_primitives::v8::AvailabilityBitfield,
						>,
					>,
					pub backed_candidates: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::BackedCandidate<
							::subxt::ext::subxt_core::utils::H256,
						>,
					>,
					pub disputes: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::DisputeStatementSet,
					>,
					pub parent_header: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum InvalidDisputeStatementKind {
					#[codec(index = 0)]
					Explicit,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct OccupiedCore<_0, _1> {
					pub next_up_on_available: ::core::option::Option<
						runtime_types::polkadot_primitives::v8::ScheduledCore,
					>,
					pub occupied_since: _1,
					pub time_out_at: _1,
					pub next_up_on_time_out: ::core::option::Option<
						runtime_types::polkadot_primitives::v8::ScheduledCore,
					>,
					pub availability: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub group_responsible: runtime_types::polkadot_primitives::v8::GroupIndex,
					pub candidate_hash: runtime_types::polkadot_core_primitives::CandidateHash,
					pub candidate_descriptor:
						runtime_types::polkadot_primitives::v8::CandidateDescriptor<_0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum OccupiedCoreAssumption {
					#[codec(index = 0)]
					Included,
					#[codec(index = 1)]
					TimedOut,
					#[codec(index = 2)]
					Free,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PersistedValidationData<_0, _1> {
					pub parent_head:
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
					pub relay_parent_number: _1,
					pub relay_parent_storage_root: _0,
					pub max_pov_size: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PvfCheckStatement {
                    pub accept: ::core::primitive::bool,
                    pub subject: runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                    pub session_index: ::core::primitive::u32,
                    pub validator_index: runtime_types::polkadot_primitives::v8::ValidatorIndex,
                }
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum PvfExecKind {
					#[codec(index = 0)]
					Backing,
					#[codec(index = 1)]
					Approval,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum PvfPrepKind {
					#[codec(index = 0)]
					Precheck,
					#[codec(index = 1)]
					Prepare,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ScheduledCore {
					pub para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
					pub collator: ::core::option::Option<
						runtime_types::polkadot_primitives::v8::collator_app::Public,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SchedulerParams<_0> {
					pub group_rotation_frequency: _0,
					pub paras_availability_period: _0,
					pub max_validators_per_core: ::core::option::Option<_0>,
					pub lookahead: ::core::primitive::u32,
					pub num_cores: ::core::primitive::u32,
					pub max_availability_timeouts: ::core::primitive::u32,
					pub on_demand_queue_max_size: ::core::primitive::u32,
					pub on_demand_target_queue_utilization:
						runtime_types::sp_arithmetic::per_things::Perbill,
					pub on_demand_fee_variability:
						runtime_types::sp_arithmetic::per_things::Perbill,
					pub on_demand_base_fee: ::core::primitive::u128,
					pub ttl: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ScrapedOnChainVotes<_0> {
					pub session: ::core::primitive::u32,
					pub backing_validators_per_candidate:
						::subxt::ext::subxt_core::alloc::vec::Vec<(
							runtime_types::polkadot_primitives::v8::CandidateReceipt<_0>,
							::subxt::ext::subxt_core::alloc::vec::Vec<(
								runtime_types::polkadot_primitives::v8::ValidatorIndex,
								runtime_types::polkadot_primitives::v8::ValidityAttestation,
							)>,
						)>,
					pub disputes: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::DisputeStatementSet,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SessionInfo {
					pub active_validator_indices: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::ValidatorIndex,
					>,
					pub random_seed: [::core::primitive::u8; 32usize],
					pub dispute_period: ::core::primitive::u32,
					pub validators: runtime_types::polkadot_primitives::v8::IndexedVec<
						runtime_types::polkadot_primitives::v8::ValidatorIndex,
						runtime_types::polkadot_primitives::v8::validator_app::Public,
					>,
					pub discovery_keys: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::sp_authority_discovery::app::Public,
					>,
					pub assignment_keys: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::assignment_app::Public,
					>,
					pub validator_groups: runtime_types::polkadot_primitives::v8::IndexedVec<
						runtime_types::polkadot_primitives::v8::GroupIndex,
						::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::polkadot_primitives::v8::ValidatorIndex,
						>,
					>,
					pub n_cores: ::core::primitive::u32,
					pub zeroth_delay_tranche_width: ::core::primitive::u32,
					pub relay_vrf_modulo_samples: ::core::primitive::u32,
					pub n_delay_tranches: ::core::primitive::u32,
					pub no_show_slots: ::core::primitive::u32,
					pub needed_approvals: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum UpgradeGoAhead {
					#[codec(index = 0)]
					Abort,
					#[codec(index = 1)]
					GoAhead,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum UpgradeRestriction {
					#[codec(index = 0)]
					Present,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum ValidDisputeStatementKind {
					#[codec(index = 0)]
					Explicit,
					#[codec(index = 1)]
					BackingSeconded(::subxt::ext::subxt_core::utils::H256),
					#[codec(index = 2)]
					BackingValid(::subxt::ext::subxt_core::utils::H256),
					#[codec(index = 3)]
					ApprovalChecking,
					#[codec(index = 4)]
					ApprovalCheckingMultipleCandidates(
						::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::polkadot_core_primitives::CandidateHash,
						>,
					),
				}
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct ValidatorIndex(pub ::core::primitive::u32);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum ValidityAttestation {
					#[codec(index = 1)]
					Implicit(runtime_types::polkadot_primitives::v8::validator_app::Signature),
					#[codec(index = 2)]
					Explicit(runtime_types::polkadot_primitives::v8::validator_app::Signature),
				}
			}
		}
		pub mod polkadot_runtime_common {
			use super::runtime_types;
			pub mod assigned_slots {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
                        assign_perm_parachain_slot {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 1)]
                        assign_temp_parachain_slot {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            lease_period_start: runtime_types::polkadot_runtime_common::assigned_slots::SlotLeasePeriodStart,
                        },
                        #[codec(index = 2)]
                        unassign_parachain_slot {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 3)]
                        set_max_permanent_slots { slots: ::core::primitive::u32 },
                        #[codec(index = 4)]
                        set_max_temporary_slots { slots: ::core::primitive::u32 },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						ParaDoesntExist,
						#[codec(index = 1)]
						NotParathread,
						#[codec(index = 2)]
						CannotUpgrade,
						#[codec(index = 3)]
						CannotDowngrade,
						#[codec(index = 4)]
						SlotAlreadyAssigned,
						#[codec(index = 5)]
						SlotNotAssigned,
						#[codec(index = 6)]
						OngoingLeaseExists,
						#[codec(index = 7)]
						MaxPermanentSlotsExceeded,
						#[codec(index = 8)]
						MaxTemporarySlotsExceeded,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						PermanentSlotAssigned(
							runtime_types::polkadot_parachain_primitives::primitives::Id,
						),
						#[codec(index = 1)]
						TemporarySlotAssigned(
							runtime_types::polkadot_parachain_primitives::primitives::Id,
						),
						#[codec(index = 2)]
						MaxPermanentSlotsChanged { slots: ::core::primitive::u32 },
						#[codec(index = 3)]
						MaxTemporarySlotsChanged { slots: ::core::primitive::u32 },
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ParachainTemporarySlot<_0, _1> {
					pub manager: _0,
					pub period_begin: _1,
					pub period_count: _1,
					pub last_lease: ::core::option::Option<_1>,
					pub lease_count: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum SlotLeasePeriodStart {
					#[codec(index = 0)]
					Current,
					#[codec(index = 1)]
					Next,
				}
			}
			pub mod auctions {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						new_auction {
							#[codec(compact)]
							duration: ::core::primitive::u32,
							#[codec(compact)]
							lease_period_index: ::core::primitive::u32,
						},
						#[codec(index = 1)]
						bid {
							#[codec(compact)]
							para: runtime_types::polkadot_parachain_primitives::primitives::Id,
							#[codec(compact)]
							auction_index: ::core::primitive::u32,
							#[codec(compact)]
							first_slot: ::core::primitive::u32,
							#[codec(compact)]
							last_slot: ::core::primitive::u32,
							#[codec(compact)]
							amount: ::core::primitive::u128,
						},
						#[codec(index = 2)]
						cancel_auction,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						AuctionInProgress,
						#[codec(index = 1)]
						LeasePeriodInPast,
						#[codec(index = 2)]
						ParaNotRegistered,
						#[codec(index = 3)]
						NotCurrentAuction,
						#[codec(index = 4)]
						NotAuction,
						#[codec(index = 5)]
						AuctionEnded,
						#[codec(index = 6)]
						AlreadyLeasedOut,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						AuctionStarted {
							auction_index: ::core::primitive::u32,
							lease_period: ::core::primitive::u32,
							ending: ::core::primitive::u32,
						},
						#[codec(index = 1)]
						AuctionClosed { auction_index: ::core::primitive::u32 },
						#[codec(index = 2)]
						Reserved {
							bidder: ::sp_core::crypto::AccountId32,
							extra_reserved: ::core::primitive::u128,
							total_amount: ::core::primitive::u128,
						},
						#[codec(index = 3)]
						Unreserved {
							bidder: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 4)]
						ReserveConfiscated {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							leaser: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 5)]
						BidAccepted {
							bidder: ::sp_core::crypto::AccountId32,
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							amount: ::core::primitive::u128,
							first_slot: ::core::primitive::u32,
							last_slot: ::core::primitive::u32,
						},
						#[codec(index = 6)]
						WinningOffset {
							auction_index: ::core::primitive::u32,
							block_number: ::core::primitive::u32,
						},
					}
				}
			}
			pub mod crowdloan {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						create {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
							#[codec(compact)]
							cap: ::core::primitive::u128,
							#[codec(compact)]
							first_period: ::core::primitive::u32,
							#[codec(compact)]
							last_period: ::core::primitive::u32,
							#[codec(compact)]
							end: ::core::primitive::u32,
							verifier:
								::core::option::Option<runtime_types::sp_runtime::MultiSigner>,
						},
						#[codec(index = 1)]
						contribute {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
							#[codec(compact)]
							value: ::core::primitive::u128,
							signature:
								::core::option::Option<runtime_types::sp_runtime::MultiSignature>,
						},
						#[codec(index = 2)]
						withdraw {
							who: ::sp_core::crypto::AccountId32,
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 3)]
						refund {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 4)]
						dissolve {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 5)]
						edit {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
							#[codec(compact)]
							cap: ::core::primitive::u128,
							#[codec(compact)]
							first_period: ::core::primitive::u32,
							#[codec(compact)]
							last_period: ::core::primitive::u32,
							#[codec(compact)]
							end: ::core::primitive::u32,
							verifier:
								::core::option::Option<runtime_types::sp_runtime::MultiSigner>,
						},
						#[codec(index = 6)]
						add_memo {
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
							memo: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						},
						#[codec(index = 7)]
						poke { index: runtime_types::polkadot_parachain_primitives::primitives::Id },
						#[codec(index = 8)]
						contribute_all {
							#[codec(compact)]
							index: runtime_types::polkadot_parachain_primitives::primitives::Id,
							signature:
								::core::option::Option<runtime_types::sp_runtime::MultiSignature>,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						FirstPeriodInPast,
						#[codec(index = 1)]
						FirstPeriodTooFarInFuture,
						#[codec(index = 2)]
						LastPeriodBeforeFirstPeriod,
						#[codec(index = 3)]
						LastPeriodTooFarInFuture,
						#[codec(index = 4)]
						CannotEndInPast,
						#[codec(index = 5)]
						EndTooFarInFuture,
						#[codec(index = 6)]
						Overflow,
						#[codec(index = 7)]
						ContributionTooSmall,
						#[codec(index = 8)]
						InvalidParaId,
						#[codec(index = 9)]
						CapExceeded,
						#[codec(index = 10)]
						ContributionPeriodOver,
						#[codec(index = 11)]
						InvalidOrigin,
						#[codec(index = 12)]
						NotParachain,
						#[codec(index = 13)]
						LeaseActive,
						#[codec(index = 14)]
						BidOrLeaseActive,
						#[codec(index = 15)]
						FundNotEnded,
						#[codec(index = 16)]
						NoContributions,
						#[codec(index = 17)]
						NotReadyToDissolve,
						#[codec(index = 18)]
						InvalidSignature,
						#[codec(index = 19)]
						MemoTooLarge,
						#[codec(index = 20)]
						AlreadyInNewRaise,
						#[codec(index = 21)]
						VrfDelayInProgress,
						#[codec(index = 22)]
						NoLeasePeriod,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						Created {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 1)]
						Contributed {
							who: ::sp_core::crypto::AccountId32,
							fund_index:
								runtime_types::polkadot_parachain_primitives::primitives::Id,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 2)]
						Withdrew {
							who: ::sp_core::crypto::AccountId32,
							fund_index:
								runtime_types::polkadot_parachain_primitives::primitives::Id,
							amount: ::core::primitive::u128,
						},
						#[codec(index = 3)]
						PartiallyRefunded {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 4)]
						AllRefunded {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 5)]
						Dissolved {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 6)]
						HandleBidResult {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							result: ::core::result::Result<
								(),
								runtime_types::sp_runtime::DispatchError,
							>,
						},
						#[codec(index = 7)]
						Edited {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 8)]
						MemoUpdated {
							who: ::sp_core::crypto::AccountId32,
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							memo: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						},
						#[codec(index = 9)]
						AddedToNewRaise {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct FundInfo<_0, _1, _2, _3> {
					pub depositor: _0,
					pub verifier: ::core::option::Option<runtime_types::sp_runtime::MultiSigner>,
					pub deposit: _1,
					pub raised: _1,
					pub end: _2,
					pub cap: _1,
					pub last_contribution:
						runtime_types::polkadot_runtime_common::crowdloan::LastContribution<_2>,
					pub first_period: _3,
					pub last_period: _3,
					pub fund_index: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum LastContribution<_0> {
					#[codec(index = 0)]
					Never,
					#[codec(index = 1)]
					PreEnding(::core::primitive::u32),
					#[codec(index = 2)]
					Ending(_0),
				}
			}
			pub mod identity_migrator {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						reap_identity { who: ::sp_core::crypto::AccountId32 },
						#[codec(index = 1)]
						poke_deposit { who: ::sp_core::crypto::AccountId32 },
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						IdentityReaped { who: ::sp_core::crypto::AccountId32 },
						#[codec(index = 1)]
						DepositUpdated {
							who: ::sp_core::crypto::AccountId32,
							identity: ::core::primitive::u128,
							subs: ::core::primitive::u128,
						},
					}
				}
			}
			pub mod impls {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum VersionedLocatableAsset {
					#[codec(index = 3)]
					V3 {
						location: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						asset_id: runtime_types::xcm::v3::multiasset::AssetId,
					},
					#[codec(index = 4)]
					V4 {
						location: runtime_types::staging_xcm::v4::location::Location,
						asset_id: runtime_types::staging_xcm::v4::asset::AssetId,
					},
				}
			}
			pub mod paras_registrar {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
                        register {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            genesis_head: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                            validation_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                        },
                        #[codec(index = 1)]
                        force_register {
                            who: ::sp_core::crypto::AccountId32,
                            deposit: ::core::primitive::u128,
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            genesis_head: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                            validation_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                        },
                        #[codec(index = 2)]
                        deregister {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 3)]
                        swap {
                            id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            other: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 4)]
                        remove_lock {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 5)]
                        reserve,
                        #[codec(index = 6)]
                        add_lock {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 7)]
                        schedule_code_upgrade {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                        },
                        #[codec(index = 8)]
                        set_current_head {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_head: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                        },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						NotRegistered,
						#[codec(index = 1)]
						AlreadyRegistered,
						#[codec(index = 2)]
						NotOwner,
						#[codec(index = 3)]
						CodeTooLarge,
						#[codec(index = 4)]
						HeadDataTooLarge,
						#[codec(index = 5)]
						NotParachain,
						#[codec(index = 6)]
						NotParathread,
						#[codec(index = 7)]
						CannotDeregister,
						#[codec(index = 8)]
						CannotDowngrade,
						#[codec(index = 9)]
						CannotUpgrade,
						#[codec(index = 10)]
						ParaLocked,
						#[codec(index = 11)]
						NotReserved,
						#[codec(index = 12)]
						InvalidCode,
						#[codec(index = 13)]
						CannotSwap,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						Registered {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							manager: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 1)]
						Deregistered {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 2)]
						Reserved {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							who: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 3)]
						Swapped {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							other_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ParaInfo<_0, _1> {
					pub manager: _0,
					pub deposit: _1,
					pub locked: ::core::option::Option<::core::primitive::bool>,
				}
			}
			pub mod paras_sudo_wrapper {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						sudo_schedule_para_initialize {
							id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							genesis:
								runtime_types::polkadot_runtime_parachains::paras::ParaGenesisArgs,
						},
						#[codec(index = 1)]
						sudo_schedule_para_cleanup {
							id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 2)]
						sudo_schedule_parathread_upgrade {
							id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 3)]
						sudo_schedule_parachain_downgrade {
							id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 4)]
						sudo_queue_downward_xcm {
							id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							xcm: ::subxt::ext::subxt_core::alloc::boxed::Box<
								runtime_types::xcm::VersionedXcm,
							>,
						},
						#[codec(index = 5)]
						sudo_establish_hrmp_channel {
							sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
							recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
							max_capacity: ::core::primitive::u32,
							max_message_size: ::core::primitive::u32,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						ParaDoesntExist,
						#[codec(index = 1)]
						ParaAlreadyExists,
						#[codec(index = 2)]
						ExceedsMaxMessageSize,
						#[codec(index = 3)]
						CouldntCleanup,
						#[codec(index = 4)]
						NotParathread,
						#[codec(index = 5)]
						NotParachain,
						#[codec(index = 6)]
						CannotUpgrade,
						#[codec(index = 7)]
						CannotDowngrade,
						#[codec(index = 8)]
						TooManyCores,
					}
				}
			}
			pub mod slots {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						force_lease {
							para: runtime_types::polkadot_parachain_primitives::primitives::Id,
							leaser: ::sp_core::crypto::AccountId32,
							amount: ::core::primitive::u128,
							period_begin: ::core::primitive::u32,
							period_count: ::core::primitive::u32,
						},
						#[codec(index = 1)]
						clear_all_leases {
							para: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 2)]
						trigger_onboard {
							para: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						ParaNotOnboarding,
						#[codec(index = 1)]
						LeaseError,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						NewLeasePeriod { lease_period: ::core::primitive::u32 },
						#[codec(index = 1)]
						Leased {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							leaser: ::sp_core::crypto::AccountId32,
							period_begin: ::core::primitive::u32,
							period_count: ::core::primitive::u32,
							extra_reserved: ::core::primitive::u128,
							total_amount: ::core::primitive::u128,
						},
					}
				}
			}
		}
		pub mod polkadot_runtime_parachains {
			use super::runtime_types;
			pub mod assigner_coretime {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						AssignmentsEmpty,
						#[codec(index = 1)]
						OverScheduled,
						#[codec(index = 2)]
						UnderScheduled,
						#[codec(index = 3)]
						DisallowedInsert,
						#[codec(index = 4)]
						DuplicateInsert,
						#[codec(index = 5)]
						AssignmentsNotSorted,
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AssignmentState {
					pub ratio:
						runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600,
					pub remaining:
						runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CoreDescriptor<_0> {
                    pub queue: ::core::option::Option<
                        runtime_types::polkadot_runtime_parachains::assigner_coretime::QueueDescriptor<
                            _0,
                        >,
                    >,
                    pub current_work: ::core::option::Option<
                        runtime_types::polkadot_runtime_parachains::assigner_coretime::WorkState<
                            _0,
                        >,
                    >,
                }
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct PartsOf57600(pub ::core::primitive::u16);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct QueueDescriptor<_0> {
					pub first: _0,
					pub last: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Schedule<_0> {
					pub assignments: ::subxt::ext::subxt_core::alloc::vec::Vec<(
						runtime_types::pallet_broker::coretime_interface::CoreAssignment,
						runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600,
					)>,
					pub end_hint: ::core::option::Option<_0>,
					pub next_schedule: ::core::option::Option<_0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct WorkState<_0> {
                    pub assignments: ::subxt::ext::subxt_core::alloc::vec::Vec<
                        (
                            runtime_types::pallet_broker::coretime_interface::CoreAssignment,
                            runtime_types::polkadot_runtime_parachains::assigner_coretime::AssignmentState,
                        ),
                    >,
                    pub end_hint: ::core::option::Option<_0>,
                    pub pos: ::core::primitive::u16,
                    pub step: runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600,
                }
			}
			pub mod configuration {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
                        set_validation_upgrade_cooldown { new: ::core::primitive::u32 },
                        #[codec(index = 1)]
                        set_validation_upgrade_delay { new: ::core::primitive::u32 },
                        #[codec(index = 2)]
                        set_code_retention_period { new: ::core::primitive::u32 },
                        #[codec(index = 3)]
                        set_max_code_size { new: ::core::primitive::u32 },
                        #[codec(index = 4)]
                        set_max_pov_size { new: ::core::primitive::u32 },
                        #[codec(index = 5)]
                        set_max_head_data_size { new: ::core::primitive::u32 },
                        #[codec(index = 6)]
                        set_coretime_cores { new: ::core::primitive::u32 },
                        #[codec(index = 7)]
                        set_max_availability_timeouts { new: ::core::primitive::u32 },
                        #[codec(index = 8)]
                        set_group_rotation_frequency { new: ::core::primitive::u32 },
                        #[codec(index = 9)]
                        set_paras_availability_period { new: ::core::primitive::u32 },
                        #[codec(index = 11)]
                        set_scheduling_lookahead { new: ::core::primitive::u32 },
                        #[codec(index = 12)]
                        set_max_validators_per_core {
                            new: ::core::option::Option<::core::primitive::u32>,
                        },
                        #[codec(index = 13)]
                        set_max_validators {
                            new: ::core::option::Option<::core::primitive::u32>,
                        },
                        #[codec(index = 14)]
                        set_dispute_period { new: ::core::primitive::u32 },
                        #[codec(index = 15)]
                        set_dispute_post_conclusion_acceptance_period {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 18)]
                        set_no_show_slots { new: ::core::primitive::u32 },
                        #[codec(index = 19)]
                        set_n_delay_tranches { new: ::core::primitive::u32 },
                        #[codec(index = 20)]
                        set_zeroth_delay_tranche_width { new: ::core::primitive::u32 },
                        #[codec(index = 21)]
                        set_needed_approvals { new: ::core::primitive::u32 },
                        #[codec(index = 22)]
                        set_relay_vrf_modulo_samples { new: ::core::primitive::u32 },
                        #[codec(index = 23)]
                        set_max_upward_queue_count { new: ::core::primitive::u32 },
                        #[codec(index = 24)]
                        set_max_upward_queue_size { new: ::core::primitive::u32 },
                        #[codec(index = 25)]
                        set_max_downward_message_size { new: ::core::primitive::u32 },
                        #[codec(index = 27)]
                        set_max_upward_message_size { new: ::core::primitive::u32 },
                        #[codec(index = 28)]
                        set_max_upward_message_num_per_candidate {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 29)]
                        set_hrmp_open_request_ttl { new: ::core::primitive::u32 },
                        #[codec(index = 30)]
                        set_hrmp_sender_deposit { new: ::core::primitive::u128 },
                        #[codec(index = 31)]
                        set_hrmp_recipient_deposit { new: ::core::primitive::u128 },
                        #[codec(index = 32)]
                        set_hrmp_channel_max_capacity { new: ::core::primitive::u32 },
                        #[codec(index = 33)]
                        set_hrmp_channel_max_total_size { new: ::core::primitive::u32 },
                        #[codec(index = 34)]
                        set_hrmp_max_parachain_inbound_channels {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 36)]
                        set_hrmp_channel_max_message_size {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 37)]
                        set_hrmp_max_parachain_outbound_channels {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 39)]
                        set_hrmp_max_message_num_per_candidate {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 42)]
                        set_pvf_voting_ttl { new: ::core::primitive::u32 },
                        #[codec(index = 43)]
                        set_minimum_validation_upgrade_delay {
                            new: ::core::primitive::u32,
                        },
                        #[codec(index = 44)]
                        set_bypass_consistency_check { new: ::core::primitive::bool },
                        #[codec(index = 45)]
                        set_async_backing_params {
                            new: runtime_types::polkadot_primitives::v8::async_backing::AsyncBackingParams,
                        },
                        #[codec(index = 46)]
                        set_executor_params {
                            new: runtime_types::polkadot_primitives::v8::executor_params::ExecutorParams,
                        },
                        #[codec(index = 47)]
                        set_on_demand_base_fee { new: ::core::primitive::u128 },
                        #[codec(index = 48)]
                        set_on_demand_fee_variability {
                            new: runtime_types::sp_arithmetic::per_things::Perbill,
                        },
                        #[codec(index = 49)]
                        set_on_demand_queue_max_size { new: ::core::primitive::u32 },
                        #[codec(index = 50)]
                        set_on_demand_target_queue_utilization {
                            new: runtime_types::sp_arithmetic::per_things::Perbill,
                        },
                        #[codec(index = 51)]
                        set_on_demand_ttl { new: ::core::primitive::u32 },
                        #[codec(index = 52)]
                        set_minimum_backing_votes { new: ::core::primitive::u32 },
                        #[codec(index = 53)]
                        set_node_feature {
                            index: ::core::primitive::u8,
                            value: ::core::primitive::bool,
                        },
                        #[codec(index = 54)]
                        set_approval_voting_params {
                            new: runtime_types::polkadot_primitives::v8::ApprovalVotingParams,
                        },
                        #[codec(index = 55)]
                        set_scheduler_params {
                            new: runtime_types::polkadot_primitives::v8::SchedulerParams<
                                ::core::primitive::u32,
                            >,
                        },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						InvalidNewValue,
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct HostConfiguration<_0> {
					pub max_code_size: ::core::primitive::u32,
					pub max_head_data_size: ::core::primitive::u32,
					pub max_upward_queue_count: ::core::primitive::u32,
					pub max_upward_queue_size: ::core::primitive::u32,
					pub max_upward_message_size: ::core::primitive::u32,
					pub max_upward_message_num_per_candidate: ::core::primitive::u32,
					pub hrmp_max_message_num_per_candidate: ::core::primitive::u32,
					pub validation_upgrade_cooldown: _0,
					pub validation_upgrade_delay: _0,
					pub async_backing_params:
						runtime_types::polkadot_primitives::v8::async_backing::AsyncBackingParams,
					pub max_pov_size: ::core::primitive::u32,
					pub max_downward_message_size: ::core::primitive::u32,
					pub hrmp_max_parachain_outbound_channels: ::core::primitive::u32,
					pub hrmp_sender_deposit: ::core::primitive::u128,
					pub hrmp_recipient_deposit: ::core::primitive::u128,
					pub hrmp_channel_max_capacity: ::core::primitive::u32,
					pub hrmp_channel_max_total_size: ::core::primitive::u32,
					pub hrmp_max_parachain_inbound_channels: ::core::primitive::u32,
					pub hrmp_channel_max_message_size: ::core::primitive::u32,
					pub executor_params:
						runtime_types::polkadot_primitives::v8::executor_params::ExecutorParams,
					pub code_retention_period: _0,
					pub max_validators: ::core::option::Option<_0>,
					pub dispute_period: ::core::primitive::u32,
					pub dispute_post_conclusion_acceptance_period: _0,
					pub no_show_slots: ::core::primitive::u32,
					pub n_delay_tranches: ::core::primitive::u32,
					pub zeroth_delay_tranche_width: ::core::primitive::u32,
					pub needed_approvals: ::core::primitive::u32,
					pub relay_vrf_modulo_samples: ::core::primitive::u32,
					pub pvf_voting_ttl: ::core::primitive::u32,
					pub minimum_validation_upgrade_delay: _0,
					pub minimum_backing_votes: ::core::primitive::u32,
					pub node_features: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub approval_voting_params:
						runtime_types::polkadot_primitives::v8::ApprovalVotingParams,
					pub scheduler_params:
						runtime_types::polkadot_primitives::v8::SchedulerParams<_0>,
				}
			}
			pub mod coretime {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 1)]
                        request_core_count { count: ::core::primitive::u16 },
                        #[codec(index = 2)]
                        request_revenue_at { when: ::core::primitive::u32 },
                        #[codec(index = 4)]
                        assign_core {
                            core: ::core::primitive::u16,
                            begin: ::core::primitive::u32,
                            assignment: ::subxt::ext::subxt_core::alloc::vec::Vec<
                                (
                                    runtime_types::pallet_broker::coretime_interface::CoreAssignment,
                                    runtime_types::polkadot_runtime_parachains::assigner_coretime::PartsOf57600,
                                ),
                            >,
                            end_hint: ::core::option::Option<::core::primitive::u32>,
                        },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						NotBroker,
						#[codec(index = 1)]
						RequestedFutureRevenue,
						#[codec(index = 2)]
						AssetTransferFailed,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						RevenueInfoRequested { when: ::core::primitive::u32 },
						#[codec(index = 1)]
						CoreAssigned { core: runtime_types::polkadot_primitives::v8::CoreIndex },
					}
				}
			}
			pub mod disputes {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						force_unfreeze,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						DuplicateDisputeStatementSets,
						#[codec(index = 1)]
						AncientDisputeStatement,
						#[codec(index = 2)]
						ValidatorIndexOutOfBounds,
						#[codec(index = 3)]
						InvalidSignature,
						#[codec(index = 4)]
						DuplicateStatement,
						#[codec(index = 5)]
						SingleSidedDispute,
						#[codec(index = 6)]
						MaliciousBacker,
						#[codec(index = 7)]
						MissingBackingVotes,
						#[codec(index = 8)]
						UnconfirmedDispute,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						DisputeInitiated(
							runtime_types::polkadot_core_primitives::CandidateHash,
							runtime_types::polkadot_runtime_parachains::disputes::DisputeLocation,
						),
						#[codec(index = 1)]
						DisputeConcluded(
							runtime_types::polkadot_core_primitives::CandidateHash,
							runtime_types::polkadot_runtime_parachains::disputes::DisputeResult,
						),
						#[codec(index = 2)]
						Revert(::core::primitive::u32),
					}
				}
				pub mod slashing {
					use super::runtime_types;
					pub mod pallet {
						use super::runtime_types;
						#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
						pub enum Call {
							#[codec(index = 0)]
							report_dispute_lost_unsigned {
								dispute_proof: ::subxt::ext::subxt_core::alloc::boxed::Box<
									runtime_types::polkadot_primitives::v8::slashing::DisputeProof,
								>,
								key_owner_proof: ::sp_session::MembershipProof,
							},
						}
						#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
						pub enum Error {
							#[codec(index = 0)]
							InvalidKeyOwnershipProof,
							#[codec(index = 1)]
							InvalidSessionIndex,
							#[codec(index = 2)]
							InvalidCandidateHash,
							#[codec(index = 3)]
							InvalidValidatorIndex,
							#[codec(index = 4)]
							ValidatorIndexIdMismatch,
							#[codec(index = 5)]
							DuplicateSlashingReport,
						}
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum DisputeLocation {
					#[codec(index = 0)]
					Local,
					#[codec(index = 1)]
					Remote,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum DisputeResult {
					#[codec(index = 0)]
					Valid,
					#[codec(index = 1)]
					Invalid,
				}
			}
			pub mod hrmp {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
                        hrmp_init_open_channel {
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            proposed_max_capacity: ::core::primitive::u32,
                            proposed_max_message_size: ::core::primitive::u32,
                        },
                        #[codec(index = 1)]
                        hrmp_accept_open_channel {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 2)]
                        hrmp_close_channel {
                            channel_id: runtime_types::polkadot_parachain_primitives::primitives::HrmpChannelId,
                        },
                        #[codec(index = 3)]
                        force_clean_hrmp {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            num_inbound: ::core::primitive::u32,
                            num_outbound: ::core::primitive::u32,
                        },
                        #[codec(index = 4)]
                        force_process_hrmp_open { channels: ::core::primitive::u32 },
                        #[codec(index = 5)]
                        force_process_hrmp_close { channels: ::core::primitive::u32 },
                        #[codec(index = 6)]
                        hrmp_cancel_open_request {
                            channel_id: runtime_types::polkadot_parachain_primitives::primitives::HrmpChannelId,
                            open_requests: ::core::primitive::u32,
                        },
                        #[codec(index = 7)]
                        force_open_hrmp_channel {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            max_capacity: ::core::primitive::u32,
                            max_message_size: ::core::primitive::u32,
                        },
                        #[codec(index = 8)]
                        establish_system_channel {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 9)]
                        poke_channel_deposits {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 10)]
                        establish_channel_with_system {
                            target_system_chain: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						OpenHrmpChannelToSelf,
						#[codec(index = 1)]
						OpenHrmpChannelInvalidRecipient,
						#[codec(index = 2)]
						OpenHrmpChannelZeroCapacity,
						#[codec(index = 3)]
						OpenHrmpChannelCapacityExceedsLimit,
						#[codec(index = 4)]
						OpenHrmpChannelZeroMessageSize,
						#[codec(index = 5)]
						OpenHrmpChannelMessageSizeExceedsLimit,
						#[codec(index = 6)]
						OpenHrmpChannelAlreadyExists,
						#[codec(index = 7)]
						OpenHrmpChannelAlreadyRequested,
						#[codec(index = 8)]
						OpenHrmpChannelLimitExceeded,
						#[codec(index = 9)]
						AcceptHrmpChannelDoesntExist,
						#[codec(index = 10)]
						AcceptHrmpChannelAlreadyConfirmed,
						#[codec(index = 11)]
						AcceptHrmpChannelLimitExceeded,
						#[codec(index = 12)]
						CloseHrmpChannelUnauthorized,
						#[codec(index = 13)]
						CloseHrmpChannelDoesntExist,
						#[codec(index = 14)]
						CloseHrmpChannelAlreadyUnderway,
						#[codec(index = 15)]
						CancelHrmpOpenChannelUnauthorized,
						#[codec(index = 16)]
						OpenHrmpChannelDoesntExist,
						#[codec(index = 17)]
						OpenHrmpChannelAlreadyConfirmed,
						#[codec(index = 18)]
						WrongWitness,
						#[codec(index = 19)]
						ChannelCreationNotAuthorized,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
                        OpenChannelRequested {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            proposed_max_capacity: ::core::primitive::u32,
                            proposed_max_message_size: ::core::primitive::u32,
                        },
                        #[codec(index = 1)]
                        OpenChannelCanceled {
                            by_parachain: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            channel_id: runtime_types::polkadot_parachain_primitives::primitives::HrmpChannelId,
                        },
                        #[codec(index = 2)]
                        OpenChannelAccepted {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 3)]
                        ChannelClosed {
                            by_parachain: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            channel_id: runtime_types::polkadot_parachain_primitives::primitives::HrmpChannelId,
                        },
                        #[codec(index = 4)]
                        HrmpChannelForceOpened {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            proposed_max_capacity: ::core::primitive::u32,
                            proposed_max_message_size: ::core::primitive::u32,
                        },
                        #[codec(index = 5)]
                        HrmpSystemChannelOpened {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            proposed_max_capacity: ::core::primitive::u32,
                            proposed_max_message_size: ::core::primitive::u32,
                        },
                        #[codec(index = 6)]
                        OpenChannelDepositsUpdated {
                            sender: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            recipient: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                    }
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct HrmpChannel {
					pub max_capacity: ::core::primitive::u32,
					pub max_total_size: ::core::primitive::u32,
					pub max_message_size: ::core::primitive::u32,
					pub msg_count: ::core::primitive::u32,
					pub total_size: ::core::primitive::u32,
					pub mqc_head: ::core::option::Option<::subxt::ext::subxt_core::utils::H256>,
					pub sender_deposit: ::core::primitive::u128,
					pub recipient_deposit: ::core::primitive::u128,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct HrmpOpenChannelRequest {
					pub confirmed: ::core::primitive::bool,
					pub _age: ::core::primitive::u32,
					pub sender_deposit: ::core::primitive::u128,
					pub max_message_size: ::core::primitive::u32,
					pub max_capacity: ::core::primitive::u32,
					pub max_total_size: ::core::primitive::u32,
				}
			}
			pub mod inclusion {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						ValidatorIndexOutOfBounds,
						#[codec(index = 1)]
						UnscheduledCandidate,
						#[codec(index = 2)]
						HeadDataTooLarge,
						#[codec(index = 3)]
						PrematureCodeUpgrade,
						#[codec(index = 4)]
						NewCodeTooLarge,
						#[codec(index = 5)]
						DisallowedRelayParent,
						#[codec(index = 6)]
						InvalidAssignment,
						#[codec(index = 7)]
						InvalidGroupIndex,
						#[codec(index = 8)]
						InsufficientBacking,
						#[codec(index = 9)]
						InvalidBacking,
						#[codec(index = 10)]
						ValidationDataHashMismatch,
						#[codec(index = 11)]
						IncorrectDownwardMessageHandling,
						#[codec(index = 12)]
						InvalidUpwardMessages,
						#[codec(index = 13)]
						HrmpWatermarkMishandling,
						#[codec(index = 14)]
						InvalidOutboundHrmp,
						#[codec(index = 15)]
						InvalidValidationCodeHash,
						#[codec(index = 16)]
						ParaHeadMismatch,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						CandidateBacked(
							runtime_types::polkadot_primitives::v8::CandidateReceipt<
								::subxt::ext::subxt_core::utils::H256,
							>,
							runtime_types::polkadot_parachain_primitives::primitives::HeadData,
							runtime_types::polkadot_primitives::v8::CoreIndex,
							runtime_types::polkadot_primitives::v8::GroupIndex,
						),
						#[codec(index = 1)]
						CandidateIncluded(
							runtime_types::polkadot_primitives::v8::CandidateReceipt<
								::subxt::ext::subxt_core::utils::H256,
							>,
							runtime_types::polkadot_parachain_primitives::primitives::HeadData,
							runtime_types::polkadot_primitives::v8::CoreIndex,
							runtime_types::polkadot_primitives::v8::GroupIndex,
						),
						#[codec(index = 2)]
						CandidateTimedOut(
							runtime_types::polkadot_primitives::v8::CandidateReceipt<
								::subxt::ext::subxt_core::utils::H256,
							>,
							runtime_types::polkadot_parachain_primitives::primitives::HeadData,
							runtime_types::polkadot_primitives::v8::CoreIndex,
						),
						#[codec(index = 3)]
						UpwardMessagesReceived {
							from: runtime_types::polkadot_parachain_primitives::primitives::Id,
							count: ::core::primitive::u32,
						},
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum AggregateMessageOrigin {
					#[codec(index = 0)]
					Ump(runtime_types::polkadot_runtime_parachains::inclusion::UmpQueueId),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CandidatePendingAvailability<_0, _1> {
					pub core: runtime_types::polkadot_primitives::v8::CoreIndex,
					pub hash: runtime_types::polkadot_core_primitives::CandidateHash,
					pub descriptor: runtime_types::polkadot_primitives::v8::CandidateDescriptor<_0>,
					pub commitments:
						runtime_types::polkadot_primitives::v8::CandidateCommitments<_1>,
					pub availability_votes: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub backers: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub relay_parent_number: _1,
					pub backed_in_number: _1,
					pub backing_group: runtime_types::polkadot_primitives::v8::GroupIndex,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum UmpQueueId {
					#[codec(index = 0)]
					Para(runtime_types::polkadot_parachain_primitives::primitives::Id),
				}
			}
			pub mod initializer {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						force_approve { up_to: ::core::primitive::u32 },
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BufferedSessionChange {
					pub validators: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::validator_app::Public,
					>,
					pub queued: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_primitives::v8::validator_app::Public,
					>,
					pub session_index: ::core::primitive::u32,
				}
			}
			pub mod on_demand {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						place_order_allow_death {
							max_amount: ::core::primitive::u128,
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
						#[codec(index = 1)]
						place_order_keep_alive {
							max_amount: ::core::primitive::u128,
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						QueueFull,
						#[codec(index = 1)]
						SpotPriceHigherThanMaxAmount,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
						OnDemandOrderPlaced {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							spot_price: ::core::primitive::u128,
							ordered_by: ::sp_core::crypto::AccountId32,
						},
						#[codec(index = 1)]
						SpotPriceSet { spot_price: ::core::primitive::u128 },
					}
				}
				pub mod types {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct CoreAffinityCount {
						pub core_index: runtime_types::polkadot_primitives::v8::CoreIndex,
						pub count: ::core::primitive::u32,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct EnqueuedOrder {
                        pub para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        pub idx: runtime_types::polkadot_runtime_parachains::on_demand::types::QueueIndex,
                    }
					#[derive(
						::codec::Decode,
						::codec::Encode,
						::subxt::ext::subxt_core::ext::codec::CompactAs,
						Clone,
						Debug,
						PartialEq,
					)]
					pub struct QueueIndex(pub ::core::primitive::u32);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct QueueStatusType {
                        pub traffic: runtime_types::sp_arithmetic::fixed_point::FixedU128,
                        pub next_index: runtime_types::polkadot_runtime_parachains::on_demand::types::QueueIndex,
                        pub smallest_index: runtime_types::polkadot_runtime_parachains::on_demand::types::QueueIndex,
                        pub freed_indices: ::subxt::ext::subxt_core::alloc::vec::Vec<
                            runtime_types::polkadot_runtime_parachains::on_demand::types::ReverseQueueIndex,
                        >,
                    }
					#[derive(
						::codec::Decode,
						::codec::Encode,
						::subxt::ext::subxt_core::ext::codec::CompactAs,
						Clone,
						Debug,
						PartialEq,
					)]
					pub struct ReverseQueueIndex(pub ::core::primitive::u32);
				}
			}
			pub mod origin {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Origin {
						#[codec(index = 0)]
						Parachain(runtime_types::polkadot_parachain_primitives::primitives::Id),
					}
				}
			}
			pub mod paras {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
                        force_set_current_code {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                        },
                        #[codec(index = 1)]
                        force_set_current_head {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_head: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                        },
                        #[codec(index = 2)]
                        force_schedule_code_upgrade {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                            relay_parent_number: ::core::primitive::u32,
                        },
                        #[codec(index = 3)]
                        force_note_new_head {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            new_head: runtime_types::polkadot_parachain_primitives::primitives::HeadData,
                        },
                        #[codec(index = 4)]
                        force_queue_action {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                        },
                        #[codec(index = 5)]
                        add_trusted_validation_code {
                            validation_code: runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
                        },
                        #[codec(index = 6)]
                        poke_unused_validation_code {
                            validation_code_hash: runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                        },
                        #[codec(index = 7)]
                        include_pvf_check_statement {
                            stmt: runtime_types::polkadot_primitives::v8::PvfCheckStatement,
                            signature: runtime_types::polkadot_primitives::v8::validator_app::Signature,
                        },
                        #[codec(index = 8)]
                        force_set_most_recent_context {
                            para: runtime_types::polkadot_parachain_primitives::primitives::Id,
                            context: ::core::primitive::u32,
                        },
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						NotRegistered,
						#[codec(index = 1)]
						CannotOnboard,
						#[codec(index = 2)]
						CannotOffboard,
						#[codec(index = 3)]
						CannotUpgrade,
						#[codec(index = 4)]
						CannotDowngrade,
						#[codec(index = 5)]
						PvfCheckStatementStale,
						#[codec(index = 6)]
						PvfCheckStatementFuture,
						#[codec(index = 7)]
						PvfCheckValidatorIndexOutOfBounds,
						#[codec(index = 8)]
						PvfCheckInvalidSignature,
						#[codec(index = 9)]
						PvfCheckDoubleVote,
						#[codec(index = 10)]
						PvfCheckSubjectInvalid,
						#[codec(index = 11)]
						CannotUpgradeCode,
						#[codec(index = 12)]
						InvalidCode,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Event {
						#[codec(index = 0)]
                        CurrentCodeUpdated(
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 1)]
                        CurrentHeadUpdated(
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 2)]
                        CodeUpgradeScheduled(
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 3)]
                        NewHeadNoted(
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 4)]
                        ActionQueued(
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                            ::core::primitive::u32,
                        ),
                        #[codec(index = 5)]
                        PvfCheckStarted(
                            runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 6)]
                        PvfCheckAccepted(
                            runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                        #[codec(index = 7)]
                        PvfCheckRejected(
                            runtime_types::polkadot_parachain_primitives::primitives::ValidationCodeHash,
                            runtime_types::polkadot_parachain_primitives::primitives::Id,
                        ),
                    }
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ParaGenesisArgs {
					pub genesis_head:
						runtime_types::polkadot_parachain_primitives::primitives::HeadData,
					pub validation_code:
						runtime_types::polkadot_parachain_primitives::primitives::ValidationCode,
					pub para_kind: ::core::primitive::bool,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum ParaLifecycle {
					#[codec(index = 0)]
					Onboarding,
					#[codec(index = 1)]
					Parathread,
					#[codec(index = 2)]
					Parachain,
					#[codec(index = 3)]
					UpgradingParathread,
					#[codec(index = 4)]
					DowngradingParachain,
					#[codec(index = 5)]
					OffboardingParathread,
					#[codec(index = 6)]
					OffboardingParachain,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ParaPastCodeMeta<_0> {
					pub upgrade_times: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_runtime_parachains::paras::ReplacementTimes<_0>,
					>,
					pub last_pruned: ::core::option::Option<_0>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PvfCheckActiveVoteState<_0> {
					pub votes_accept: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub votes_reject: ::subxt::ext::subxt_core::utils::bits::DecodedBits<
						::core::primitive::u8,
						::subxt::ext::subxt_core::utils::bits::Lsb0,
					>,
					pub age: ::core::primitive::u32,
					pub created_at: _0,
					pub causes: ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::polkadot_runtime_parachains::paras::PvfCheckCause<_0>,
					>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum PvfCheckCause<_0> {
					#[codec(index = 0)]
					Onboarding(runtime_types::polkadot_parachain_primitives::primitives::Id),
					#[codec(index = 1)]
					Upgrade {
						id: runtime_types::polkadot_parachain_primitives::primitives::Id,
						included_at: _0,
						upgrade_strategy:
							runtime_types::polkadot_runtime_parachains::paras::UpgradeStrategy,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ReplacementTimes<_0> {
					pub expected_at: _0,
					pub activated_at: _0,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum UpgradeStrategy {
					#[codec(index = 0)]
					SetGoAheadSignal,
					#[codec(index = 1)]
					ApplyAtExpectedBlock,
				}
			}
			pub mod paras_inherent {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {
						#[codec(index = 0)]
						enter {
							data: runtime_types::polkadot_primitives::v8::InherentData<
								::sp_runtime::generic::Header<
									::core::primitive::u32,
									::sp_runtime::traits::BlakeTwo256,
								>,
							>,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						TooManyInclusionInherents,
						#[codec(index = 1)]
						InvalidParentHeader,
						#[codec(index = 2)]
						InherentOverweight,
						#[codec(index = 3)]
						CandidatesFilteredDuringExecution,
						#[codec(index = 4)]
						UnscheduledCandidate,
					}
				}
			}
			pub mod scheduler {
				use super::runtime_types;
				pub mod common {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Assignment {
						#[codec(index = 0)]
						Pool {
							para_id: runtime_types::polkadot_parachain_primitives::primitives::Id,
							core_index: runtime_types::polkadot_primitives::v8::CoreIndex,
						},
						#[codec(index = 1)]
						Bulk(runtime_types::polkadot_parachain_primitives::primitives::Id),
					}
				}
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum CoreOccupied<_0> {
						#[codec(index = 0)]
                        Free,
                        #[codec(index = 1)]
                        Paras(
                            runtime_types::polkadot_runtime_parachains::scheduler::pallet::ParasEntry<
                                _0,
                            >,
                        ),
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct ParasEntry<_0> {
                        pub assignment: runtime_types::polkadot_runtime_parachains::scheduler::common::Assignment,
                        pub availability_timeouts: ::core::primitive::u32,
                        pub ttl: _0,
                    }
				}
			}
			pub mod shared {
				use super::runtime_types;
				pub mod pallet {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Call {}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct AllowedRelayParentsTracker<_0, _1> {
					pub buffer: ::subxt::ext::subxt_core::alloc::vec::Vec<(_0, _0)>,
					pub latest_number: _1,
				}
			}
		}
		pub mod sp_arithmetic {
			use super::runtime_types;
			pub mod fixed_point {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct FixedI64(pub ::core::primitive::i64);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct FixedU128(pub ::core::primitive::u128);
			}
			pub mod per_things {
				use super::runtime_types;
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct PerU16(pub ::core::primitive::u16);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Perbill(pub ::core::primitive::u32);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Percent(pub ::core::primitive::u8);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Permill(pub ::core::primitive::u32);
				#[derive(
					::codec::Decode,
					::codec::Encode,
					::subxt::ext::subxt_core::ext::codec::CompactAs,
					Clone,
					Debug,
					PartialEq,
				)]
				pub struct Perquintill(pub ::core::primitive::u64);
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ArithmeticError {
				#[codec(index = 0)]
				Underflow,
				#[codec(index = 1)]
				Overflow,
				#[codec(index = 2)]
				DivisionByZero,
			}
		}
		pub mod sp_authority_discovery {
			use super::runtime_types;
			pub mod app {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Public(pub [::core::primitive::u8; 32usize]);
			}
		}
		pub mod sp_consensus_babe {
			use super::runtime_types;
			pub mod app {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Public(pub [::core::primitive::u8; 32usize]);
			}
			pub mod digests {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum NextConfigDescriptor {
					#[codec(index = 1)]
					V1 {
						c: (::core::primitive::u64, ::core::primitive::u64),
						allowed_slots: runtime_types::sp_consensus_babe::AllowedSlots,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum PreDigest {
					#[codec(index = 1)]
					Primary(runtime_types::sp_consensus_babe::digests::PrimaryPreDigest),
					#[codec(index = 2)]
					SecondaryPlain(
						runtime_types::sp_consensus_babe::digests::SecondaryPlainPreDigest,
					),
					#[codec(index = 3)]
					SecondaryVRF(runtime_types::sp_consensus_babe::digests::SecondaryVRFPreDigest),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PrimaryPreDigest {
					pub authority_index: ::core::primitive::u32,
					pub slot: runtime_types::sp_consensus_slots::Slot,
					pub vrf_signature: runtime_types::sp_core::sr25519::vrf::VrfSignature,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SecondaryPlainPreDigest {
					pub authority_index: ::core::primitive::u32,
					pub slot: runtime_types::sp_consensus_slots::Slot,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct SecondaryVRFPreDigest {
					pub authority_index: ::core::primitive::u32,
					pub slot: runtime_types::sp_consensus_slots::Slot,
					pub vrf_signature: runtime_types::sp_core::sr25519::vrf::VrfSignature,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum AllowedSlots {
				#[codec(index = 0)]
				PrimarySlots,
				#[codec(index = 1)]
				PrimaryAndSecondaryPlainSlots,
				#[codec(index = 2)]
				PrimaryAndSecondaryVRFSlots,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct BabeConfiguration {
				pub slot_duration: ::core::primitive::u64,
				pub epoch_length: ::core::primitive::u64,
				pub c: (::core::primitive::u64, ::core::primitive::u64),
				pub authorities: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					runtime_types::sp_consensus_babe::app::Public,
					::core::primitive::u64,
				)>,
				pub randomness: [::core::primitive::u8; 32usize],
				pub allowed_slots: runtime_types::sp_consensus_babe::AllowedSlots,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct BabeEpochConfiguration {
				pub c: (::core::primitive::u64, ::core::primitive::u64),
				pub allowed_slots: runtime_types::sp_consensus_babe::AllowedSlots,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Epoch {
				pub epoch_index: ::core::primitive::u64,
				pub start_slot: runtime_types::sp_consensus_slots::Slot,
				pub duration: ::core::primitive::u64,
				pub authorities: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					runtime_types::sp_consensus_babe::app::Public,
					::core::primitive::u64,
				)>,
				pub randomness: [::core::primitive::u8; 32usize],
				pub config: runtime_types::sp_consensus_babe::BabeEpochConfiguration,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct OpaqueKeyOwnershipProof(
				pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			);
		}
		pub mod sp_consensus_beefy {
			use super::runtime_types;
			pub mod commitment {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Commitment<_0> {
					pub payload: runtime_types::sp_consensus_beefy::payload::Payload,
					pub block_number: _0,
					pub validator_set_id: ::core::primitive::u64,
				}
			}
			pub mod ecdsa_crypto {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Public(pub [::core::primitive::u8; 33usize]);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Signature(pub [::core::primitive::u8; 65usize]);
			}
			pub mod mmr {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct BeefyAuthoritySet<_0> {
					pub id: ::core::primitive::u64,
					pub len: ::core::primitive::u32,
					pub keyset_commitment: _0,
				}
			}
			pub mod payload {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Payload(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<(
						[::core::primitive::u8; 2usize],
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					)>,
				);
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct DoubleVotingProof<_0, _1, _2> {
				pub first: runtime_types::sp_consensus_beefy::VoteMessage<_0, _1, _2>,
				pub second: runtime_types::sp_consensus_beefy::VoteMessage<_0, _1, _2>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ForkVotingProof<_0, _1, _2> {
				pub vote: runtime_types::sp_consensus_beefy::VoteMessage<
					::core::primitive::u32,
					_1,
					runtime_types::sp_consensus_beefy::ecdsa_crypto::Signature,
				>,
				pub ancestry_proof: _2,
				pub header: _0,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct FutureBlockVotingProof<_0, _1> {
				pub vote: runtime_types::sp_consensus_beefy::VoteMessage<
					_0,
					_1,
					runtime_types::sp_consensus_beefy::ecdsa_crypto::Signature,
				>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ValidatorSet<_0> {
				pub validators: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
				pub id: ::core::primitive::u64,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct VoteMessage<_0, _1, _2> {
				pub commitment: runtime_types::sp_consensus_beefy::commitment::Commitment<_0>,
				pub id: _1,
				pub signature: _2,
			}
		}
		pub mod sp_consensus_grandpa {
			use super::runtime_types;
			pub mod app {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Public(pub [::core::primitive::u8; 32usize]);
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Signature(pub [::core::primitive::u8; 64usize]);
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Equivocation<_0, _1> {
				#[codec(index = 0)]
				Prevote(
					runtime_types::finality_grandpa::Equivocation<
						runtime_types::sp_consensus_grandpa::app::Public,
						runtime_types::finality_grandpa::Prevote<_0, _1>,
						runtime_types::sp_consensus_grandpa::app::Signature,
					>,
				),
				#[codec(index = 1)]
				Precommit(
					runtime_types::finality_grandpa::Equivocation<
						runtime_types::sp_consensus_grandpa::app::Public,
						runtime_types::finality_grandpa::Precommit<_0, _1>,
						runtime_types::sp_consensus_grandpa::app::Signature,
					>,
				),
			}
		}
		pub mod sp_consensus_slots {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct EquivocationProof<_0, _1> {
				pub offender: _1,
				pub slot: runtime_types::sp_consensus_slots::Slot,
				pub first_header: _0,
				pub second_header: _0,
			}
			#[derive(
				::codec::Decode,
				::codec::Encode,
				::subxt::ext::subxt_core::ext::codec::CompactAs,
				Clone,
				Debug,
				PartialEq,
			)]
			pub struct Slot(pub ::core::primitive::u64);
		}
		pub mod sp_core {
			use super::runtime_types;
			pub mod crypto {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct KeyTypeId(pub [::core::primitive::u8; 4usize]);
			}
			pub mod sr25519 {
				use super::runtime_types;
				pub mod vrf {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct VrfSignature {
						pub pre_output: [::core::primitive::u8; 32usize],
						pub proof: [::core::primitive::u8; 64usize],
					}
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct OpaqueMetadata(
				pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			);
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Void {}
		}
		pub mod sp_inherents {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct CheckInherentsResult {
				pub okay: ::core::primitive::bool,
				pub fatal_error: ::core::primitive::bool,
				pub errors: runtime_types::sp_inherents::InherentData,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct InherentData {
				pub data: ::subxt::ext::subxt_core::utils::KeyedVec<
					[::core::primitive::u8; 8usize],
					::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				>,
			}
		}
		pub mod sp_mmr_primitives {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct AncestryProof<_0> {
				pub prev_peaks: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
				pub prev_leaf_count: ::core::primitive::u64,
				pub leaf_count: ::core::primitive::u64,
				pub items: ::subxt::ext::subxt_core::alloc::vec::Vec<(::core::primitive::u64, _0)>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct EncodableOpaqueLeaf(
				pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			);
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum Error {
				#[codec(index = 0)]
				InvalidNumericOp,
				#[codec(index = 1)]
				Push,
				#[codec(index = 2)]
				GetRoot,
				#[codec(index = 3)]
				Commit,
				#[codec(index = 4)]
				GenerateProof,
				#[codec(index = 5)]
				Verify,
				#[codec(index = 6)]
				LeafNotFound,
				#[codec(index = 7)]
				PalletNotIncluded,
				#[codec(index = 8)]
				InvalidLeafIndex,
				#[codec(index = 9)]
				InvalidBestKnownBlock,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct LeafProof<_0> {
				pub leaf_indices: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u64>,
				pub leaf_count: ::core::primitive::u64,
				pub items: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
			}
		}
		pub mod sp_npos_elections {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ElectionScore {
				pub minimal_stake: ::core::primitive::u128,
				pub sum_stake: ::core::primitive::u128,
				pub sum_stake_squared: ::core::primitive::u128,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Support<_0> {
				pub total: ::core::primitive::u128,
				pub voters:
					::subxt::ext::subxt_core::alloc::vec::Vec<(_0, ::core::primitive::u128)>,
			}
		}
		pub mod sp_runtime {
			use super::runtime_types;
			pub mod generic {
				use super::runtime_types;
				pub mod block {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Block<_0, _1> {
						pub header: _0,
						pub extrinsics: ::subxt::ext::subxt_core::alloc::vec::Vec<_1>,
					}
				}
				pub mod digest {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum DigestItem {
						#[codec(index = 6)]
						PreRuntime(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 4)]
						Consensus(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 5)]
						Seal(
							[::core::primitive::u8; 4usize],
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						),
						#[codec(index = 0)]
						Other(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
						#[codec(index = 8)]
						RuntimeEnvironmentUpdated,
					}
				}
			}
			pub mod transaction_validity {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum InvalidTransaction {
					#[codec(index = 0)]
					Call,
					#[codec(index = 1)]
					Payment,
					#[codec(index = 2)]
					Future,
					#[codec(index = 3)]
					Stale,
					#[codec(index = 4)]
					BadProof,
					#[codec(index = 5)]
					AncientBirthBlock,
					#[codec(index = 6)]
					ExhaustsResources,
					#[codec(index = 7)]
					Custom(::core::primitive::u8),
					#[codec(index = 8)]
					BadMandatory,
					#[codec(index = 9)]
					MandatoryValidation,
					#[codec(index = 10)]
					BadSigner,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum TransactionSource {
					#[codec(index = 0)]
					InBlock,
					#[codec(index = 1)]
					Local,
					#[codec(index = 2)]
					External,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum TransactionValidityError {
					#[codec(index = 0)]
					Invalid(runtime_types::sp_runtime::transaction_validity::InvalidTransaction),
					#[codec(index = 1)]
					Unknown(runtime_types::sp_runtime::transaction_validity::UnknownTransaction),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum UnknownTransaction {
					#[codec(index = 0)]
					CannotLookup,
					#[codec(index = 1)]
					NoUnsignedValidator,
					#[codec(index = 2)]
					Custom(::core::primitive::u8),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct ValidTransaction {
					pub priority: ::core::primitive::u64,
					pub requires: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>,
					pub provides: ::subxt::ext::subxt_core::alloc::vec::Vec<
						::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
					>,
					pub longevity: ::core::primitive::u64,
					pub propagate: ::core::primitive::bool,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum DispatchError {
				#[codec(index = 0)]
				Other,
				#[codec(index = 1)]
				CannotLookup,
				#[codec(index = 2)]
				BadOrigin,
				#[codec(index = 3)]
				Module(runtime_types::sp_runtime::ModuleError),
				#[codec(index = 4)]
				ConsumerRemaining,
				#[codec(index = 5)]
				NoProviders,
				#[codec(index = 6)]
				TooManyConsumers,
				#[codec(index = 7)]
				Token(runtime_types::sp_runtime::TokenError),
				#[codec(index = 8)]
				Arithmetic(runtime_types::sp_arithmetic::ArithmeticError),
				#[codec(index = 9)]
				Transactional(runtime_types::sp_runtime::TransactionalError),
				#[codec(index = 10)]
				Exhausted,
				#[codec(index = 11)]
				Corruption,
				#[codec(index = 12)]
				Unavailable,
				#[codec(index = 13)]
				RootNotAllowed,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct DispatchErrorWithPostInfo<_0> {
				pub post_info: _0,
				pub error: runtime_types::sp_runtime::DispatchError,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ExtrinsicInclusionMode {
				#[codec(index = 0)]
				AllExtrinsics,
				#[codec(index = 1)]
				OnlyInherents,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ModuleError {
				pub index: ::core::primitive::u8,
				pub error: [::core::primitive::u8; 4usize],
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum MultiSignature {
				#[codec(index = 0)]
				Ed25519([::core::primitive::u8; 64usize]),
				#[codec(index = 1)]
				Sr25519([::core::primitive::u8; 64usize]),
				#[codec(index = 2)]
				Ecdsa([::core::primitive::u8; 65usize]),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum MultiSigner {
				#[codec(index = 0)]
				Ed25519([::core::primitive::u8; 32usize]),
				#[codec(index = 1)]
				Sr25519([::core::primitive::u8; 32usize]),
				#[codec(index = 2)]
				Ecdsa([::core::primitive::u8; 33usize]),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct OpaqueValue(
				pub ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
			);
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum TokenError {
				#[codec(index = 0)]
				FundsUnavailable,
				#[codec(index = 1)]
				OnlyProvider,
				#[codec(index = 2)]
				BelowMinimum,
				#[codec(index = 3)]
				CannotCreate,
				#[codec(index = 4)]
				UnknownAsset,
				#[codec(index = 5)]
				Frozen,
				#[codec(index = 6)]
				Unsupported,
				#[codec(index = 7)]
				CannotCreateHold,
				#[codec(index = 8)]
				NotExpendable,
				#[codec(index = 9)]
				Blocked,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum TransactionalError {
				#[codec(index = 0)]
				LimitReached,
				#[codec(index = 1)]
				NoLayer,
			}
		}
		pub mod sp_staking {
			use super::runtime_types;
			pub mod offence {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct OffenceDetails<_0, _1> {
					pub offender: _1,
					pub reporters: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Exposure<_0, _1> {
				#[codec(compact)]
				pub total: _1,
				#[codec(compact)]
				pub own: _1,
				pub others: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::sp_staking::IndividualExposure<_0, _1>,
				>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct ExposurePage<_0, _1> {
				#[codec(compact)]
				pub page_total: _1,
				pub others: ::subxt::ext::subxt_core::alloc::vec::Vec<
					runtime_types::sp_staking::IndividualExposure<_0, _1>,
				>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct IndividualExposure<_0, _1> {
				pub who: _0,
				#[codec(compact)]
				pub value: _1,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct PagedExposureMetadata<_0> {
				#[codec(compact)]
				pub total: _0,
				#[codec(compact)]
				pub own: _0,
				pub nominator_count: ::core::primitive::u32,
				pub page_count: ::core::primitive::u32,
			}
		}
		pub mod sp_version {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RuntimeVersion {
				pub spec_name: ::subxt::ext::subxt_core::alloc::string::String,
				pub impl_name: ::subxt::ext::subxt_core::alloc::string::String,
				pub authoring_version: ::core::primitive::u32,
				pub spec_version: ::core::primitive::u32,
				pub impl_version: ::core::primitive::u32,
				pub apis: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					[::core::primitive::u8; 8usize],
					::core::primitive::u32,
				)>,
				pub transaction_version: ::core::primitive::u32,
				pub state_version: ::core::primitive::u8,
			}
		}
		pub mod sp_weights {
			use super::runtime_types;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct RuntimeDbWeight {
				pub read: ::core::primitive::u64,
				pub write: ::core::primitive::u64,
			}
		}
		pub mod staging_xcm {
			use super::runtime_types;
			pub mod v3 {
				use super::runtime_types;
				pub mod multilocation {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiLocation {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::xcm::v3::junctions::Junctions,
					}
				}
			}
			pub mod v4 {
				use super::runtime_types;
				pub mod asset {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Asset {
						pub id: runtime_types::staging_xcm::v4::asset::AssetId,
						pub fun: runtime_types::staging_xcm::v4::asset::Fungibility,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::staging_xcm::v4::asset::Assets),
						#[codec(index = 1)]
						Wild(runtime_types::staging_xcm::v4::asset::WildAsset),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct AssetId(pub runtime_types::staging_xcm::v4::location::Location);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Assets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::staging_xcm::v4::asset::Asset,
						>,
					);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::staging_xcm::v4::asset::AssetInstance),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::staging_xcm::v4::asset::AssetId,
							fun: runtime_types::staging_xcm::v4::asset::WildFungibility,
						},
						#[codec(index = 2)]
						AllCounted(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						AllOfCounted {
							id: runtime_types::staging_xcm::v4::asset::AssetId,
							fun: runtime_types::staging_xcm::v4::asset::WildFungibility,
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
				}
				pub mod junction {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network: ::core::option::Option<
								runtime_types::staging_xcm::v4::junction::NetworkId,
							>,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey {
							length: ::core::primitive::u8,
							data: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v3::junction::BodyId,
							part: runtime_types::xcm::v3::junction::BodyPart,
						},
						#[codec(index = 9)]
						GlobalConsensus(runtime_types::staging_xcm::v4::junction::NetworkId),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum NetworkId {
						#[codec(index = 0)]
						ByGenesis([::core::primitive::u8; 32usize]),
						#[codec(index = 1)]
						ByFork {
							block_number: ::core::primitive::u64,
							block_hash: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						Polkadot,
						#[codec(index = 3)]
						Kusama,
						#[codec(index = 4)]
						Westend,
						#[codec(index = 5)]
						Rococo,
						#[codec(index = 6)]
						Wococo,
						#[codec(index = 7)]
						Ethereum {
							#[codec(compact)]
							chain_id: ::core::primitive::u64,
						},
						#[codec(index = 8)]
						BitcoinCore,
						#[codec(index = 9)]
						BitcoinCash,
						#[codec(index = 10)]
						PolkadotBulletin,
					}
				}
				pub mod junctions {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1([runtime_types::staging_xcm::v4::junction::Junction; 1usize]),
						#[codec(index = 2)]
						X2([runtime_types::staging_xcm::v4::junction::Junction; 2usize]),
						#[codec(index = 3)]
						X3([runtime_types::staging_xcm::v4::junction::Junction; 3usize]),
						#[codec(index = 4)]
						X4([runtime_types::staging_xcm::v4::junction::Junction; 4usize]),
						#[codec(index = 5)]
						X5([runtime_types::staging_xcm::v4::junction::Junction; 5usize]),
						#[codec(index = 6)]
						X6([runtime_types::staging_xcm::v4::junction::Junction; 6usize]),
						#[codec(index = 7)]
						X7([runtime_types::staging_xcm::v4::junction::Junction; 7usize]),
						#[codec(index = 8)]
						X8([runtime_types::staging_xcm::v4::junction::Junction; 8usize]),
					}
				}
				pub mod location {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Location {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::staging_xcm::v4::junctions::Junctions,
					}
				}
				pub mod traits {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Outcome {
						#[codec(index = 0)]
						Complete { used: ::sp_weights::Weight },
						#[codec(index = 1)]
						Incomplete {
							used: ::sp_weights::Weight,
							error: runtime_types::xcm::v3::traits::Error,
						},
						#[codec(index = 2)]
						Error { error: runtime_types::xcm::v3::traits::Error },
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::staging_xcm::v4::Response,
						max_weight: ::sp_weights::Weight,
						querier: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						beneficiary: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_kind: runtime_types::xcm::v3::OriginKind,
						require_weight_at_most: ::sp_weights::Weight,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::staging_xcm::v4::junctions::Junctions),
					#[codec(index = 12)]
					ReportError(runtime_types::staging_xcm::v4::QueryResponseInfo),
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						beneficiary: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::staging_xcm::v4::asset::AssetFilter,
						want: runtime_types::staging_xcm::v4::asset::Assets,
						maximal: ::core::primitive::bool,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						reserve: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
						dest: runtime_types::staging_xcm::v4::location::Location,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 18)]
					ReportHolding {
						response_info: runtime_types::staging_xcm::v4::QueryResponseInfo,
						assets: runtime_types::staging_xcm::v4::asset::AssetFilter,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::staging_xcm::v4::asset::Asset,
						weight_limit: runtime_types::xcm::v3::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::staging_xcm::v4::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::staging_xcm::v4::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::staging_xcm::v4::asset::Assets,
						ticket: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						max_response_weight: ::sp_weights::Weight,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
					#[codec(index = 28)]
					BurnAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 29)]
					ExpectAsset(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 30)]
					ExpectOrigin(
						::core::option::Option<runtime_types::staging_xcm::v4::location::Location>,
					),
					#[codec(index = 31)]
					ExpectError(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 32)]
					ExpectTransactStatus(runtime_types::xcm::v3::MaybeErrorCode),
					#[codec(index = 33)]
					QueryPallet {
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						response_info: runtime_types::staging_xcm::v4::QueryResponseInfo,
					},
					#[codec(index = 34)]
					ExpectPallet {
						#[codec(compact)]
						index: ::core::primitive::u32,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						#[codec(compact)]
						crate_major: ::core::primitive::u32,
						#[codec(compact)]
						min_crate_minor: ::core::primitive::u32,
					},
					#[codec(index = 35)]
					ReportTransactStatus(runtime_types::staging_xcm::v4::QueryResponseInfo),
					#[codec(index = 36)]
					ClearTransactStatus,
					#[codec(index = 37)]
					UniversalOrigin(runtime_types::staging_xcm::v4::junction::Junction),
					#[codec(index = 38)]
					ExportMessage {
						network: runtime_types::staging_xcm::v4::junction::NetworkId,
						destination: runtime_types::staging_xcm::v4::junctions::Junctions,
						xcm: runtime_types::staging_xcm::v4::Xcm,
					},
					#[codec(index = 39)]
					LockAsset {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						unlocker: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 40)]
					UnlockAsset {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						target: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 41)]
					NoteUnlockable {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						owner: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 42)]
					RequestUnlock {
						asset: runtime_types::staging_xcm::v4::asset::Asset,
						locker: runtime_types::staging_xcm::v4::location::Location,
					},
					#[codec(index = 43)]
					SetFeesMode { jit_withdraw: ::core::primitive::bool },
					#[codec(index = 44)]
					SetTopic([::core::primitive::u8; 32usize]),
					#[codec(index = 45)]
					ClearTopic,
					#[codec(index = 46)]
					AliasOrigin(runtime_types::staging_xcm::v4::location::Location),
					#[codec(index = 47)]
					UnpaidExecution {
						weight_limit: runtime_types::xcm::v3::WeightLimit,
						check_origin: ::core::option::Option<
							runtime_types::staging_xcm::v4::location::Location,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PalletInfo {
					#[codec(compact)]
					pub index: ::core::primitive::u32,
					pub name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					pub module_name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					#[codec(compact)]
					pub major: ::core::primitive::u32,
					#[codec(compact)]
					pub minor: ::core::primitive::u32,
					#[codec(compact)]
					pub patch: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct QueryResponseInfo {
					pub destination: runtime_types::staging_xcm::v4::location::Location,
					#[codec(compact)]
					pub query_id: ::core::primitive::u64,
					pub max_weight: ::sp_weights::Weight,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::staging_xcm::v4::asset::Assets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
					#[codec(index = 4)]
					PalletsInfo(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::staging_xcm::v4::PalletInfo,
						>,
					),
					#[codec(index = 5)]
					DispatchResult(runtime_types::xcm::v3::MaybeErrorCode),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::staging_xcm::v4::Instruction,
					>,
				);
			}
		}
		pub mod staging_xcm_executor {
			use super::runtime_types;
			pub mod traits {
				use super::runtime_types;
				pub mod asset_transfer {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum TransferType {
						#[codec(index = 0)]
						Teleport,
						#[codec(index = 1)]
						LocalReserve,
						#[codec(index = 2)]
						DestinationReserve,
						#[codec(index = 3)]
						RemoteReserve(runtime_types::xcm::VersionedLocation),
					}
				}
			}
		}
		pub mod westend_runtime {
			use super::runtime_types;
			pub mod dynamic_params {
				use super::runtime_types;
				pub mod inflation {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct Falloff;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct IdealStake;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MaxInflation;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MinInflation;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Parameters {
						#[codec(index = 0)]
                        MinInflation(
                            runtime_types::westend_runtime::dynamic_params::inflation::MinInflation,
                            ::core::option::Option<
                                runtime_types::sp_arithmetic::per_things::Perquintill,
                            >,
                        ),
                        #[codec(index = 1)]
                        MaxInflation(
                            runtime_types::westend_runtime::dynamic_params::inflation::MaxInflation,
                            ::core::option::Option<
                                runtime_types::sp_arithmetic::per_things::Perquintill,
                            >,
                        ),
                        #[codec(index = 2)]
                        IdealStake(
                            runtime_types::westend_runtime::dynamic_params::inflation::IdealStake,
                            ::core::option::Option<
                                runtime_types::sp_arithmetic::per_things::Perquintill,
                            >,
                        ),
                        #[codec(index = 3)]
                        Falloff(
                            runtime_types::westend_runtime::dynamic_params::inflation::Falloff,
                            ::core::option::Option<
                                runtime_types::sp_arithmetic::per_things::Perquintill,
                            >,
                        ),
                        #[codec(index = 4)]
                        UseAuctionSlots(
                            runtime_types::westend_runtime::dynamic_params::inflation::UseAuctionSlots,
                            ::core::option::Option<::core::primitive::bool>,
                        ),
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum ParametersKey {
						#[codec(index = 0)]
                        MinInflation(
                            runtime_types::westend_runtime::dynamic_params::inflation::MinInflation,
                        ),
                        #[codec(index = 1)]
                        MaxInflation(
                            runtime_types::westend_runtime::dynamic_params::inflation::MaxInflation,
                        ),
                        #[codec(index = 2)]
                        IdealStake(
                            runtime_types::westend_runtime::dynamic_params::inflation::IdealStake,
                        ),
                        #[codec(index = 3)]
                        Falloff(
                            runtime_types::westend_runtime::dynamic_params::inflation::Falloff,
                        ),
                        #[codec(index = 4)]
                        UseAuctionSlots(
                            runtime_types::westend_runtime::dynamic_params::inflation::UseAuctionSlots,
                        ),
                    }
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum ParametersValue {
						#[codec(index = 0)]
						MinInflation(runtime_types::sp_arithmetic::per_things::Perquintill),
						#[codec(index = 1)]
						MaxInflation(runtime_types::sp_arithmetic::per_things::Perquintill),
						#[codec(index = 2)]
						IdealStake(runtime_types::sp_arithmetic::per_things::Perquintill),
						#[codec(index = 3)]
						Falloff(runtime_types::sp_arithmetic::per_things::Perquintill),
						#[codec(index = 4)]
						UseAuctionSlots(::core::primitive::bool),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct UseAuctionSlots;
				}
			}
			pub mod governance {
				use super::runtime_types;
				pub mod origins {
					use super::runtime_types;
					pub mod pallet_custom_origins {
						use super::runtime_types;
						#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
						pub enum Origin {
							#[codec(index = 0)]
							StakingAdmin,
							#[codec(index = 1)]
							Treasurer,
							#[codec(index = 2)]
							FellowshipAdmin,
							#[codec(index = 3)]
							GeneralAdmin,
							#[codec(index = 4)]
							AuctionAdmin,
							#[codec(index = 5)]
							LeaseAdmin,
							#[codec(index = 6)]
							ReferendumCanceller,
							#[codec(index = 7)]
							ReferendumKiller,
							#[codec(index = 8)]
							SmallTipper,
							#[codec(index = 9)]
							BigTipper,
							#[codec(index = 10)]
							SmallSpender,
							#[codec(index = 11)]
							MediumSpender,
							#[codec(index = 12)]
							BigSpender,
							#[codec(index = 13)]
							WhitelistedCaller,
							#[codec(index = 14)]
							FellowshipInitiates,
							#[codec(index = 15)]
							Fellows,
							#[codec(index = 16)]
							FellowshipExperts,
							#[codec(index = 17)]
							FellowshipMasters,
							#[codec(index = 18)]
							Fellowship1Dan,
							#[codec(index = 19)]
							Fellowship2Dan,
							#[codec(index = 20)]
							Fellowship3Dan,
							#[codec(index = 21)]
							Fellowship4Dan,
							#[codec(index = 22)]
							Fellowship5Dan,
							#[codec(index = 23)]
							Fellowship6Dan,
							#[codec(index = 24)]
							Fellowship7Dan,
							#[codec(index = 25)]
							Fellowship8Dan,
							#[codec(index = 26)]
							Fellowship9Dan,
						}
					}
				}
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct NposCompactSolution16 {
				pub votes1: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes2: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					),
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes3: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 2usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes4: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 3usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes5: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 4usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes6: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 5usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes7: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 6usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes8: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 7usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes9: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 8usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes10: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 9usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes11: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 10usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes12: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 11usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes13: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 12usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes14: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 13usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes15: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 14usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
				pub votes16: ::subxt::ext::subxt_core::alloc::vec::Vec<(
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u32>,
					[(
						::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
						::subxt::ext::subxt_core::ext::codec::Compact<
							runtime_types::sp_arithmetic::per_things::PerU16,
						>,
					); 15usize],
					::subxt::ext::subxt_core::ext::codec::Compact<::core::primitive::u16>,
				)>,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum OriginCaller {
				#[codec(index = 0)]
                system(
                    runtime_types::frame_support::dispatch::RawOrigin<
                        ::sp_core::crypto::AccountId32,
                    >,
                ),
                #[codec(index = 35)]
                Origins(
                    runtime_types::westend_runtime::governance::origins::pallet_custom_origins::Origin,
                ),
                #[codec(index = 41)]
                ParachainsOrigin(
                    runtime_types::polkadot_runtime_parachains::origin::pallet::Origin,
                ),
                #[codec(index = 99)]
                XcmPallet(runtime_types::pallet_xcm::pallet::Origin),
                #[codec(index = 4)]
                Void(runtime_types::sp_core::Void),
            }
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum ProxyType {
				#[codec(index = 0)]
				Any,
				#[codec(index = 1)]
				NonTransfer,
				#[codec(index = 2)]
				Governance,
				#[codec(index = 3)]
				Staking,
				#[codec(index = 4)]
				SudoBalances,
				#[codec(index = 5)]
				IdentityJudgement,
				#[codec(index = 6)]
				CancelProxy,
				#[codec(index = 7)]
				Auction,
				#[codec(index = 8)]
				NominationPools,
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct Runtime;
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeCall {
				#[codec(index = 0)]
				System(runtime_types::frame_system::pallet::Call),
				#[codec(index = 1)]
				Babe(runtime_types::pallet_babe::pallet::Call),
				#[codec(index = 2)]
				Timestamp(runtime_types::pallet_timestamp::pallet::Call),
				#[codec(index = 3)]
				Indices(runtime_types::pallet_indices::pallet::Call),
				#[codec(index = 4)]
				Balances(runtime_types::pallet_balances::pallet::Call),
				#[codec(index = 6)]
				Staking(runtime_types::pallet_staking::pallet::pallet::Call),
				#[codec(index = 70)]
				Parameters(runtime_types::pallet_parameters::pallet::Call),
				#[codec(index = 8)]
				Session(runtime_types::pallet_session::pallet::Call),
				#[codec(index = 10)]
				Grandpa(runtime_types::pallet_grandpa::pallet::Call),
				#[codec(index = 16)]
				Utility(runtime_types::pallet_utility::pallet::Call),
				#[codec(index = 17)]
				Identity(runtime_types::pallet_identity::pallet::Call),
				#[codec(index = 18)]
				Recovery(runtime_types::pallet_recovery::pallet::Call),
				#[codec(index = 19)]
				Vesting(runtime_types::pallet_vesting::pallet::Call),
				#[codec(index = 20)]
				Scheduler(runtime_types::pallet_scheduler::pallet::Call),
				#[codec(index = 28)]
				Preimage(runtime_types::pallet_preimage::pallet::Call),
				#[codec(index = 21)]
				Sudo(runtime_types::pallet_sudo::pallet::Call),
				#[codec(index = 22)]
				Proxy(runtime_types::pallet_proxy::pallet::Call),
				#[codec(index = 23)]
				Multisig(runtime_types::pallet_multisig::pallet::Call),
				#[codec(index = 24)]
				ElectionProviderMultiPhase(
					runtime_types::pallet_election_provider_multi_phase::pallet::Call,
				),
				#[codec(index = 25)]
				VoterList(runtime_types::pallet_bags_list::pallet::Call),
				#[codec(index = 29)]
				NominationPools(runtime_types::pallet_nomination_pools::pallet::Call),
				#[codec(index = 30)]
				FastUnstake(runtime_types::pallet_fast_unstake::pallet::Call),
				#[codec(index = 31)]
				ConvictionVoting(runtime_types::pallet_conviction_voting::pallet::Call),
				#[codec(index = 32)]
				Referenda(runtime_types::pallet_referenda::pallet::Call),
				#[codec(index = 36)]
				Whitelist(runtime_types::pallet_whitelist::pallet::Call),
				#[codec(index = 37)]
				Treasury(runtime_types::pallet_treasury::pallet::Call),
				#[codec(index = 42)]
				Configuration(
					runtime_types::polkadot_runtime_parachains::configuration::pallet::Call,
				),
				#[codec(index = 43)]
				ParasShared(runtime_types::polkadot_runtime_parachains::shared::pallet::Call),
				#[codec(index = 44)]
				ParaInclusion(runtime_types::polkadot_runtime_parachains::inclusion::pallet::Call),
				#[codec(index = 45)]
				ParaInherent(
					runtime_types::polkadot_runtime_parachains::paras_inherent::pallet::Call,
				),
				#[codec(index = 47)]
				Paras(runtime_types::polkadot_runtime_parachains::paras::pallet::Call),
				#[codec(index = 48)]
				Initializer(runtime_types::polkadot_runtime_parachains::initializer::pallet::Call),
				#[codec(index = 51)]
				Hrmp(runtime_types::polkadot_runtime_parachains::hrmp::pallet::Call),
				#[codec(index = 53)]
				ParasDisputes(runtime_types::polkadot_runtime_parachains::disputes::pallet::Call),
				#[codec(index = 54)]
				ParasSlashing(
					runtime_types::polkadot_runtime_parachains::disputes::slashing::pallet::Call,
				),
				#[codec(index = 56)]
				OnDemandAssignmentProvider(
					runtime_types::polkadot_runtime_parachains::on_demand::pallet::Call,
				),
				#[codec(index = 60)]
				Registrar(runtime_types::polkadot_runtime_common::paras_registrar::pallet::Call),
				#[codec(index = 61)]
				Slots(runtime_types::polkadot_runtime_common::slots::pallet::Call),
				#[codec(index = 62)]
				ParasSudoWrapper(
					runtime_types::polkadot_runtime_common::paras_sudo_wrapper::pallet::Call,
				),
				#[codec(index = 63)]
				Auctions(runtime_types::polkadot_runtime_common::auctions::pallet::Call),
				#[codec(index = 64)]
				Crowdloan(runtime_types::polkadot_runtime_common::crowdloan::pallet::Call),
				#[codec(index = 65)]
				AssignedSlots(runtime_types::polkadot_runtime_common::assigned_slots::pallet::Call),
				#[codec(index = 66)]
				Coretime(runtime_types::polkadot_runtime_parachains::coretime::pallet::Call),
				#[codec(index = 99)]
				XcmPallet(runtime_types::pallet_xcm::pallet::Call),
				#[codec(index = 100)]
				MessageQueue(runtime_types::pallet_message_queue::pallet::Call),
				#[codec(index = 101)]
				AssetRate(runtime_types::pallet_asset_rate::pallet::Call),
				#[codec(index = 102)]
				RootTesting(runtime_types::pallet_root_testing::pallet::Call),
				#[codec(index = 200)]
				Beefy(runtime_types::pallet_beefy::pallet::Call),
				#[codec(index = 248)]
				IdentityMigrator(
					runtime_types::polkadot_runtime_common::identity_migrator::pallet::Call,
				),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeError {
				#[codec(index = 0)]
				System(runtime_types::frame_system::pallet::Error),
				#[codec(index = 1)]
				Babe(runtime_types::pallet_babe::pallet::Error),
				#[codec(index = 3)]
				Indices(runtime_types::pallet_indices::pallet::Error),
				#[codec(index = 4)]
				Balances(runtime_types::pallet_balances::pallet::Error),
				#[codec(index = 6)]
				Staking(runtime_types::pallet_staking::pallet::pallet::Error),
				#[codec(index = 8)]
				Session(runtime_types::pallet_session::pallet::Error),
				#[codec(index = 10)]
				Grandpa(runtime_types::pallet_grandpa::pallet::Error),
				#[codec(index = 16)]
				Utility(runtime_types::pallet_utility::pallet::Error),
				#[codec(index = 17)]
				Identity(runtime_types::pallet_identity::pallet::Error),
				#[codec(index = 18)]
				Recovery(runtime_types::pallet_recovery::pallet::Error),
				#[codec(index = 19)]
				Vesting(runtime_types::pallet_vesting::pallet::Error),
				#[codec(index = 20)]
				Scheduler(runtime_types::pallet_scheduler::pallet::Error),
				#[codec(index = 28)]
				Preimage(runtime_types::pallet_preimage::pallet::Error),
				#[codec(index = 21)]
				Sudo(runtime_types::pallet_sudo::pallet::Error),
				#[codec(index = 22)]
				Proxy(runtime_types::pallet_proxy::pallet::Error),
				#[codec(index = 23)]
				Multisig(runtime_types::pallet_multisig::pallet::Error),
				#[codec(index = 24)]
				ElectionProviderMultiPhase(
					runtime_types::pallet_election_provider_multi_phase::pallet::Error,
				),
				#[codec(index = 25)]
				VoterList(runtime_types::pallet_bags_list::pallet::Error),
				#[codec(index = 29)]
				NominationPools(runtime_types::pallet_nomination_pools::pallet::Error),
				#[codec(index = 30)]
				FastUnstake(runtime_types::pallet_fast_unstake::pallet::Error),
				#[codec(index = 31)]
				ConvictionVoting(runtime_types::pallet_conviction_voting::pallet::Error),
				#[codec(index = 32)]
				Referenda(runtime_types::pallet_referenda::pallet::Error),
				#[codec(index = 36)]
				Whitelist(runtime_types::pallet_whitelist::pallet::Error),
				#[codec(index = 37)]
				Treasury(runtime_types::pallet_treasury::pallet::Error),
				#[codec(index = 38)]
				DelegatedStaking(runtime_types::pallet_delegated_staking::pallet::Error),
				#[codec(index = 42)]
				Configuration(
					runtime_types::polkadot_runtime_parachains::configuration::pallet::Error,
				),
				#[codec(index = 44)]
				ParaInclusion(runtime_types::polkadot_runtime_parachains::inclusion::pallet::Error),
				#[codec(index = 45)]
				ParaInherent(
					runtime_types::polkadot_runtime_parachains::paras_inherent::pallet::Error,
				),
				#[codec(index = 47)]
				Paras(runtime_types::polkadot_runtime_parachains::paras::pallet::Error),
				#[codec(index = 51)]
				Hrmp(runtime_types::polkadot_runtime_parachains::hrmp::pallet::Error),
				#[codec(index = 53)]
				ParasDisputes(runtime_types::polkadot_runtime_parachains::disputes::pallet::Error),
				#[codec(index = 54)]
				ParasSlashing(
					runtime_types::polkadot_runtime_parachains::disputes::slashing::pallet::Error,
				),
				#[codec(index = 56)]
				OnDemandAssignmentProvider(
					runtime_types::polkadot_runtime_parachains::on_demand::pallet::Error,
				),
				#[codec(index = 57)]
				CoretimeAssignmentProvider(
					runtime_types::polkadot_runtime_parachains::assigner_coretime::pallet::Error,
				),
				#[codec(index = 60)]
				Registrar(runtime_types::polkadot_runtime_common::paras_registrar::pallet::Error),
				#[codec(index = 61)]
				Slots(runtime_types::polkadot_runtime_common::slots::pallet::Error),
				#[codec(index = 62)]
				ParasSudoWrapper(
					runtime_types::polkadot_runtime_common::paras_sudo_wrapper::pallet::Error,
				),
				#[codec(index = 63)]
				Auctions(runtime_types::polkadot_runtime_common::auctions::pallet::Error),
				#[codec(index = 64)]
				Crowdloan(runtime_types::polkadot_runtime_common::crowdloan::pallet::Error),
				#[codec(index = 65)]
				AssignedSlots(
					runtime_types::polkadot_runtime_common::assigned_slots::pallet::Error,
				),
				#[codec(index = 66)]
				Coretime(runtime_types::polkadot_runtime_parachains::coretime::pallet::Error),
				#[codec(index = 99)]
				XcmPallet(runtime_types::pallet_xcm::pallet::Error),
				#[codec(index = 100)]
				MessageQueue(runtime_types::pallet_message_queue::pallet::Error),
				#[codec(index = 101)]
				AssetRate(runtime_types::pallet_asset_rate::pallet::Error),
				#[codec(index = 200)]
				Beefy(runtime_types::pallet_beefy::pallet::Error),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeEvent {
				#[codec(index = 0)]
				System(runtime_types::frame_system::pallet::Event),
				#[codec(index = 3)]
				Indices(runtime_types::pallet_indices::pallet::Event),
				#[codec(index = 4)]
				Balances(runtime_types::pallet_balances::pallet::Event),
				#[codec(index = 26)]
				TransactionPayment(runtime_types::pallet_transaction_payment::pallet::Event),
				#[codec(index = 6)]
				Staking(runtime_types::pallet_staking::pallet::pallet::Event),
				#[codec(index = 7)]
				Offences(runtime_types::pallet_offences::pallet::Event),
				#[codec(index = 70)]
				Parameters(runtime_types::pallet_parameters::pallet::Event),
				#[codec(index = 8)]
				Session(runtime_types::pallet_session::pallet::Event),
				#[codec(index = 10)]
				Grandpa(runtime_types::pallet_grandpa::pallet::Event),
				#[codec(index = 16)]
				Utility(runtime_types::pallet_utility::pallet::Event),
				#[codec(index = 17)]
				Identity(runtime_types::pallet_identity::pallet::Event),
				#[codec(index = 18)]
				Recovery(runtime_types::pallet_recovery::pallet::Event),
				#[codec(index = 19)]
				Vesting(runtime_types::pallet_vesting::pallet::Event),
				#[codec(index = 20)]
				Scheduler(runtime_types::pallet_scheduler::pallet::Event),
				#[codec(index = 28)]
				Preimage(runtime_types::pallet_preimage::pallet::Event),
				#[codec(index = 21)]
				Sudo(runtime_types::pallet_sudo::pallet::Event),
				#[codec(index = 22)]
				Proxy(runtime_types::pallet_proxy::pallet::Event),
				#[codec(index = 23)]
				Multisig(runtime_types::pallet_multisig::pallet::Event),
				#[codec(index = 24)]
				ElectionProviderMultiPhase(
					runtime_types::pallet_election_provider_multi_phase::pallet::Event,
				),
				#[codec(index = 25)]
				VoterList(runtime_types::pallet_bags_list::pallet::Event),
				#[codec(index = 29)]
				NominationPools(runtime_types::pallet_nomination_pools::pallet::Event),
				#[codec(index = 30)]
				FastUnstake(runtime_types::pallet_fast_unstake::pallet::Event),
				#[codec(index = 31)]
				ConvictionVoting(runtime_types::pallet_conviction_voting::pallet::Event),
				#[codec(index = 32)]
				Referenda(runtime_types::pallet_referenda::pallet::Event),
				#[codec(index = 36)]
				Whitelist(runtime_types::pallet_whitelist::pallet::Event),
				#[codec(index = 37)]
				Treasury(runtime_types::pallet_treasury::pallet::Event),
				#[codec(index = 38)]
				DelegatedStaking(runtime_types::pallet_delegated_staking::pallet::Event),
				#[codec(index = 44)]
				ParaInclusion(runtime_types::polkadot_runtime_parachains::inclusion::pallet::Event),
				#[codec(index = 47)]
				Paras(runtime_types::polkadot_runtime_parachains::paras::pallet::Event),
				#[codec(index = 51)]
				Hrmp(runtime_types::polkadot_runtime_parachains::hrmp::pallet::Event),
				#[codec(index = 53)]
				ParasDisputes(runtime_types::polkadot_runtime_parachains::disputes::pallet::Event),
				#[codec(index = 56)]
				OnDemandAssignmentProvider(
					runtime_types::polkadot_runtime_parachains::on_demand::pallet::Event,
				),
				#[codec(index = 60)]
				Registrar(runtime_types::polkadot_runtime_common::paras_registrar::pallet::Event),
				#[codec(index = 61)]
				Slots(runtime_types::polkadot_runtime_common::slots::pallet::Event),
				#[codec(index = 63)]
				Auctions(runtime_types::polkadot_runtime_common::auctions::pallet::Event),
				#[codec(index = 64)]
				Crowdloan(runtime_types::polkadot_runtime_common::crowdloan::pallet::Event),
				#[codec(index = 65)]
				AssignedSlots(
					runtime_types::polkadot_runtime_common::assigned_slots::pallet::Event,
				),
				#[codec(index = 66)]
				Coretime(runtime_types::polkadot_runtime_parachains::coretime::pallet::Event),
				#[codec(index = 99)]
				XcmPallet(runtime_types::pallet_xcm::pallet::Event),
				#[codec(index = 100)]
				MessageQueue(runtime_types::pallet_message_queue::pallet::Event),
				#[codec(index = 101)]
				AssetRate(runtime_types::pallet_asset_rate::pallet::Event),
				#[codec(index = 102)]
				RootTesting(runtime_types::pallet_root_testing::pallet::Event),
				#[codec(index = 248)]
				IdentityMigrator(
					runtime_types::polkadot_runtime_common::identity_migrator::pallet::Event,
				),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeFreezeReason {
				#[codec(index = 29)]
				NominationPools(runtime_types::pallet_nomination_pools::pallet::FreezeReason),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeHoldReason {
				#[codec(index = 28)]
				Preimage(runtime_types::pallet_preimage::pallet::HoldReason),
				#[codec(index = 38)]
				DelegatedStaking(runtime_types::pallet_delegated_staking::pallet::HoldReason),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeParameters {
				#[codec(index = 0)]
				Inflation(runtime_types::westend_runtime::dynamic_params::inflation::Parameters),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeParametersKey {
				#[codec(index = 0)]
				Inflation(runtime_types::westend_runtime::dynamic_params::inflation::ParametersKey),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum RuntimeParametersValue {
				#[codec(index = 0)]
				Inflation(
					runtime_types::westend_runtime::dynamic_params::inflation::ParametersValue,
				),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub struct SessionKeys {
				pub grandpa: runtime_types::sp_consensus_grandpa::app::Public,
				pub babe: runtime_types::sp_consensus_babe::app::Public,
				pub para_validator: runtime_types::polkadot_primitives::v8::validator_app::Public,
				pub para_assignment: runtime_types::polkadot_primitives::v8::assignment_app::Public,
				pub authority_discovery: runtime_types::sp_authority_discovery::app::Public,
				pub beefy: runtime_types::sp_consensus_beefy::ecdsa_crypto::Public,
			}
		}
		pub mod xcm {
			use super::runtime_types;
			pub mod double_encoded {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct DoubleEncoded {
					pub encoded: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
				}
			}
			pub mod v2 {
				use super::runtime_types;
				pub mod junction {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network: runtime_types::xcm::v2::NetworkId,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network: runtime_types::xcm::v2::NetworkId,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network: runtime_types::xcm::v2::NetworkId,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey(
							runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
								::core::primitive::u8,
							>,
						),
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v2::BodyId,
							part: runtime_types::xcm::v2::BodyPart,
						},
					}
				}
				pub mod multiasset {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetId {
						#[codec(index = 0)]
						Concrete(runtime_types::xcm::v2::multilocation::MultiLocation),
						#[codec(index = 1)]
						Abstract(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
						#[codec(index = 6)]
						Blob(::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::xcm::v2::multiasset::AssetInstance),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiAsset {
						pub id: runtime_types::xcm::v2::multiasset::AssetId,
						pub fun: runtime_types::xcm::v2::multiasset::Fungibility,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum MultiAssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::xcm::v2::multiasset::MultiAssets),
						#[codec(index = 1)]
						Wild(runtime_types::xcm::v2::multiasset::WildMultiAsset),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiAssets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::xcm::v2::multiasset::MultiAsset,
						>,
					);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildMultiAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::xcm::v2::multiasset::AssetId,
							fun: runtime_types::xcm::v2::multiasset::WildFungibility,
						},
					}
				}
				pub mod multilocation {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1(runtime_types::xcm::v2::junction::Junction),
						#[codec(index = 2)]
						X2(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 3)]
						X3(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 4)]
						X4(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 5)]
						X5(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 6)]
						X6(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 7)]
						X7(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
						#[codec(index = 8)]
						X8(
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
							runtime_types::xcm::v2::junction::Junction,
						),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiLocation {
						pub parents: ::core::primitive::u8,
						pub interior: runtime_types::xcm::v2::multilocation::Junctions,
					}
				}
				pub mod traits {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						Overflow,
						#[codec(index = 1)]
						Unimplemented,
						#[codec(index = 2)]
						UntrustedReserveLocation,
						#[codec(index = 3)]
						UntrustedTeleportLocation,
						#[codec(index = 4)]
						MultiLocationFull,
						#[codec(index = 5)]
						MultiLocationNotInvertible,
						#[codec(index = 6)]
						BadOrigin,
						#[codec(index = 7)]
						InvalidLocation,
						#[codec(index = 8)]
						AssetNotFound,
						#[codec(index = 9)]
						FailedToTransactAsset,
						#[codec(index = 10)]
						NotWithdrawable,
						#[codec(index = 11)]
						LocationCannotHold,
						#[codec(index = 12)]
						ExceedsMaxMessageSize,
						#[codec(index = 13)]
						DestinationUnsupported,
						#[codec(index = 14)]
						Transport,
						#[codec(index = 15)]
						Unroutable,
						#[codec(index = 16)]
						UnknownClaim,
						#[codec(index = 17)]
						FailedToDecode,
						#[codec(index = 18)]
						MaxWeightInvalid,
						#[codec(index = 19)]
						NotHoldingFees,
						#[codec(index = 20)]
						TooExpensive,
						#[codec(index = 21)]
						Trap(::core::primitive::u64),
						#[codec(index = 22)]
						UnhandledXcmVersion,
						#[codec(index = 23)]
						WeightLimitReached(::core::primitive::u64),
						#[codec(index = 24)]
						Barrier,
						#[codec(index = 25)]
						WeightNotComputable,
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum BodyId {
					#[codec(index = 0)]
					Unit,
					#[codec(index = 1)]
					Named(
						runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					Index(#[codec(compact)] ::core::primitive::u32),
					#[codec(index = 3)]
					Executive,
					#[codec(index = 4)]
					Technical,
					#[codec(index = 5)]
					Legislative,
					#[codec(index = 6)]
					Judicial,
					#[codec(index = 7)]
					Defense,
					#[codec(index = 8)]
					Administration,
					#[codec(index = 9)]
					Treasury,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum BodyPart {
					#[codec(index = 0)]
					Voice,
					#[codec(index = 1)]
					Members {
						#[codec(compact)]
						count: ::core::primitive::u32,
					},
					#[codec(index = 2)]
					Fraction {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
					#[codec(index = 3)]
					AtLeastProportion {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
					#[codec(index = 4)]
					MoreThanProportion {
						#[codec(compact)]
						nom: ::core::primitive::u32,
						#[codec(compact)]
						denom: ::core::primitive::u32,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::xcm::v2::Response,
						#[codec(compact)]
						max_weight: ::core::primitive::u64,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_type: runtime_types::xcm::v2::OriginKind,
						#[codec(compact)]
						require_weight_at_most: ::core::primitive::u64,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::xcm::v2::multilocation::Junctions),
					#[codec(index = 12)]
					ReportError {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_assets: ::core::primitive::u32,
						beneficiary: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_assets: ::core::primitive::u32,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						receive: runtime_types::xcm::v2::multiasset::MultiAssets,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						reserve: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v2::Xcm,
					},
					#[codec(index = 18)]
					QueryHolding {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						dest: runtime_types::xcm::v2::multilocation::MultiLocation,
						assets: runtime_types::xcm::v2::multiasset::MultiAssetFilter,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::xcm::v2::multiasset::MultiAsset,
						weight_limit: runtime_types::xcm::v2::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::xcm::v2::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::xcm::v2::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::xcm::v2::multiasset::MultiAssets,
						ticket: runtime_types::xcm::v2::multilocation::MultiLocation,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						#[codec(compact)]
						max_response_weight: ::core::primitive::u64,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum NetworkId {
					#[codec(index = 0)]
					Any,
					#[codec(index = 1)]
					Named(
						runtime_types::bounded_collections::weak_bounded_vec::WeakBoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					Polkadot,
					#[codec(index = 3)]
					Kusama,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum OriginKind {
					#[codec(index = 0)]
					Native,
					#[codec(index = 1)]
					SovereignAccount,
					#[codec(index = 2)]
					Superuser,
					#[codec(index = 3)]
					Xcm,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::xcm::v2::multiasset::MultiAssets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v2::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum WeightLimit {
					#[codec(index = 0)]
					Unlimited,
					#[codec(index = 1)]
					Limited(#[codec(compact)] ::core::primitive::u64),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::xcm::v2::Instruction,
					>,
				);
			}
			pub mod v3 {
				use super::runtime_types;
				pub mod junction {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum BodyId {
						#[codec(index = 0)]
						Unit,
						#[codec(index = 1)]
						Moniker([::core::primitive::u8; 4usize]),
						#[codec(index = 2)]
						Index(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						Executive,
						#[codec(index = 4)]
						Technical,
						#[codec(index = 5)]
						Legislative,
						#[codec(index = 6)]
						Judicial,
						#[codec(index = 7)]
						Defense,
						#[codec(index = 8)]
						Administration,
						#[codec(index = 9)]
						Treasury,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum BodyPart {
						#[codec(index = 0)]
						Voice,
						#[codec(index = 1)]
						Members {
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
						#[codec(index = 2)]
						Fraction {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
						#[codec(index = 3)]
						AtLeastProportion {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
						#[codec(index = 4)]
						MoreThanProportion {
							#[codec(compact)]
							nom: ::core::primitive::u32,
							#[codec(compact)]
							denom: ::core::primitive::u32,
						},
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junction {
						#[codec(index = 0)]
						Parachain(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 1)]
						AccountId32 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							id: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						AccountIndex64 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							#[codec(compact)]
							index: ::core::primitive::u64,
						},
						#[codec(index = 3)]
						AccountKey20 {
							network:
								::core::option::Option<runtime_types::xcm::v3::junction::NetworkId>,
							key: [::core::primitive::u8; 20usize],
						},
						#[codec(index = 4)]
						PalletInstance(::core::primitive::u8),
						#[codec(index = 5)]
						GeneralIndex(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 6)]
						GeneralKey {
							length: ::core::primitive::u8,
							data: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 7)]
						OnlyChild,
						#[codec(index = 8)]
						Plurality {
							id: runtime_types::xcm::v3::junction::BodyId,
							part: runtime_types::xcm::v3::junction::BodyPart,
						},
						#[codec(index = 9)]
						GlobalConsensus(runtime_types::xcm::v3::junction::NetworkId),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum NetworkId {
						#[codec(index = 0)]
						ByGenesis([::core::primitive::u8; 32usize]),
						#[codec(index = 1)]
						ByFork {
							block_number: ::core::primitive::u64,
							block_hash: [::core::primitive::u8; 32usize],
						},
						#[codec(index = 2)]
						Polkadot,
						#[codec(index = 3)]
						Kusama,
						#[codec(index = 4)]
						Westend,
						#[codec(index = 5)]
						Rococo,
						#[codec(index = 6)]
						Wococo,
						#[codec(index = 7)]
						Ethereum {
							#[codec(compact)]
							chain_id: ::core::primitive::u64,
						},
						#[codec(index = 8)]
						BitcoinCore,
						#[codec(index = 9)]
						BitcoinCash,
						#[codec(index = 10)]
						PolkadotBulletin,
					}
				}
				pub mod junctions {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Junctions {
						#[codec(index = 0)]
						Here,
						#[codec(index = 1)]
						X1(runtime_types::xcm::v3::junction::Junction),
						#[codec(index = 2)]
						X2(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 3)]
						X3(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 4)]
						X4(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 5)]
						X5(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 6)]
						X6(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 7)]
						X7(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
						#[codec(index = 8)]
						X8(
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
							runtime_types::xcm::v3::junction::Junction,
						),
					}
				}
				pub mod multiasset {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetId {
						#[codec(index = 0)]
						Concrete(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
						#[codec(index = 1)]
						Abstract([::core::primitive::u8; 32usize]),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum AssetInstance {
						#[codec(index = 0)]
						Undefined,
						#[codec(index = 1)]
						Index(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 2)]
						Array4([::core::primitive::u8; 4usize]),
						#[codec(index = 3)]
						Array8([::core::primitive::u8; 8usize]),
						#[codec(index = 4)]
						Array16([::core::primitive::u8; 16usize]),
						#[codec(index = 5)]
						Array32([::core::primitive::u8; 32usize]),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Fungibility {
						#[codec(index = 0)]
						Fungible(#[codec(compact)] ::core::primitive::u128),
						#[codec(index = 1)]
						NonFungible(runtime_types::xcm::v3::multiasset::AssetInstance),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiAsset {
						pub id: runtime_types::xcm::v3::multiasset::AssetId,
						pub fun: runtime_types::xcm::v3::multiasset::Fungibility,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum MultiAssetFilter {
						#[codec(index = 0)]
						Definite(runtime_types::xcm::v3::multiasset::MultiAssets),
						#[codec(index = 1)]
						Wild(runtime_types::xcm::v3::multiasset::WildMultiAsset),
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub struct MultiAssets(
						pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
							runtime_types::xcm::v3::multiasset::MultiAsset,
						>,
					);
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildFungibility {
						#[codec(index = 0)]
						Fungible,
						#[codec(index = 1)]
						NonFungible,
					}
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum WildMultiAsset {
						#[codec(index = 0)]
						All,
						#[codec(index = 1)]
						AllOf {
							id: runtime_types::xcm::v3::multiasset::AssetId,
							fun: runtime_types::xcm::v3::multiasset::WildFungibility,
						},
						#[codec(index = 2)]
						AllCounted(#[codec(compact)] ::core::primitive::u32),
						#[codec(index = 3)]
						AllOfCounted {
							id: runtime_types::xcm::v3::multiasset::AssetId,
							fun: runtime_types::xcm::v3::multiasset::WildFungibility,
							#[codec(compact)]
							count: ::core::primitive::u32,
						},
					}
				}
				pub mod traits {
					use super::runtime_types;
					#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
					pub enum Error {
						#[codec(index = 0)]
						Overflow,
						#[codec(index = 1)]
						Unimplemented,
						#[codec(index = 2)]
						UntrustedReserveLocation,
						#[codec(index = 3)]
						UntrustedTeleportLocation,
						#[codec(index = 4)]
						LocationFull,
						#[codec(index = 5)]
						LocationNotInvertible,
						#[codec(index = 6)]
						BadOrigin,
						#[codec(index = 7)]
						InvalidLocation,
						#[codec(index = 8)]
						AssetNotFound,
						#[codec(index = 9)]
						FailedToTransactAsset,
						#[codec(index = 10)]
						NotWithdrawable,
						#[codec(index = 11)]
						LocationCannotHold,
						#[codec(index = 12)]
						ExceedsMaxMessageSize,
						#[codec(index = 13)]
						DestinationUnsupported,
						#[codec(index = 14)]
						Transport,
						#[codec(index = 15)]
						Unroutable,
						#[codec(index = 16)]
						UnknownClaim,
						#[codec(index = 17)]
						FailedToDecode,
						#[codec(index = 18)]
						MaxWeightInvalid,
						#[codec(index = 19)]
						NotHoldingFees,
						#[codec(index = 20)]
						TooExpensive,
						#[codec(index = 21)]
						Trap(::core::primitive::u64),
						#[codec(index = 22)]
						ExpectationFalse,
						#[codec(index = 23)]
						PalletNotFound,
						#[codec(index = 24)]
						NameMismatch,
						#[codec(index = 25)]
						VersionIncompatible,
						#[codec(index = 26)]
						HoldingWouldOverflow,
						#[codec(index = 27)]
						ExportError,
						#[codec(index = 28)]
						ReanchorFailed,
						#[codec(index = 29)]
						NoDeal,
						#[codec(index = 30)]
						FeesNotMet,
						#[codec(index = 31)]
						LockError,
						#[codec(index = 32)]
						NoPermission,
						#[codec(index = 33)]
						Unanchored,
						#[codec(index = 34)]
						NotDepositable,
						#[codec(index = 35)]
						UnhandledXcmVersion,
						#[codec(index = 36)]
						WeightLimitReached(::sp_weights::Weight),
						#[codec(index = 37)]
						Barrier,
						#[codec(index = 38)]
						WeightNotComputable,
						#[codec(index = 39)]
						ExceedsStackLimit,
					}
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Instruction {
					#[codec(index = 0)]
					WithdrawAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 1)]
					ReserveAssetDeposited(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 2)]
					ReceiveTeleportedAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 3)]
					QueryResponse {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						response: runtime_types::xcm::v3::Response,
						max_weight: ::sp_weights::Weight,
						querier: ::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					},
					#[codec(index = 4)]
					TransferAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						beneficiary: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 5)]
					TransferReserveAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 6)]
					Transact {
						origin_kind: runtime_types::xcm::v3::OriginKind,
						require_weight_at_most: ::sp_weights::Weight,
						call: runtime_types::xcm::double_encoded::DoubleEncoded,
					},
					#[codec(index = 7)]
					HrmpNewChannelOpenRequest {
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						max_message_size: ::core::primitive::u32,
						#[codec(compact)]
						max_capacity: ::core::primitive::u32,
					},
					#[codec(index = 8)]
					HrmpChannelAccepted {
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 9)]
					HrmpChannelClosing {
						#[codec(compact)]
						initiator: ::core::primitive::u32,
						#[codec(compact)]
						sender: ::core::primitive::u32,
						#[codec(compact)]
						recipient: ::core::primitive::u32,
					},
					#[codec(index = 10)]
					ClearOrigin,
					#[codec(index = 11)]
					DescendOrigin(runtime_types::xcm::v3::junctions::Junctions),
					#[codec(index = 12)]
					ReportError(runtime_types::xcm::v3::QueryResponseInfo),
					#[codec(index = 13)]
					DepositAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						beneficiary: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 14)]
					DepositReserveAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 15)]
					ExchangeAsset {
						give: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						want: runtime_types::xcm::v3::multiasset::MultiAssets,
						maximal: ::core::primitive::bool,
					},
					#[codec(index = 16)]
					InitiateReserveWithdraw {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						reserve: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 17)]
					InitiateTeleport {
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
						dest: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 18)]
					ReportHolding {
						response_info: runtime_types::xcm::v3::QueryResponseInfo,
						assets: runtime_types::xcm::v3::multiasset::MultiAssetFilter,
					},
					#[codec(index = 19)]
					BuyExecution {
						fees: runtime_types::xcm::v3::multiasset::MultiAsset,
						weight_limit: runtime_types::xcm::v3::WeightLimit,
					},
					#[codec(index = 20)]
					RefundSurplus,
					#[codec(index = 21)]
					SetErrorHandler(runtime_types::xcm::v3::Xcm),
					#[codec(index = 22)]
					SetAppendix(runtime_types::xcm::v3::Xcm),
					#[codec(index = 23)]
					ClearError,
					#[codec(index = 24)]
					ClaimAsset {
						assets: runtime_types::xcm::v3::multiasset::MultiAssets,
						ticket: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 25)]
					Trap(#[codec(compact)] ::core::primitive::u64),
					#[codec(index = 26)]
					SubscribeVersion {
						#[codec(compact)]
						query_id: ::core::primitive::u64,
						max_response_weight: ::sp_weights::Weight,
					},
					#[codec(index = 27)]
					UnsubscribeVersion,
					#[codec(index = 28)]
					BurnAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 29)]
					ExpectAsset(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 30)]
					ExpectOrigin(
						::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					),
					#[codec(index = 31)]
					ExpectError(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 32)]
					ExpectTransactStatus(runtime_types::xcm::v3::MaybeErrorCode),
					#[codec(index = 33)]
					QueryPallet {
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						response_info: runtime_types::xcm::v3::QueryResponseInfo,
					},
					#[codec(index = 34)]
					ExpectPallet {
						#[codec(compact)]
						index: ::core::primitive::u32,
						name: ::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						module_name:
							::subxt::ext::subxt_core::alloc::vec::Vec<::core::primitive::u8>,
						#[codec(compact)]
						crate_major: ::core::primitive::u32,
						#[codec(compact)]
						min_crate_minor: ::core::primitive::u32,
					},
					#[codec(index = 35)]
					ReportTransactStatus(runtime_types::xcm::v3::QueryResponseInfo),
					#[codec(index = 36)]
					ClearTransactStatus,
					#[codec(index = 37)]
					UniversalOrigin(runtime_types::xcm::v3::junction::Junction),
					#[codec(index = 38)]
					ExportMessage {
						network: runtime_types::xcm::v3::junction::NetworkId,
						destination: runtime_types::xcm::v3::junctions::Junctions,
						xcm: runtime_types::xcm::v3::Xcm,
					},
					#[codec(index = 39)]
					LockAsset {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						unlocker: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 40)]
					UnlockAsset {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						target: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 41)]
					NoteUnlockable {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						owner: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 42)]
					RequestUnlock {
						asset: runtime_types::xcm::v3::multiasset::MultiAsset,
						locker: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					},
					#[codec(index = 43)]
					SetFeesMode { jit_withdraw: ::core::primitive::bool },
					#[codec(index = 44)]
					SetTopic([::core::primitive::u8; 32usize]),
					#[codec(index = 45)]
					ClearTopic,
					#[codec(index = 46)]
					AliasOrigin(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
					#[codec(index = 47)]
					UnpaidExecution {
						weight_limit: runtime_types::xcm::v3::WeightLimit,
						check_origin: ::core::option::Option<
							runtime_types::staging_xcm::v3::multilocation::MultiLocation,
						>,
					},
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum MaybeErrorCode {
					#[codec(index = 0)]
					Success,
					#[codec(index = 1)]
					Error(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
					#[codec(index = 2)]
					TruncatedError(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							::core::primitive::u8,
						>,
					),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum OriginKind {
					#[codec(index = 0)]
					Native,
					#[codec(index = 1)]
					SovereignAccount,
					#[codec(index = 2)]
					Superuser,
					#[codec(index = 3)]
					Xcm,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct PalletInfo {
					#[codec(compact)]
					pub index: ::core::primitive::u32,
					pub name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					pub module_name: runtime_types::bounded_collections::bounded_vec::BoundedVec<
						::core::primitive::u8,
					>,
					#[codec(compact)]
					pub major: ::core::primitive::u32,
					#[codec(compact)]
					pub minor: ::core::primitive::u32,
					#[codec(compact)]
					pub patch: ::core::primitive::u32,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct QueryResponseInfo {
					pub destination: runtime_types::staging_xcm::v3::multilocation::MultiLocation,
					#[codec(compact)]
					pub query_id: ::core::primitive::u64,
					pub max_weight: ::sp_weights::Weight,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Response {
					#[codec(index = 0)]
					Null,
					#[codec(index = 1)]
					Assets(runtime_types::xcm::v3::multiasset::MultiAssets),
					#[codec(index = 2)]
					ExecutionResult(
						::core::option::Option<(
							::core::primitive::u32,
							runtime_types::xcm::v3::traits::Error,
						)>,
					),
					#[codec(index = 3)]
					Version(::core::primitive::u32),
					#[codec(index = 4)]
					PalletsInfo(
						runtime_types::bounded_collections::bounded_vec::BoundedVec<
							runtime_types::xcm::v3::PalletInfo,
						>,
					),
					#[codec(index = 5)]
					DispatchResult(runtime_types::xcm::v3::MaybeErrorCode),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum WeightLimit {
					#[codec(index = 0)]
					Unlimited,
					#[codec(index = 1)]
					Limited(::sp_weights::Weight),
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct Xcm(
					pub  ::subxt::ext::subxt_core::alloc::vec::Vec<
						runtime_types::xcm::v3::Instruction,
					>,
				);
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum VersionedAssetId {
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::multiasset::AssetId),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::asset::AssetId),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum VersionedAssets {
				#[codec(index = 1)]
				V2(runtime_types::xcm::v2::multiasset::MultiAssets),
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::multiasset::MultiAssets),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::asset::Assets),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum VersionedLocation {
				#[codec(index = 1)]
				V2(runtime_types::xcm::v2::multilocation::MultiLocation),
				#[codec(index = 3)]
				V3(runtime_types::staging_xcm::v3::multilocation::MultiLocation),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::location::Location),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum VersionedResponse {
				#[codec(index = 2)]
				V2(runtime_types::xcm::v2::Response),
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::Response),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::Response),
			}
			#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
			pub enum VersionedXcm {
				#[codec(index = 2)]
				V2(runtime_types::xcm::v2::Xcm),
				#[codec(index = 3)]
				V3(runtime_types::xcm::v3::Xcm),
				#[codec(index = 4)]
				V4(runtime_types::staging_xcm::v4::Xcm),
			}
		}
		pub mod xcm_runtime_apis {
			use super::runtime_types;
			pub mod conversions {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					Unsupported,
					#[codec(index = 1)]
					VersionedConversionFailed,
				}
			}
			pub mod dry_run {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct CallDryRunEffects<_0> {
					pub execution_result: ::core::result::Result<
						runtime_types::frame_support::dispatch::PostDispatchInfo,
						runtime_types::sp_runtime::DispatchErrorWithPostInfo<
							runtime_types::frame_support::dispatch::PostDispatchInfo,
						>,
					>,
					pub emitted_events: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
					pub local_xcm: ::core::option::Option<runtime_types::xcm::VersionedXcm>,
					pub forwarded_xcms: ::subxt::ext::subxt_core::alloc::vec::Vec<(
						runtime_types::xcm::VersionedLocation,
						::subxt::ext::subxt_core::alloc::vec::Vec<runtime_types::xcm::VersionedXcm>,
					)>,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					Unimplemented,
					#[codec(index = 1)]
					VersionedConversionFailed,
				}
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub struct XcmDryRunEffects<_0> {
					pub execution_result: runtime_types::staging_xcm::v4::traits::Outcome,
					pub emitted_events: ::subxt::ext::subxt_core::alloc::vec::Vec<_0>,
					pub forwarded_xcms: ::subxt::ext::subxt_core::alloc::vec::Vec<(
						runtime_types::xcm::VersionedLocation,
						::subxt::ext::subxt_core::alloc::vec::Vec<runtime_types::xcm::VersionedXcm>,
					)>,
				}
			}
			pub mod fees {
				use super::runtime_types;
				#[derive(::codec::Decode, ::codec::Encode, Clone, Debug, PartialEq)]
				pub enum Error {
					#[codec(index = 0)]
					Unimplemented,
					#[codec(index = 1)]
					VersionedConversionFailed,
					#[codec(index = 2)]
					WeightNotComputable,
					#[codec(index = 3)]
					UnhandledXcmVersion,
					#[codec(index = 4)]
					AssetNotFound,
					#[codec(index = 5)]
					Unroutable,
				}
			}
		}
	}
}
