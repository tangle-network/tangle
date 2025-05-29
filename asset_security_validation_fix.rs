use frame_support::{
    ensure,
    traits::{Get, tokens::fungibles::Inspect},
    dispatch::DispatchResult,
};
use sp_runtime::{traits::Zero, Percent};
use sp_std::{collections::btree_set::BTreeSet, vec::Vec};
use pallet_services::{
    AssetSecurityCommitment, AssetSecurityRequirement, Config, Error, Pallet,
    types::{Asset, AssetIdT},
};

/// Fix for asset security commitment validation bug #981
impl<T: Config> Pallet<T> {
    
    /// Validate asset security requirements during service request
    /// Fixes: Deployers can include non-existent assets in the asset security commitment
    pub fn validate_asset_security_requirements(
        requirements: &[AssetSecurityRequirement<T::AssetId>],
    ) -> DispatchResult {
        let mut seen_assets = BTreeSet::new();
        
        for requirement in requirements {
            // Check for duplicate assets
            ensure!(
                seen_assets.insert(&requirement.asset),
                Error::<T>::DuplicateAsset
            );
            
            // Validate asset exists
            Self::validate_asset_exists(&requirement.asset)?;
            
            // Validate percentage ranges
            ensure!(
                requirement.min_exposure_percent <= requirement.max_exposure_percent,
                Error::<T>::InvalidSecurityRequirements
            );
            
            ensure!(
                requirement.max_exposure_percent <= Percent::from_percent(100),
                Error::<T>::InvalidSecurityRequirements
            );
            
            ensure!(
                !requirement.min_exposure_percent.is_zero(),
                Error::<T>::InvalidSecurityRequirements
            );
        }
        
        // Ensure at least one native asset requirement if configured
        if T::MinimumNativeSecurityRequirement::get() > Percent::zero() {
            let has_native_asset = requirements.iter().any(|req| {
                matches!(req.asset, Asset::Custom(asset_id) if asset_id == Zero::zero())
            });
            ensure!(has_native_asset, Error::<T>::NoNativeAsset);
        }
        
        Ok(())
    }
    
    /// Validate asset security commitments during service approval
    /// Fixes: Operators can approve service requests without having delegated assets
    pub fn validate_asset_security_commitments(
        operator: &T::AccountId,
        commitments: &[AssetSecurityCommitment<T::AssetId>],
        requirements: &[AssetSecurityRequirement<T::AssetId>],
    ) -> DispatchResult {
        let mut seen_assets = BTreeSet::new();
        
        for commitment in commitments {
            // Check for duplicate assets in commitments
            ensure!(
                seen_assets.insert(&commitment.asset),
                Error::<T>::DuplicateAsset
            );
            
            // Validate asset exists
            Self::validate_asset_exists(&commitment.asset)?;
            
            // Validate operator has sufficient delegated stake for this asset
            Self::validate_operator_asset_delegation(operator, commitment)?;
            
            // Validate commitment meets requirements
            Self::validate_commitment_meets_requirement(commitment, requirements)?;
        }
        
        // Ensure all required assets have commitments
        Self::validate_all_requirements_covered(commitments, requirements)?;
        
        Ok(())
    }
    
    /// Validate that an asset exists and is valid
    fn validate_asset_exists(asset: &Asset<T::AssetId>) -> DispatchResult {
        match asset {
            Asset::Custom(asset_id) => {
                if *asset_id == Zero::zero() {
                    // Native asset always exists
                    Ok(())
                } else {
                    // Check if custom asset exists in pallet-assets or similar
                    ensure!(
                        T::Fungibles::asset_exists(*asset_id),
                        Error::<T>::AssetNotFound
                    );
                    Ok(())
                }
            },
            Asset::Erc20(token_address) => {
                // Validate ERC20 token exists and is not zero address
                ensure!(
                    *token_address != sp_core::H160::zero(),
                    Error::<T>::InvalidErc20Address
                );
                
                // Additional ERC20 validation could be added here
                // e.g., checking if contract exists via EVM calls
                Ok(())
            },
        }
    }
    
