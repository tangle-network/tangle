use super::*;
use core::fmt::Debug;
use ibc::{
	applications::transfer::MODULE_ID_STR as IBC_TRANSFER_MODULE_ID,
	core::{
		ics24_host::identifier::PortId,
		ics26_routing::context::{
			Ics26Context, Module, ModuleCallbackContext, ModuleId, ReaderContext, Router,
		},
	},
};
use sp_core::crypto::AccountId32;

#[derive(Clone, Eq, PartialEq, Debug)]
pub(crate) struct Context<T: Config> {
	pub _pd: PhantomData<T>,
	router: IbcRouter<T>,
}

impl<T: Config + Send + Sync> Default for Context<T> {
	fn default() -> Self {
		Self { _pd: PhantomData, router: IbcRouter::default() }
	}
}

impl<T: Config + Send + Sync> Context<T> {
	pub fn new() -> Self {
		Self::default()
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IbcRouter<T: Config> {
	ibc_transfer: ics20::IbcModule<T>,
	sub_router: T::Router,
}

impl<T: Config> Default for IbcRouter<T> {
	fn default() -> Self {
		Self { ibc_transfer: ics20::IbcModule::<T>::default(), sub_router: Default::default() }
	}
}

/// Module routing abstraction for downstream substrate runtimes.
pub trait ModuleRouter: Default + Clone + Eq + PartialEq + Debug {
	/// Returns a mutable reference to a `Module` registered against the specified `ModuleId`
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module>;
	/// Returns true if the `Router` has a `Module` registered against the specified `ModuleId`
	fn has_route(module_id: &ModuleId) -> bool;
	/// Should return the module_id associated with a given port_id
	fn lookup_module_by_port(port_id: &PortId) -> Option<ModuleId>;
}

impl<T: Config + Send + Sync> Router for IbcRouter<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
		// check if the user has defined any custom routes
		if let Some(module) = self.sub_router.get_route_mut(module_id) {
			return Some(module);
		}

		match module_id.as_ref() {
			IBC_TRANSFER_MODULE_ID => Some(&mut self.ibc_transfer),
			&_ => None,
		}
	}

	fn has_route(&self, module_id: &ModuleId) -> bool {
		// check if the user has defined any custom routes
		if T::Router::has_route(module_id) {
			return true;
		}

		matches!(module_id.to_string().as_str(), IBC_TRANSFER_MODULE_ID,)
	}
}

impl<T: Config + Send + Sync> Ics26Context for Context<T>
where
	AccountId32: From<<T as frame_system::Config>::AccountId>,
{
	type Router = IbcRouter<T>;

	fn router(&self) -> &Self::Router {
		&self.router
	}

	fn router_mut(&mut self) -> &mut Self::Router {
		&mut self.router
	}
}

impl<T: Config + Send + Sync> ReaderContext for Context<T> where
	AccountId32: From<<T as frame_system::Config>::AccountId>
{
}

impl<T: Config + Send + Sync> ModuleCallbackContext for Context<T> where
	AccountId32: From<<T as frame_system::Config>::AccountId>
{
}

impl ModuleRouter for () {
	fn get_route_mut(&mut self, _module_id: &ModuleId) -> Option<&mut dyn Module> {
		None
	}

	fn has_route(_module_id: &ModuleId) -> bool {
		false
	}

	fn lookup_module_by_port(_port_id: &PortId) -> Option<ModuleId> {
		None
	}
}
