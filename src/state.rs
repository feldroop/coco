use std::collections::{HashMap, HashSet};

use hyper::body::Bytes;
use rand::seq::IndexedRandom;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::{
    admin::{AdminCreateElectionBody, AdminSession, AdminSessionId},
    election::{BallotItem, Election, ElectionId, ElectionsVoteBody},
    error::{ElectionsVoteError, InvalidSessionError},
    participant::{Participant, ParticipantId},
};

#[derive(Debug, Default)]
pub struct State {
    participants_by_id: HashMap<ParticipantId, Participant>,
    elections_by_id: HashMap<ElectionId, Election>,
    admin_sessions_by_id: HashMap<AdminSessionId, AdminSession>,
}

impl State {
    // TODO: parse, not validate
    fn check_participant_validity(
        &self,
        participant: &Participant,
    ) -> Result<(), InvalidSessionError> {
        let Some(existing_participant) = self.participants_by_id.get(&participant.id) else {
            return Err(InvalidSessionError::Missing);
        };

        if existing_participant.token == participant.token {
            Ok(())
        } else {
            Err(InvalidSessionError::WrongToken)
        }
    }

    fn apply_vote(
        &mut self,
        voting_participant: &Participant,
        elections_vote_body: &ElectionsVoteBody,
    ) -> Result<(), ElectionsVoteError> {
        let Some(election) = self
            .elections_by_id
            .get_mut(&elections_vote_body.election_id)
        else {
            return Err(ElectionsVoteError::MissingElection);
        };

        if election
            .participant_ids_who_voted
            .contains(&voting_participant.id)
        {
            return Err(ElectionsVoteError::AlreadyVoted);
        }

        let Some(ballot_item) = election
            .ballot_items_by_id
            .get_mut(&elections_vote_body.selected_ballot_item_id)
        else {
            return Err(ElectionsVoteError::MissingBallotItem);
        };

        ballot_item.num_votes += 1;

        election
            .participant_ids_who_voted
            .insert(voting_participant.id);

        Ok(())
    }

    // TODO: parse, not validate
    fn check_admin_session_validity(
        &self,
        admin_session: &AdminSession,
    ) -> Result<(), InvalidSessionError> {
        let Some(existing_admin_session) = self.admin_sessions_by_id.get(&admin_session.id) else {
            return Err(InvalidSessionError::Missing);
        };

        if existing_admin_session.token == admin_session.token {
            Ok(())
        } else {
            Err(InvalidSessionError::WrongToken)
        }
    }

    // TODO: parse, not validate
    fn check_credentials(
        &self,
        requesting_credentials: &RequestingCredentials,
    ) -> Result<(), InvalidSessionError> {
        match requesting_credentials {
            RequestingCredentials::Normal(participant) => {
                self.check_participant_validity(participant)
            }
            RequestingCredentials::Admin(admin_session) => {
                self.check_admin_session_validity(admin_session)
            }
        }
    }
}

pub enum Message {
    ParticipantsGet {
        answer_sender: oneshot::Sender<Participant>,
    },
    ElectionsGet {
        answer_sender: oneshot::Sender<Result<Bytes, InvalidSessionError>>,
        requesting_credentials: RequestingCredentials,
    },
    ElectionsVote {
        answer_sender: oneshot::Sender<Result<(), ElectionsVoteError>>,
        requesting_participant: Participant,
        elections_vote_body: ElectionsVoteBody,
    },
    AdminStartSession {
        answer_sender: oneshot::Sender<AdminSession>,
    },
    AdminCreateElection {
        answer_sender: oneshot::Sender<Result<(), InvalidSessionError>>,
        requesting_admin_session: AdminSession,
        admin_create_election_body: AdminCreateElectionBody,
    },
}

pub enum RequestingCredentials {
    Normal(Participant),
    Admin(AdminSession),
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

    state.elections_by_id.insert(
        1,
        Election {
            id: 1,
            name: String::from("What is your favorite color?"),
            ballot_items_by_id: HashMap::from([
                (
                    0,
                    BallotItem {
                        id: 0,
                        name: String::from("Red"),
                        num_votes: 0,
                    },
                ),
                (
                    1,
                    BallotItem {
                        id: 1,
                        name: String::from("Green"),
                        num_votes: 0,
                    },
                ),
                (
                    2,
                    BallotItem {
                        id: 2,
                        name: String::from("Blue"),
                        num_votes: 0,
                    },
                ),
            ]),
            participant_ids_who_voted: HashSet::new(),
        },
    );

    while let Some(message) = message_receiver.recv().await {
        let answer_send_is_err = match message {
            Message::ParticipantsGet { answer_sender } => {
                let id = state.participants_by_id.len();
                let new_participant = Participant {
                    id,
                    token: generate_token(),
                };
                state.participants_by_id.insert(id, new_participant.clone());

                answer_sender.send(new_participant).is_err()
            }
            Message::ElectionsGet {
                answer_sender,
                requesting_credentials,
            } => {
                let answer = if let Err(err) = state.check_credentials(&requesting_credentials) {
                    Err(err)
                } else if let Ok(serialized) = serde_json::to_vec(&state.elections_by_id) {
                    Ok(Bytes::from_owner(serialized))
                } else {
                    error!("Unexpected serialization error.");
                    Err(InvalidSessionError::Unexpected)
                };

                answer_sender.send(answer).is_err()
            }
            Message::ElectionsVote {
                answer_sender,
                requesting_participant,
                elections_vote_body,
            } => {
                let answer =
                    if let Err(err) = state.check_participant_validity(&requesting_participant) {
                        Err(err.into())
                    } else if let Err(err) =
                        state.apply_vote(&requesting_participant, &elections_vote_body)
                    {
                        Err(err)
                    } else {
                        Ok(())
                    };

                answer_sender.send(answer).is_err()
            }
            Message::AdminStartSession { answer_sender } => {
                let id = state.admin_sessions_by_id.len();
                let new_admin_session = AdminSession {
                    id,
                    token: generate_token(),
                };
                state
                    .admin_sessions_by_id
                    .insert(id, new_admin_session.clone());

                answer_sender.send(new_admin_session).is_err()
            }
            Message::AdminCreateElection {
                answer_sender,
                admin_create_election_body,
                requesting_admin_session: admin_session,
            } => {
                let answer = if let Err(err) = state.check_admin_session_validity(&admin_session) {
                    Err(err)
                } else {
                    let id = state.elections_by_id.len();
                    let name = admin_create_election_body.name;
                    let ballot_items_by_id = admin_create_election_body
                        .ballot_items
                        .into_iter()
                        .enumerate()
                        .map(|(id, name)| {
                            (
                                id,
                                BallotItem {
                                    id,
                                    name,
                                    num_votes: 0,
                                },
                            )
                        })
                        .collect();

                    let new_election = Election {
                        id,
                        name,
                        ballot_items_by_id,
                        participant_ids_who_voted: HashSet::new(),
                    };

                    state.elections_by_id.insert(id, new_election);

                    Ok(())
                };

                answer_sender.send(answer).is_err()
            }
        };

        if answer_send_is_err {
            error!("Unexpected send answer error.");
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
