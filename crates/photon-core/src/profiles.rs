use crate::ProfileId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProfileKind {
    Normal,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub id: ProfileId,
    pub name: String,
    pub kind: ProfileKind,
}

impl Profile {
    pub fn new(name: impl Into<String>, kind: ProfileKind) -> Self {
        Self {
            id: ProfileId::next(),
            name: name.into(),
            kind,
        }
    }

    pub fn is_private(&self) -> bool {
        self.kind == ProfileKind::Private
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_profile_flag() {
        assert!(!Profile::new("a", ProfileKind::Normal).is_private());
        assert!(Profile::new("b", ProfileKind::Private).is_private());
    }
}
