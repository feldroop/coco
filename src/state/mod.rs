use std::collections::{HashMap, HashSet};

use hyper::body::Bytes;
use rand::seq::IndexedRandom;
use tokio::sync::{mpsc, oneshot};
use tracing::error;

use crate::{
    admin::{AdminCreateElectionBody, AdminSession, AdminSessionId},
    election::{BallotItem, Election, ElectionId, ElectionsVoteBody},
    error::{ElectionsVoteError, InvalidCredentialsError},
    participant::{Participant, ParticipantCredentials, ParticipantId, ValidParticipantId},
};

#[derive(Debug, Default)]
pub struct State {
    participants_by_id: HashMap<ParticipantId, Participant>,
    elections_by_id: HashMap<ElectionId, Election>,
    admin_sessions_by_id: HashMap<AdminSessionId, AdminSession>,
}

struct ValidCredentials;

struct ValidAdminSession;

impl State {
    fn check_credentials(
        &self,
        requesting_credentials: &RequestingCredentials,
    ) -> Result<ValidCredentials, InvalidCredentialsError> {
        match requesting_credentials {
            RequestingCredentials::Normal(participant) => self
                .check_participant_validity(participant)
                .map(|_| ValidCredentials),
            RequestingCredentials::Admin(admin_session) => self
                .check_admin_session_validity(admin_session)
                .map(|_| ValidCredentials),
        }
    }

    fn check_participant_validity(
        &self,
        requesting_participant_credentials: &ParticipantCredentials,
    ) -> Result<ValidParticipantId, InvalidCredentialsError> {
        let Some(existing_participant) = self
            .participants_by_id
            .get(&requesting_participant_credentials.id)
        else {
            return Err(InvalidCredentialsError::Missing);
        };

        if existing_participant.credentials.token == requesting_participant_credentials.token {
            Ok(ValidParticipantId(requesting_participant_credentials.id))
        } else {
            Err(InvalidCredentialsError::WrongToken)
        }
    }

    fn check_admin_session_validity(
        &self,
        admin_session: &AdminSession,
    ) -> Result<ValidAdminSession, InvalidCredentialsError> {
        let Some(existing_admin_session) = self.admin_sessions_by_id.get(&admin_session.id) else {
            return Err(InvalidCredentialsError::Missing);
        };

        if existing_admin_session.token == admin_session.token {
            Ok(ValidAdminSession)
        } else {
            Err(InvalidCredentialsError::WrongToken)
        }
    }

    fn create_participant(&mut self) -> ParticipantCredentials {
        let id = self.participants_by_id.len();

        let new_participant = Participant {
            credentials: ParticipantCredentials {
                id,
                token: generate_token(),
            },
            voted_ballot_item_ids_by_election_id: HashMap::new(),
        };

        let new_participant_credentials = new_participant.credentials.clone();
        self.participants_by_id.insert(id, new_participant.clone());

        new_participant_credentials
    }

    fn create_admin_session(&mut self) -> AdminSession {
        let id = self.admin_sessions_by_id.len();
        let new_admin_session = AdminSession {
            id,
            token: generate_token(),
        };
        self.admin_sessions_by_id
            .insert(id, new_admin_session.clone());

        new_admin_session
    }

    fn create_election(
        &mut self,
        _valid_admin_session: ValidAdminSession,
        admin_create_election_body: AdminCreateElectionBody,
    ) {
        let id = self.elections_by_id.len();
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

        self.elections_by_id.insert(id, new_election);
    }

    fn apply_vote(
        &mut self,
        participant_id: ValidParticipantId,
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
            .contains(&participant_id.0)
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

        election.participant_ids_who_voted.insert(participant_id.0);

        Ok(())
    }
}

