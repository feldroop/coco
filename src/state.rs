use std::collections::{HashMap, HashSet};

use hyper::body::Bytes;
use rand::seq::IndexedRandom;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::{
    election::{BallotItem, Election, ElectionId, ElectionsVoteBody},
    participant::{Participant, ParticipantId},
};

#[derive(Debug, Default)]
pub struct State {
    participants_by_id: HashMap<ParticipantId, Participant>,
    elections_by_id: HashMap<ElectionId, Election>,
}

impl State {
    fn participant_is_valid(&self, participant: &Participant) -> bool {
        let Some(existing_participant) = self.participants_by_id.get(&participant.id) else {
            return false;
        };

        existing_participant.token == participant.token
    }

    // TODO: better return type.
    fn apply_vote(
        &mut self,
        voting_participant: &Participant,
        elections_vote_body: &ElectionsVoteBody,
    ) -> bool {
        let Some(election) = self
            .elections_by_id
            .get_mut(&elections_vote_body.election_id)
        else {
            return false;
        };

        if election
            .participant_ids_who_voted
            .contains(&voting_participant.id)
        {
            return false;
        }

        let Some(ballot_item) = election
            .ballot_items_by_id
            .get_mut(&elections_vote_body.selected_ballot_item_id)
        else {
            return false;
        };

        ballot_item.num_votes += 1;

        election
            .participant_ids_who_voted
            .insert(voting_participant.id);

        true
    }
}

pub enum Message {
    AddParticipant {
        answer_sender: oneshot::Sender<Participant>,
    },
    GetElections {
        answer_sender: oneshot::Sender<Option<Bytes>>,
        requesting_participant: Participant,
    },
    ElectionsVote {
        answer_sender: oneshot::Sender<bool>,
        requesting_participant: Participant,
        elections_vote_body: ElectionsVoteBody,
    },
}

pub async fn central_state_authority(mut message_receiver: mpsc::Receiver<Message>) {
    let mut state = State::default();

    state.elections_by_id.insert(
        0,
        Election {
            id: 0,
            name: String::from("What is your favorite pet?"),
            ballot_items_by_id: HashMap::from([
                (
                    0,
                    BallotItem {
                        id: 0,
                        name: String::from("Cat"),
                        num_votes: 0,
                    },
                ),
                (
                    1,
                    BallotItem {
                        id: 1,
                        name: String::from("Dog"),
                        num_votes: 0,
                    },
                ),
            ]),
            participant_ids_who_voted: HashSet::new(),
        },
    );

    while let Some(message) = message_receiver.recv().await {
        match message {
            Message::AddParticipant { answer_sender } => {
                let id = state.participants_by_id.len();
                let new_participant = Participant {
                    id,
                    token: generate_token(),
                };
                state.participants_by_id.insert(id, new_participant.clone());

                if answer_sender.send(new_participant).is_err() {
                    error!("Unexpected AddParticipant send answer error.");
                }
            }
            Message::GetElections {
                answer_sender,
                requesting_participant,
            } => {
                let answer = if state.participant_is_valid(&requesting_participant) {
                    let Ok(serialized) = serde_json::to_vec(&state.elections_by_id) else {
                        error!("Unexpected serialization error.");
                        continue;
                    };

                    Some(Bytes::from_owner(serialized))
                } else {
                    None
                };

                if answer_sender.send(answer).is_err() {
                    error!("Unexpected GetElections send answer error.");
                }
            }
            Message::ElectionsVote {
                answer_sender,
                requesting_participant,
                elections_vote_body,
            } => {
                if answer_sender
                    .send(
                        state.participant_is_valid(&requesting_participant)
                            && state.apply_vote(&requesting_participant, &elections_vote_body),
                    )
                    .is_err()
                {
                    error!("Unexpected ElectionsVote send answer error.");
                }
            }
        }
    }
}

fn generate_token() -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const TOKEN_LENGTH: usize = 32;
    let mut rng = rand::rng();
    let token_seq: Vec<_> = (0..TOKEN_LENGTH)
        .map(|_| *CHARS.choose(&mut rng).unwrap())
        .collect();
    String::try_from(token_seq).unwrap()
}
