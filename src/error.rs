use hyper::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum InvalidParticipantError {
    #[error("You need to log in first.")]
    Missing,
    #[error("The supplied login token for this participant is wrong.")]
    WrongToken,
    #[error("Unexpected internal error.")]
    Unexpected,
}

impl InvalidParticipantError {
    pub fn http_status_code(&self) -> StatusCode {
        match self {
            InvalidParticipantError::Missing => StatusCode::UNAUTHORIZED,
            InvalidParticipantError::WrongToken => StatusCode::UNAUTHORIZED,
            InvalidParticipantError::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ElectionsVoteError {
    #[error("{0}")]
    InvalidParticipant(#[from] InvalidParticipantError),
    #[error("The election was deleted by the administrator.")]
    MissingElection,
    #[error("You already voted.")]
    AlreadyVoted,
    #[error("The election was modified by the administrator.")]
    MissingBallotItem,
}

impl ElectionsVoteError {
    pub fn http_status_code(&self) -> StatusCode {
        match self {
            ElectionsVoteError::InvalidParticipant(invalid_participant_error) => {
                invalid_participant_error.http_status_code()
            }
            ElectionsVoteError::MissingElection => StatusCode::NOT_FOUND,
            ElectionsVoteError::AlreadyVoted => StatusCode::FORBIDDEN,
            ElectionsVoteError::MissingBallotItem => StatusCode::NOT_FOUND,
        }
    }
}
