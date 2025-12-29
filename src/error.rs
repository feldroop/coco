use hyper::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum InvalidSessionError {
    #[error("You need to log in first.")]
    Missing,
    #[error("The supplied login token for this participant is wrong.")]
    WrongToken,
    #[error("Unexpected internal error.")]
    Unexpected,
}

impl InvalidSessionError {
    pub fn http_status_code(&self) -> StatusCode {
        match self {
            InvalidSessionError::Missing => StatusCode::UNAUTHORIZED,
            InvalidSessionError::WrongToken => StatusCode::UNAUTHORIZED,
            InvalidSessionError::Unexpected => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ElectionsVoteError {
    #[error("{0}")]
    InvalidParticipant(#[from] InvalidSessionError),
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
