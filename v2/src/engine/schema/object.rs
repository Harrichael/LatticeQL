//! Source-agnostic schema model: an `ObjectSchema` represents one "thing
//! with named attributes" — a SQL table, a GraphQL object type, a REST
//! resource — described uniformly regardless of where the data lives.
//!
//! ```ignore
//! let users = ObjectSchema::new("public", "users")
//!     .with_attribute(Attribute::new("id", Type::new("BIGINT", Kind::Integer)))
//!     .with_attribute(Attribute::new("email", Type::new("VARCHAR(255)", Kind::Text)));
//!
//! // Same object, two roles in one query.
//! let author = users.clone().with_alias("author");
//! let reviewer = users.with_alias("reviewer");
//! ```

// Two views on an attribute's type travel together: `source` is the raw
// label from the originating system, preserved verbatim so it can be shown
// to the user; `kind` is our normalized category that general logic
// (compatibility checks, pathfinding) can reason over without knowing the
// source vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub source: String,
    pub kind: Kind,
}

impl Type {
    pub fn new(source: impl Into<String>, kind: Kind) -> Self {
        Self { source: source.into(), kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Integer,
    Text,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Attribute {
    pub name: String,
    pub ty: Type,
}

impl Attribute {
    pub fn new(name: impl Into<String>, ty: Type) -> Self {
        Self { name: name.into(), ty }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectSchema {
    pub namespace: String,
    pub name: String,
    // Distinguishes multiple appearances of the same object in one path
    // (e.g. `users AS author` vs `users AS reviewer`).
    pub instance_alias: Option<String>,
    pub attributes: Vec<Attribute>,
}

impl ObjectSchema {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
            instance_alias: None,
            attributes: Vec::new(),
        }
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.instance_alias = Some(alias.into());
        self
    }

    pub fn with_attribute(mut self, attr: Attribute) -> Self {
        self.attributes.push(attr);
        self
    }

    pub fn handle(&self) -> &str {
        self.instance_alias.as_deref().unwrap_or(&self.name)
    }

    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name == name)
    }
}
