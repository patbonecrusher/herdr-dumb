use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TestEndpointId(String);

impl TestEndpointId {
    pub(crate) fn parse(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.is_empty() {
            return Err("empty test identity".into());
        }
        Ok(Self(value))
    }
}

impl fmt::Display for TestEndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone)]
pub(crate) struct TestEndpoint {
    pub(crate) id: TestEndpointId,
    pub(crate) label: String,
    pub(crate) enabled: bool,
}
