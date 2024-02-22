<h1> Roles Pallet </h1>
The Roles Pallet introduces the concept of roles for validators within the Tangle blockchain. Validators can opt into one or more roles by restaking their tokens.

By opting into a role, validators agree to new slashing conditions specific to that role and are rewarded accordingly.

---

<h2>Types of Role</h2>

- ##### TSS Roles
    Validators can opt into various threshold signature schemes roles, following are different types of TSS roles 
    - DfnsCGGMP21Secp256k1
    - DfnsCGGMP21Secp256r1
    - DfnsCGGMP21Stark
    - ZcashFrostP256
    - ZcashFrostP384
    - ZcashFrostSecp256k1
    - ZcashFrostEd25519
    - ZcashFrostEd448
    - ZcashFrostRistretto255

- ##### ZkSaaS Roles,
    Validators can opt into various zero-knowledge proof (ZkSaaS) roles, following are different type of ZKSaaS roles 
    - ZkSaaSGroth16

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
    Validators can withdraw unbonded funds after a certain period. These operations are only allowed when the validator has no active roles.

---

<h2>Rewards</h2>

The validator rewards system is designed to distribute rewards among validators in the current era based on their contributions and stake in the system. Here's a summary of how the rewards are computed:

### 1. Active Validator Rewards (50%)

- **Function:** `compute_active_validator_rewards`
- **Purpose:** Calculates rewards for validators who have completed jobs in the current era.
- **Method:**
  - Retrieves active validators and their completed jobs from storage.
  - Calculates the total number of jobs completed by all active validators.
  - Distributes rewards among active validators proportionally to the number of jobs they have completed relative to the total jobs completed.
- **Formula:**

    1. **Calculate Total Jobs Completed by Active Validators:** $\text{total jobs completed}$ (TJC), $\text{jobs completed}$ (JC)
    $$\text{TJC} = \sum_{\text{validator}} \text{JC}\_{\text{validator}}$$

    2. **Compute Reward Share for Each Active Validator $v$:** $\text{validator share}$ (VS)
    $$\text{VS}\_v = \frac{\text{JC}_v}{\text{TJC}}$$

    3. **Compute Reward for Each Active Validator $v$:** $\text{validator reward}$ (VR)

    $$\text{VR}_v = \text{VS}_v \times R$$


### 2. Validator Rewards by Restake (50%)

- **Function:** `compute_validator_rewards_by_restake`
- **Purpose:** Computes rewards for validators based on the amount they have staked in the system.
- **Method:**
  - Retrieves the total stake in the system and the restake amount of individual validators.
  - Calculates the ratio of restake to total stake in the system.
  - Adjusts the total rewards based on the missing restake ratio to ensure rewards are distributed properly.
  - Calculates rewards for each validator based on their restake amount relative to the total restake in the system.
- **Formula:**

    1. **Compute Total Restake in the System:** $\text{total restake}$ (TR)
    $$TR = \sum_{i=1}^{n} R_i$$

    3. **Compute Restake-to-Stake Ratio:** $\text{Restake-to-Stake Ratio}$ (RSR)
    $$\text{RSR} = \frac{TR}{S_{\text{era}}}$$

    4. **Compute Missing Restake Ratio:** $\text{Missing Restake Ratio}$ (MRR)
    $$\text{MRR} = \text{MaxRestake} - \text{RSR}$$

    5. **Adjust Total Rewards:** $\text{Adjusted Total Rewards}$ (ATR)
        - $\text{if } \text{MRR} \neq 0$, $\text{ATR} = (100 - \text{MRR}) \times R$
        - $\text{otherwise } \text{ATR} = R$

    6. **Compute Reward Share for Each Restaker:** $\text{RS}_i$
    $$\text{RS}_i = \frac{R_i}{TR}$$

    7. **Compute Reward for Each Restaker:** $\text{Reward}_i$
    $$\text{Reward}_i = \text{RS}_i \times \text{ATR}$$


#### Example:
Lets take an example in era 100, we have 20 restakers at era 100, and the roles reward for era is 1000TNT
10 restakers have completed 5 jobs each.

1. Active Validator Rewards
    - 50% of 1000TNT is meant for active restakers (completed atleast one job in last era)
    - 10 restakers have completed 5 jobs each, since everyone completed the same amount of jobs, 500TNT is equally divided among all 10 restakers.
    - If one restaker had completed a higher number of jobs compared to the rest of the restakers, they would get a larger share of the rewards.

2. Rewards by restake
    - 50% of 1000TNT is meant for all restakers, as long as you were restaked in the era, you are eligible for a share of the reward
    - The restaker reward share is determined by the amount of restake, if a restaker has restaked 100TNT and the total restake in the system is 1000TNT, then the restaker is eligible for 10% of the rewards.
    - The restaker rewards are also weighted by the total restake in the system compared to the total stake. If the total stake in the system is 100_000 TNT and only 1000TNT is restaked (1%), then the total rewards are reduced propotional to this value.


---

<h2> Contributing </h2>

Interested in contributing to the Tangle Network? Thank you so much for your interest! We are always appreciative for contributions from the open-source community!

If you have a contribution in mind, please check out our [Contribution Guide](../../.github/CONTRIBUTING.md) for information on how to do so. We are excited for your first contribution!

---

<h2> License </h2>

Licensed under <a href="LICENSE">GNU General Public License v3.0</a>.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you, as defined in the GNU General Public License v3.0 license, shall be licensed as above, without any additional terms or conditions.



