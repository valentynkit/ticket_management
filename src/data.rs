pub mod ticket_data {
    use serde::{Deserialize, Serialize};

    pub use super::data_fields::{Description, Status, TicketId, Title};

    #[derive(Deserialize, Default)]
    pub struct TicketPatch {
        pub(crate) title: Option<Title>,
        pub(crate) description: Option<Description>,
        pub(crate) status: Option<Status>,
    }

    #[derive(Deserialize)]
    pub struct TicketDraft {
        title: Title,
        description: Description,
    }
    impl TicketDraft {
        pub fn new(title: Title, description: Description) -> Self {
            Self { title, description }
        }
    }

    #[derive(Serialize, Clone)]
    pub(crate) struct Ticket {
        id: TicketId,
        pub title: Title,
        pub description: Description,
        pub status: Status,
    }
    impl Ticket {
        pub fn new(id: TicketId, draft: TicketDraft) -> Self {
            let TicketDraft { title, description } = draft;
            Self {
                id,
                title,
                description,
                status: Status::ToDo,
            }
        }
    }
}

pub mod data_fields {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Clone, Debug, Copy, PartialEq, Eq, Deserialize, Serialize)]
    #[serde(rename_all = "snake_case")]
    pub enum Status {
        ToDo,
        InProgress,
        Done,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
    pub struct TicketId(u64);
    impl TicketId {
        pub fn inner(&self) -> u64 {
            self.0
        }
    }

    impl From<u64> for TicketId {
        fn from(value: u64) -> Self {
            Self(value)
        }
    }

    impl Display for TicketId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Debug, PartialEq, Clone, Eq, Deserialize, Serialize)]
    #[serde(try_from = "String")]
    pub struct Description(String);

    impl Display for Description {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    const fn validate_description(str: &str) -> Result<(), DescriptionError> {
        match str.len() {
            0 => Err(DescriptionError::Empty),
            x if x < 5 => Err(DescriptionError::TooShort(x)),
            x if x > 200 => Err(DescriptionError::TooLong(x)),
            _ => Ok(()),
        }
    }

    // TODO: could we use any blanket implementation, or cover implementation?
    impl TryFrom<&str> for Description {
        type Error = DescriptionError;

        fn try_from(value: &str) -> Result<Self, DescriptionError> {
            value.to_owned().try_into()
        }
    }
    impl TryFrom<String> for Description {
        type Error = DescriptionError;

        fn try_from(value: String) -> Result<Self, DescriptionError> {
            validate_description(&value)?;
            Ok(Self(value))
        }
    }
    #[derive(Error, Debug)]
    pub enum DescriptionError {
        #[error("The description couldn't be empty")]
        Empty,
        #[error("The description couldn't be shorter than 5 characters, actual lenght: `{0}`")]
        TooShort(usize),
        #[error("The description couldn't be longer than 200 characters, actual lenght: `{0}`")]
        TooLong(usize),
    }

    #[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Eq)]
    #[serde(try_from = "String")]
    pub struct Title(String);

    const fn validate_title(str: &str) -> Result<(), TitleError> {
        match str.len() {
            0 => Err(TitleError::Empty),
            x if x < 3 => Err(TitleError::TooShort(x)),
            x if x > 20 => Err(TitleError::TooLong(x)),
            _ => Ok(()),
        }
    }

    impl TryFrom<&str> for Title {
        type Error = TitleError;

        fn try_from(value: &str) -> Result<Self, TitleError> {
            value.to_owned().try_into()
        }
    }
    impl TryFrom<String> for Title {
        type Error = TitleError;

        fn try_from(value: String) -> Result<Self, TitleError> {
            validate_title(&value)?;
            Ok(Self(value))
        }
    }

    #[derive(Error, Debug)]
    pub enum TitleError {
        #[error("The title couldn't be empty")]
        Empty,
        #[error("The title couldn't be shorter than 3 characters, actual lenght: `{0}`")]
        TooShort(usize),
        #[error("The description couldn't be longer than 20 characters, actual lenght: `{0}`")]
        TooLong(usize),
    }

    impl Display for Title {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
}