pub enum Message {
    ParticipantsAdd {
        answer_sender: oneshot::Sender<ParticipantCredentials>,
    },
    ParticipantsGetVotes {
        answer_sender: oneshot::Sender<Result<Bytes, InvalidCredentialsError>>,
        requesting_participant_credentials: ParticipantCredentials,
    },
    ElectionsGet {
        answer_sender: oneshot::Sender<Result<Bytes, InvalidCredentialsError>>,
        requesting_credentials: RequestingCredentials,
    },
    ElectionsVote {
        answer_sender: oneshot::Sender<Result<(), ElectionsVoteError>>,
        requesting_participant_credentials: ParticipantCredentials,
        elections_vote_body: ElectionsVoteBody,
    },
    AdminStartSession {
        answer_sender: oneshot::Sender<AdminSession>,
    },
    AdminCreateElection {
        answer_sender: oneshot::Sender<Result<(), InvalidCredentialsError>>,
        requesting_admin_session: AdminSession,
        admin_create_election_body: AdminCreateElectionBody,
    },
}

pub enum RequestingCredentials {
    Normal(ParticipantCredentials),
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
            Message::ParticipantsAdd { answer_sender } => {
                let new_participant_credentials = state.create_participant();
                answer_sender.send(new_participant_credentials).is_err()
            }
            Message::ParticipantsGetVotes {
                answer_sender,
                requesting_participant_credentials,
            } => {
                let answer = get_votes_of_participant(&state, requesting_participant_credentials);
                answer_sender.send(answer).is_err()
            }
            Message::ElectionsGet {
                answer_sender,
                requesting_credentials,
            } => {
                let answer = get_elections(&state, requesting_credentials);
                answer_sender.send(answer).is_err()
            }
            Message::ElectionsVote {
                answer_sender,
                requesting_participant_credentials,
                elections_vote_body,
            } => {
                let answer = vote(
                    &mut state,
                    requesting_participant_credentials,
                    elections_vote_body,
                );

                answer_sender.send(answer).is_err()
            }
            Message::AdminStartSession { answer_sender } => {
                let new_admin_session = state.create_admin_session();
                answer_sender.send(new_admin_session).is_err()
            }
            Message::AdminCreateElection {
                answer_sender,
                admin_create_election_body,
                requesting_admin_session: admin_session,
            } => {
                let answer =
                    create_election_as_admin(&mut state, admin_create_election_body, admin_session);

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

fn get_votes_of_participant(
    state: &State,
    requesting_participant_credentials: ParticipantCredentials,
) -> Result<Bytes, InvalidCredentialsError> {
    let participant_id = state.check_participant_validity(&requesting_participant_credentials)?;

    let votes = &state.participants_by_id[&participant_id.0].voted_ballot_item_ids_by_election_id;

    match serde_json::to_vec(votes) {
        Ok(serialized) => Ok(Bytes::from_owner(serialized)),
        Err(_) => Err(InvalidCredentialsError::Unexpected),
    }
}

fn get_elections(
    state: &State,
    requesting_credentials: RequestingCredentials,
) -> Result<Bytes, InvalidCredentialsError> {
    // TODO: refactor to use ValidCredentials and make state.elections_by_id inaccessible with it
    if let Err(err) = state.check_credentials(&requesting_credentials) {
        Err(err)
    } else if let Ok(serialized) = serde_json::to_vec(&state.elections_by_id) {
        Ok(Bytes::from_owner(serialized))
    } else {
        error!("Unexpected serialization error.");
        Err(InvalidCredentialsError::Unexpected)
    }
}

fn vote(
    state: &mut State,
    requesting_participant_credentials: ParticipantCredentials,
    elections_vote_body: ElectionsVoteBody,
) -> Result<(), ElectionsVoteError> {
    let participant_id = state.check_participant_validity(&requesting_participant_credentials)?;

    state.apply_vote(participant_id, &elections_vote_body)?;

    if let Some(participant) = state.participants_by_id.get_mut(&participant_id.0) {
        participant.voted_ballot_item_ids_by_election_id.insert(
            elections_vote_body.election_id,
            elections_vote_body.selected_ballot_item_id,
        );
    }

    Ok(())
}

fn create_election_as_admin(
    state: &mut State,
    admin_create_election_body: AdminCreateElectionBody,
    admin_session: AdminSession,
) -> Result<(), InvalidCredentialsError> {
    let valid_admin_session = state.check_admin_session_validity(&admin_session)?;
    state.create_election(valid_admin_session, admin_create_election_body);

    Ok(())
}
