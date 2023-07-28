use super::*;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::{testing_env, VMContext};
use std::convert::TryInto;
use near_sdk::MockedBlockchain;

fn get_context(is_view: bool) -> VMContext {
    VMContextBuilder::new()
        .signer_account_id("bob_near".try_into().unwrap())
        .is_view(is_view)
        .build()
}

#[test]
fn create_election_test_single() {
    let context = get_context(false);
    testing_env!(context);

    let mut contract = Voteer::new();

    let candidates = vec!["head： watch movie", "tail: play basketball", "stand: read a book"]
    .iter().map(|v|v.to_string()).collect::<Vec<_>>();
    let election_name = "to do what by flip a coin".to_string();
    let election_id = contract.create_election(&candidates, false, &election_name);
    let options = contract.get_candidates(election_id);
    assert_eq!(candidates.len(), options.len());

    for (c1, c2) in candidates.iter().zip(options.iter()) {
        assert_eq!(c1, &c2.name);
        assert_eq!(0, c2.supported);
    }

    // vote once
    contract.vote(election_id, &vec![candidates[1].to_string(),]);
    let options = contract.get_candidates(election_id);
    assert_eq!(candidates.len(), options.len());
    for (c1, c2) in candidates.iter().zip(options.iter()) {
        assert_eq!(c1, &c2.name);
        if c1 == &candidates[1].to_string() {
            assert_eq!(1, c2.supported);
        } else {
            assert_eq!(0, c2.supported);
        }
    }

    // repeat vote won't work
    contract.vote(election_id, &vec![candidates[1].to_string(),]);
    let options = contract.get_candidates(election_id);
    assert_eq!(candidates.len(), options.len());
    for (c1, c2) in candidates.iter().zip(options.iter()) {
        assert_eq!(c1, &c2.name);
        if c1 == &candidates[1].to_string()  {
            assert_eq!(1, c2.supported);
        } else {
            assert_eq!(0, c2.supported);
        }
    }
    
    // test user's election
    let elections = contract.get_elections(&"bob_near".to_string());
    assert_eq!(1, elections.len());
    assert_eq!(election_name, elections[0].name);
}