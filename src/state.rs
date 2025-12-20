use std::{collections::HashMap, sync::atomic::AtomicUsize};

use tokio::sync::oneshot;

// type BallotItemId = usize;
// type ElectionId = usize;
pub type ParticipantId = usize;

pub static NEXT_PARTICIPANT_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default)]
pub struct State {
    pub participants: HashMap<ParticipantId, Participant>,
    // elections: HashMap<ElectionId, Election>,
}

#[derive(Debug)]
pub struct Participant {
    pub id: usize,
}

// #[derive(Debug)]
// struct Election {
//     id: ElectionId,
//     participants_who_voted: HashSet<ParticipantId>,
//     ballot_items: HashMap<BallotItemId, BallotItem>,
// }

// #[derive(Debug)]
// struct BallotItem {
//     id: ElectionId,
//     name: String,
//     num_votes: usize,
// }

pub enum Message {
    AddParticipant {
        answer_sender: oneshot::Sender<usize>,
    },
}
