use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPackage {
    pub name: String,
    #[serde(rename = "package")]
    pub package_name: String,
    pub version: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallApp {
    pub entry: String,
    pub executable: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallDesktop {
    pub name: String,
    pub icon: String,
    #[serde(rename = "exec")]
    pub exec_cmd: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallManifest {
    pub info: InstallPackage,
    pub app: Option<InstallApp>,
    #[serde(rename = "desktop")]
    pub desk: Option<InstallDesktop>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VersionPart {
    Number(i32),
    Text(String),
}
