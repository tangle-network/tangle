<h1> Roles Pallet </h1>
The Roles Pallet introduces the concept of roles for validators within the Tangle blockchain. Validators can opt into one or more roles by restaking their tokens.

By opting into a role, validators agree to new slashing conditions specific to that role and are rewarded accordingly.

---

<h2>Types of Role</h2>

- ##### TSS Roles
    Validators can opt into various threshold signature schemes roles, following are different types of TSS roles 
    - ZengoGG20Secp256k1, 
    - DfnsCGGMP21Secp256k1
    - DfnsCGGMP21Secp256r1
    - DfnsCGGMP21Stark
    - ZcashFrostP256

- ##### ZkSaaS Roles,
    Validators can opt into various zero-knowledge proof (ZkSaaS) roles, following are different type of ZKSaaS roles 
    - ZkSaaSGroth16
    - ZkSaaSMarlin

- ##### LightClientRelaying Role
    This role is designed for validators participating in light client relaying services.

---

<h2>Restaking</h2>
Restaking enables validators to reuse their staked amounts to provide various services within the Tangle network. When validators opt into roles and restake tokens, they agree to adhere to new slashing conditions associated with the selected roles.

Currently we allow max 50% of staked amount that can be restaked for providing various services within the Tangle network.

- ##### Independent Restaking: 
    Validators can restake independently for each role individually. The staked amount allocated to each role is subject to slashing conditions specific to that role. Rewards are distributed based on the allocated restake for each role.

- ##### Shared Restaking:
    Validators can opt for shared restaking, allowing their restaked amount to be distributed across all roles. This provides flexibility and a more collaborative approach to restaking


---

<h2>Dispatchable calls</h2>

- ##### create_profile
    Validators can create a profile, specifying the roles they want to participate in and the restaking amounts for each role. Profiles can be shared or independent, and the total restaking amount should meet minimum requirements.

- ##### update_profile
    Validators can update their profiles, adjusting restaking amounts for roles. The update operation ensures that the new restaking amounts meet the minimum requirements and do not exceed the maximum allowed.

- ##### delete_profile
    Validators can submit a request to delete their profile, initiating the process to exit from all services. This operation fails if there are pending jobs associated with the roles.

- ##### chill
    Validators can declare no desire to validate or nominate, effectively opting out of all roles and services.

- ##### unbond_funds
    Validators can unbond funds, these operations are only allowed when the validator has no active roles.

- ##### withdraw_unbonded
    withdraw unbonded funds after a certain period. These operations are only allowed when the validator has no active roles.

---

<h2> Contributing </h2>

Interested in contributing to the Webb Tangle Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](../../.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

---

<h2> License </h2>

Licensed under <a href="LICENSE">GNU General Public License v3.0</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the GNU General Public License v3.0 license, shall be licensed as above, without any additional terms or conditions.




