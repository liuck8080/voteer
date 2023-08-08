//! This contract implements simple election backed by storage on blockchain.
//!
//! The contract provides methods to vote options
//!
use std::collections::HashSet;
use std::vec;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault};

near_sdk::setup_alloc!();

#[derive(BorshSerialize, BorshStorageKey)]
enum Prefix {
    ElectionDict,
    CandidateMap { hash: Vec<u8> },// option => supporter set
    CandidateVec { hash: Vec<u8> },
    SupporterSet { hash: Vec<u8> }, // supporter set of an option
    UserElectionMap,
    UserElectionSet { hash: Vec<u8> },
}

impl Prefix {
    fn candicate_map(election_id: u128) -> Self {
        let mut bytes = b"CandidateMap".to_vec();
        bytes.extend(election_id.to_le_bytes());
        Prefix::CandidateMap {
            hash: env::sha256(&bytes),
        }
    }
    fn candicate_vec(election_id: u128) -> Self {
        let mut bytes = b"CandidateVec".to_vec();
        bytes.extend(election_id.to_le_bytes());
        Prefix::CandidateVec {
            hash: env::sha256(&bytes),
        }
    }

    fn supporter_set(election_id: u128, idx: usize) -> Self {
        let mut bytes = b"SupporterSet".to_vec();
        bytes.extend(election_id.to_le_bytes());
        bytes.extend(idx.to_le_bytes());
        Prefix::SupporterSet {
            hash: env::sha256(&bytes),
        }
    }

    fn user_election_set(user: &AccountId) -> Self {
        let mut bytes = b"UserElectionSet".to_vec();
        bytes.push(b'_');
        bytes.extend(user.as_bytes());
        Prefix::UserElectionSet {
            hash: env::sha256(&bytes),
        }
    }
}

/**
 * this structure only for frontend display
 */
#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Candidate {
    name: String,
    supported: u64, // supporter number, dynamically get from chain
}

#[derive(Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ElectionInfo {
    name: String, // election name
    id: U128, // election id
}

///! this is an election provide several options;
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Election {
    // Option => <supporter set>
    dict: Vector<UnorderedSet<AccountId>>,
    // option list, each option's supporters account ids are stored in according dict's set
    candidates: Vector<String>,
    // the name of the election
    name: String,
    // if election support multiple selection
    multiple: bool,
}

impl Election {
    pub fn new(candidates: &Vec<String>, multiple: bool, election_id: u128, name: &str) -> Self {
        let mut res = Election {
            name: name.to_string(),
            dict: Vector::new(Prefix::candicate_map(election_id)),
            candidates: Vector::new(Prefix::candicate_vec(election_id)),
            multiple,
        };
        let mut seen = HashSet::new(); // reduce duplicates
        for candidate in candidates {
            if !seen.contains(candidate) {
                res.dict.push(&UnorderedSet::new(Prefix::supporter_set(
                    election_id,
                    res.candidates.len() as usize,
                )));
                res.candidates.push(candidate);
                seen.insert(candidate);
            }
        }
        res
    }

    pub fn vote(&mut self, options: &Vec<String>, owner_id: AccountId) -> bool {
        if !((self.multiple && options.len() >= 1) || (!self.multiple && options.len() == 1)) {
            return false;
        }

        let mut changed = false;
        for (idx, candidate) in self.candidates.iter().enumerate() {
            let mut support_set = self.dict.get(idx as u64).unwrap();
            if options.contains(&candidate) {
                //vote for
                if support_set.insert(&owner_id) {
                    self.dict.replace(idx as u64, &support_set);
                    changed = true;
                }
            } else {
                // not for, should remove
                if support_set.remove(&owner_id) {
                    self.dict.replace(idx as u64, &support_set);
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn revoke_vote(&mut self, options: &Vec<String>, owner_id: AccountId) -> bool {
        let mut changed = false;
        self.candidates
            .iter()
            .enumerate()
            .filter(|item| options.iter().any(|y| item.1 == y.as_str()))
            .map(|option| option.0)
            .for_each(|idx| {
                let mut su = self.dict.get(idx as u64).unwrap();
                if su.remove(&owner_id) {
                    self.dict.replace(idx as u64, &su);
                    changed = true
                }
            });
        changed
    }

    pub fn get_candidates(&self) -> Vec<Candidate> {
        let mut ans = Vec::with_capacity(self.candidates.len() as usize);
        for (idx, option) in self.candidates.iter().enumerate() {
            ans.push(Candidate {
                name: option,
                supported: self.dict.get(idx as u64).unwrap().len(),
            });
        }
        ans
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Voteer {
    election_count: u128,
    elections: UnorderedMap<u128, Election>,
    user_election: UnorderedMap<AccountId, Vector<u128>>,
}

#[near_bindgen]
impl Voteer {
    #[init]
    pub fn new() -> Voteer {
        Voteer {
            election_count: 0,
            elections: UnorderedMap::new(Prefix::ElectionDict),
            user_election: UnorderedMap::new(Prefix::UserElectionMap),
        }
    }

    #[payable]
    pub fn create_election(
        &mut self,
        candidates: &Vec<String>,
        multiple: bool,
        name: &String,
    ) -> U128 {
        let election_id = self.election_count;
        self.election_count += 1;
        let elect = Election::new(candidates, multiple, election_id, name);
        self.elections.insert(&election_id, &elect);
        {
            // update user's election set
            let user_id = env::signer_account_id();
            let mut election_vec = self
                .user_election
                .get(&user_id)
                .unwrap_or_else(|| Vector::new(Prefix::user_election_set(&user_id)));
            election_vec.push(&election_id);
            self.user_election.insert(&user_id, &election_vec);
        }
        election_id.into()
    }

    pub fn vote(&mut self, election_id: U128, options: &Vec<String>) {
        if let Some(mut election) = self.elections.get(&election_id.into()) {
            if election.vote(options, env::signer_account_id()) {
                self.elections.insert(&election_id.into(), &election);
            }
        }
    }

    pub fn revoke(&mut self, election_id: U128, options: &Vec<String>) {
        if let Some(mut election) = self.elections.get(&election_id.into()) {
            if election.revoke_vote(options, env::signer_account_id()) {
                self.elections.insert(&election_id.into(), &election);
            }
        }
    }

    pub fn get_candidates(&self, election_id: U128) -> Vec<Candidate> {
        if let Some(election) = self.elections.get(&election_id.into()) {
            return election.get_candidates();
        }
        vec![]
    }

    pub fn get_elections(&self, user_id: &AccountId) -> Vec<ElectionInfo> {
        if let Some(ids) = self.user_election.get(user_id) {
            let mut ans = Vec::with_capacity(ids.len() as usize);
            for id in ids.iter() {
                ans.push(ElectionInfo {
                    name: self.elections.get(&id).unwrap().name,
                    id: id.into(),
                });
            }
            return ans;
        }
        vec![]
    }

    pub fn get_last5elections(self) -> Vec<ElectionInfo> {
        let sz = self.election_count.min(5) as usize;
        let mut ans = Vec::with_capacity(sz);
        for id in (0..self.election_count).rev().take(sz) {
            ans.push(ElectionInfo {
                name: self.elections.get(&id).unwrap().name,
                id: id.into(),
            });
        }
        return ans;
    }
}

#[cfg(test)]
mod tests;
