use std::{fmt::Display, ops::Deref};

pub struct Refspec {
    val: String,
}

impl Refspec {
    pub fn new(run_id: String, tag: String) -> Self {
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
