use chrono::{DateTime, Local};
use uuid::Uuid;

#[derive(PartialEq, Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub enum ProjectType {
    #[default]
    Json,
    Binary,
}

impl ProjectType {
    pub fn extension(&self) -> &'static str {
        match self {
            ProjectType::Json => "json",
            ProjectType::Binary => "mbp",
        }
    }
}

pub trait ToFileName {
    type Out;
    fn to_project_file_name(&self) -> Self::Out;
}

impl ToFileName for &str {
    type Out = String;
    fn to_project_file_name(&self) -> Self::Out {
        self.to_string().to_project_file_name()
    }
}

impl ToFileName for String {
    type Out = String;

    fn to_project_file_name(&self) -> Self::Out {
        let mut res = String::new();
        for c in self.chars() {
            if c == ' ' {
                res.push('_');
            } else if c.is_ascii_alphanumeric() {
                res.push(c.to_ascii_lowercase());
            }
        }

        let res: Vec<_> = res
            .chars()
            .fold((String::new(), None), |(mut acc, prev), c| {
                if !(prev == Some('_') && c == '_') {
                    acc.push(c);
                }
                (acc, Some(c))
            })
            .0
            .chars()
            .collect();

        match res.iter().as_slice() {
            ['_', rest @ .., '_'] => rest.iter().collect(),
            ['_', rest @ ..] => rest.iter().collect(),
            [rest @ .., '_'] => rest.iter().collect(),
            rest => rest.iter().collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub id: Uuid,
    pub last_saved: DateTime<Local>,
    pub created_at: DateTime<Local>,
    #[serde(skip)]
    pub file_name: String,
    #[serde(skip)]
    pub project_type: ProjectType,
}