    /// Validate operator has sufficient delegated stake for the asset commitment
    fn validate_operator_asset_delegation(
        operator: &T::AccountId,
        commitment: &AssetSecurityCommitment<T::AssetId>,
    ) -> DispatchResult {
        // Get operator's total delegated stake for this asset
        let delegated_amount = T::OperatorDelegationManager::get_operator_asset_stake(
            operator,
            &commitment.asset.clone().into(),
        );
        
        // Ensure operator has at least the committed amount
        ensure!(
            delegated_amount >= commitment.amount,
            Error::<T>::InsufficientDelegatedStake
        );
        
        // Additional validation: check if operator is active
        ensure!(
            T::OperatorDelegationManager::is_operator_active(operator),
            Error::<T>::OperatorNotActive
        );
        
        Ok(())
    }
    
    /// Validate that commitment meets the requirement specifications
    fn validate_commitment_meets_requirement(
        commitment: &AssetSecurityCommitment<T::AssetId>,
        requirements: &[AssetSecurityRequirement<T::AssetId>],
    ) -> DispatchResult {
        // Find matching requirement for this asset
        let requirement = requirements
            .iter()
            .find(|req| req.asset == commitment.asset)
            .ok_or(Error::<T>::UnexpectedAssetCommitment)?;
        
        // Get operator's total stake to calculate percentage
        let total_stake = T::OperatorDelegationManager::get_operator_total_stake(
            &commitment.operator,
        );
        
        ensure!(!total_stake.is_zero(), Error::<T>::NoOperatorStake);
        
        // Calculate commitment percentage
        let commitment_percent = Percent::from_rational(commitment.amount, total_stake);
        
        // Validate commitment is within required range
        ensure!(
            commitment_percent >= requirement.min_exposure_percent,
            Error::<T>::CommitmentBelowMinimum
        );
        
        ensure!(
            commitment_percent <= requirement.max_exposure_percent,
            Error::<T>::CommitmentAboveMaximum
        );
        
        Ok(())
    }
    
    /// Ensure all asset requirements have corresponding commitments
    fn validate_all_requirements_covered(
        commitments: &[AssetSecurityCommitment<T::AssetId>],
        requirements: &[AssetSecurityRequirement<T::AssetId>],
    ) -> DispatchResult {
        for requirement in requirements {
            let has_commitment = commitments
                .iter()
                .any(|commitment| commitment.asset == requirement.asset);
            
            ensure!(
                has_commitment,
                Error::<T>::MissingAssetCommitment
            );
        }
        
        Ok(())
    }
}

/// Enhanced error types for asset security validation
#[derive(Debug)]
pub enum AssetSecurityError {
    AssetNotFound,
    InvalidErc20Address,
    InsufficientDelegatedStake,
    UnexpectedAssetCommitment,
    NoOperatorStake,
    CommitmentBelowMinimum,
    CommitmentAboveMaximum,
    MissingAssetCommitment,
}

/// Integration with existing service request validation
impl<T: Config> Pallet<T> {
    
    /// Enhanced request validation with asset security checks
    pub fn do_request_with_security_validation(
        owner: &T::AccountId,
        blueprint_id: u64,
        asset_security_requirements: Vec<AssetSecurityRequirement<T::AssetId>>,
        operators: Vec<T::AccountId>,
        // ... other parameters
    ) -> DispatchResult {
        // Existing validations...
        
        // NEW: Validate asset security requirements
        Self::validate_asset_security_requirements(&asset_security_requirements)?;
        
        // Ensure all operators can potentially meet the requirements
        for operator in &operators {
            Self::validate_operator_can_meet_requirements(
                operator,
                &asset_security_requirements,
            )?;
        }
        
        // Continue with existing request logic...
        Ok(())
    }
    
