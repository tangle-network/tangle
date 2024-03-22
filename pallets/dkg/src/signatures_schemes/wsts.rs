use std::collections::HashMap;
use sp_runtime::DispatchResult;
use wsts::common::PolyCommitment;
use crate::{Config, Error};

pub fn verify_wsts_signature<T: Config>(msg: Vec<u8>, signature: Vec<u8>, verifying_key: Vec<u8>) -> DispatchResult {
    let signature: wsts::common::Signature = bincode2::deserialize(&signature).map_err(|_| Error::<T>::InvalidSignatureDeserialization)?;
    let verifying_key: HashMap<u32, PolyCommitment> = bincode2::deserialize(&verifying_key).map_err(|_| Error::<T>::InvalidVerifyingKey)?;
    let public_key_point = verifying_key.get(&0).ok_or(Error::<T>::InvalidVerifyingKey)?.poly.clone();
    if signature.verify(&public_key_point, &msg) {
        Ok(())
    } else {
        Err(Error::<T>::InvalidSignature.into())
    }
}