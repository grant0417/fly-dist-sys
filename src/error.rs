use crate::proto::MessageBody;

macro_rules! error_kind {
    ($name:ident, $kind:expr, $is_kind_fn:ident) => {
        pub fn $name() -> Self {
            Self::new($kind, $kind.to_string())
        }

        pub fn $is_kind_fn(&self) -> bool {
            self.kind == $kind
        }
    };
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub text: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind_str = self.kind.to_string();
        if kind_str == self.text {
            write!(f, "{}", kind_str)
        } else {
            write!(f, "{}: {}", kind_str, self.text)
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(kind: ErrorKind, text: impl Into<String>) -> Self {
        Self {
            kind,
            text: text.into(),
        }
    }

    error_kind!(timeout, ErrorKind::Timeout, is_timeout);
    error_kind!(node_not_found, ErrorKind::NodeNotFound, is_node_not_found);
    error_kind!(not_supported, ErrorKind::NotSupported, is_not_supported);
    error_kind!(
        temporarily_unavailable,
        ErrorKind::TemporarlilyUnavailable,
        is_temporarily_unavailable
    );
    error_kind!(
        malformed_request,
        ErrorKind::MalformedRequest,
        is_malformed_request
    );
    error_kind!(crash, ErrorKind::Crash, is_crash);
    error_kind!(abort, ErrorKind::Abort, is_abort);
    error_kind!(
        key_does_not_exist,
        ErrorKind::KeyDoesNotExist,
        is_key_does_not_exist
    );
    error_kind!(
        key_already_exists,
        ErrorKind::KeyAlreadyExists,
        is_key_already_exists
    );
    error_kind!(
        precondition_failed,
        ErrorKind::PreconditionFailed,
        is_precondition_failed
    );
    error_kind!(txn_conflict, ErrorKind::TxnConflict, is_txn_conflict);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorKind {
    /// Indicates that the requested operation could not be completed within a timeout.
    Timeout = 0,
    /// Thrown when a client sends an RPC request to a node which does not exist.
    NodeNotFound = 1,
    /// Use this error to indicate that a requested operation is not supported by the current implementation. Helpful for stubbing out APIs during development.
    NotSupported = 10,
    /// Indicates that the operation definitely cannot be performed at this time--perhaps because the server is in a read-only state, has not yet been initialized, believes its peers to be down, and so on. Do not use this error for indeterminate cases, when the operation may actually have taken place.
    TemporarlilyUnavailable = 11,
    /// The client's request did not conform to the server's expectations, and could not possibly have been processed.
    MalformedRequest = 12,
    /// Indicates that some kind of general, indefinite error occurred. Use this as a catch-all for errors you can't otherwise categorize, or as a starting point for your error handler: it's safe to return internal-error for every problem by default, then add special cases for more specific errors later.
    Crash = 13,
    /// Indicates that some kind of general, definite error occurred. Use this as a catch-all for errors you can't otherwise categorize, when you specifically know that the requested operation has not taken place. For instance, you might encounter an indefinite failure during the prepare phase of a transaction: since you haven't started the commit process yet, the transaction can't have taken place. It's therefore safe to return a definite abort to the client.
    Abort = 14,
    /// The client requested an operation on a key which does not exist (assuming the operation should not automatically create missing keys).
    KeyDoesNotExist = 20,
    /// The client requested the creation of a key which already exists, and the server will not overwrite it.
    KeyAlreadyExists = 21,
    /// The requested operation expected some conditions to hold, and those conditions were not met. For instance, a compare-and-set operation might assert that the value of a key is currently 5; if the value is 3, the server would return precondition-failed.
    PreconditionFailed = 22,
    /// The requested transaction has been aborted because of a conflict with another transaction. Servers need not return this error on every conflict: they may choose to retry automatically instead.
    TxnConflict = 30,
}

impl ErrorKind {
    pub fn from_u8(kind: u8) -> Option<Self> {
        match kind {
            0 => Some(Self::Timeout),
            1 => Some(Self::NodeNotFound),
            10 => Some(Self::NotSupported),
            11 => Some(Self::TemporarlilyUnavailable),
            12 => Some(Self::MalformedRequest),
            13 => Some(Self::Crash),
            14 => Some(Self::Abort),
            20 => Some(Self::KeyDoesNotExist),
            21 => Some(Self::KeyAlreadyExists),
            22 => Some(Self::PreconditionFailed),
            30 => Some(Self::TxnConflict),
            _ => None,
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Timeout => write!(f, "timeout"),
            ErrorKind::NodeNotFound => write!(f, "node not found"),
            ErrorKind::NotSupported => write!(f, "not supported"),
            ErrorKind::TemporarlilyUnavailable => write!(f, "temporarily unavailable"),
            ErrorKind::MalformedRequest => write!(f, "malformed request"),
            ErrorKind::Crash => write!(f, "crash"),
            ErrorKind::Abort => write!(f, "abort"),
            ErrorKind::KeyDoesNotExist => write!(f, "key does not exist"),
            ErrorKind::KeyAlreadyExists => write!(f, "key already exists"),
            ErrorKind::PreconditionFailed => write!(f, "precondition failed"),
            ErrorKind::TxnConflict => write!(f, "transaction conflict"),
        }
    }
}

impl From<Error> for MessageBody {
    fn from(err: Error) -> Self {
        MessageBody::new("error")
            .with_field("kind", err.kind as u8)
            .with_field("text", err.text)
    }
}

impl From<MessageBody> for Error {
    fn from(body: MessageBody) -> Self {
        Self {
            kind: ErrorKind::from_u8(body.extra.get("code").unwrap().as_u64().unwrap() as u8)
                .unwrap(),
            text: body.extra.get("text").unwrap().as_str().unwrap().into(),
        }
    }
}
