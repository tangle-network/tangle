//! Uses substrate's child trie api for ICS23 vector commitment.
//! This allows us to progressively mutate the trie and recalculate its root in O(log n).
//!
//! A much better approach than having to reconstruct the trie every time its changed
//! just to recalculate its root hash.

pub mod acknowledgements;
pub mod channels;
pub mod client_states;
pub mod clients;
pub mod connections;
pub mod consensus_states;
pub mod next_seq_ack;
pub mod next_seq_recv;
pub mod next_seq_send;
pub mod packet_commitments;
pub mod receipts;