    /// Enhanced approve validation with asset security checks  
    pub fn do_approve_with_security_validation(
        operator: &T::AccountId,
        request_id: u64,
        security_commitments: Vec<AssetSecurityCommitment<T::AssetId>>,
    ) -> DispatchResult {
        // Get service request
        let request = Self::service_requests(request_id)?;
        
        // NEW: Validate security commitments
        Self::validate_asset_security_commitments(
            operator,
            &security_commitments,
            &request.asset_security_requirements,
        )?;
        
        // Continue with existing approval logic...
        Ok(())
    }
    
    /// Check if operator can potentially meet security requirements
    fn validate_operator_can_meet_requirements(
        operator: &T::AccountId,
        requirements: &[AssetSecurityRequirement<T::AssetId>],
    ) -> DispatchResult {
        for requirement in requirements {
            let delegated_amount = T::OperatorDelegationManager::get_operator_asset_stake(
                operator,
                &requirement.asset.clone().into(),
            );
            
            // Check if operator has any delegation for this asset
            ensure!(
                !delegated_amount.is_zero(),
                Error::<T>::OperatorHasNoAssetStake
            );
        }
        
        Ok(())
    }
}

/// Additional error variants to add to the pallet
/*
Add these to the Error enum in lib.rs:

/// Asset not found or doesn't exist
AssetNotFound,
/// Invalid ERC20 token address (zero address)
InvalidErc20Address,  
/// Operator doesn't have sufficient delegated stake for commitment
InsufficientDelegatedStake,
/// Asset commitment provided but not required
UnexpectedAssetCommitment,
/// Operator has no stake at all
NoOperatorStake,
/// Commitment percentage below minimum requirement
CommitmentBelowMinimum,
/// Commitment percentage above maximum requirement  
CommitmentAboveMaximum,
/// Required asset has no corresponding commitment
MissingAssetCommitment,
/// Operator has no stake for required asset
OperatorHasNoAssetStake,
*/

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{assert_err, assert_ok};
    
    #[test]
    fn test_validate_asset_exists_native() {
        // Test native asset validation
        let native_asset = Asset::Custom(0u32.into());
        assert_ok!(TestPallet::validate_asset_exists(&native_asset));
    }
    
    #[test]
    fn test_validate_asset_exists_invalid_erc20() {
        // Test invalid ERC20 address
        let invalid_erc20 = Asset::Erc20(sp_core::H160::zero());
        assert_err!(
            TestPallet::validate_asset_exists(&invalid_erc20),
            Error::<Test>::InvalidErc20Address
        );
    }
    
    #[test]
    fn test_duplicate_asset_requirements() {
        let requirements = vec![
            AssetSecurityRequirement {
                asset: Asset::Custom(1u32.into()),
                min_exposure_percent: Percent::from_percent(10),
                max_exposure_percent: Percent::from_percent(20),
            },
            AssetSecurityRequirement {
                asset: Asset::Custom(1u32.into()), // Duplicate!
                min_exposure_percent: Percent::from_percent(5),
                max_exposure_percent: Percent::from_percent(15),
            },
        ];
        
        assert_err!(
            TestPallet::validate_asset_security_requirements(&requirements),
            Error::<Test>::DuplicateAsset
        );
    }
    
    #[test]
    fn test_insufficient_delegated_stake() {
        let operator = AccountId::from([1u8; 32]);
        let commitment = AssetSecurityCommitment {
            operator: operator.clone(),
            asset: Asset::Custom(1u32.into()),
            amount: 1000u64, // More than operator has
        };
        
        // Mock operator has only 500 delegated
        MockDelegationManager::set_operator_stake(&operator, 500u64);
        
        assert_err!(
            TestPallet::validate_operator_asset_delegation(&operator, &commitment),
            Error::<Test>::InsufficientDelegatedStake
        );
    }
} 