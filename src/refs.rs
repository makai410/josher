use std::{fmt::Display, ops::Deref};

pub struct Refspec {
    val: String,
}

impl Refspec {
    pub fn new(run_id: &str, tag: &str) -> Self {
        Self {
            val: format!("refs/josher/{run_id}/{tag}"),
        }
    }
}

impl From<String> for Refspec {
    fn from(value: String) -> Self {
        Self { val: value }
    }
}
impl Deref for Refspec {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl Display for Refspec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.val)
    }
}

pub enum RefTag {
    DestinationBase,
    SourceHead,
}

impl RefTag {
    pub const fn as_str(&self) -> &'static str {
        match self {
            RefTag::DestinationBase => "destination-base",
            RefTag::SourceHead => "source-head",
        }
    }
}
