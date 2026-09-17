mod activation;
mod control;
mod health;
mod message_policy;
mod registry;
mod supervisor;
#[cfg(test)]
mod test_support;
mod writer;

pub(crate) use activation::*;
pub(crate) use control::*;
pub(crate) use message_policy::*;
pub(crate) use registry::*;
pub(crate) use supervisor::*;
#[cfg(test)]
pub(crate) use test_support::{TestEndpoint, TestEndpointId};
pub(crate) use writer::NativeEndpointTransport;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ClientEndpointId {
    Local,
    // Synthetic identities exercise stale input and snapshot isolation without networking.
    #[cfg(test)]
    Test(TestEndpointId),
}

impl ClientEndpointId {
    pub(crate) fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }

    pub(crate) fn storage_key(&self) -> String {
        match self {
            Self::Local => "local".into(),
            #[cfg(test)]
            Self::Test(id) => format!("test:{id}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClientEndpointStatus {
    Connecting,
    Online,
    Reconnecting,
    Attention,
    #[cfg(test)]
    Disabled,
}
