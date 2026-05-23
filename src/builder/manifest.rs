use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(rename = "package")]
    pub pack: Package,
    pub description: String,
    pub app: Option<App>,
    #[serde(rename = "desktop")]
    pub desk: Option<Desktop>,
    pub include: Option<HashMap<String, String>>,
    #[serde(rename = "build_path")]
    pub build_base_dir: String,
}

impl Manifest {
    pub fn example_json() -> String {
        json!({
            "package": {
                "name": "Example Package",
                "package": "example-package",
                "version": "1.0.0",
                "authors": ["Example Author <author@example.com>"]
            },
            "description": "A sample lpack manifest for packaging applications.",
            "app": {
                "binary": "example-binary",
                "entry": "example-app"
            },
            "desktop": {
                "name": "Example App",
                "icon": "icon.png",
                "exec": "example-app"
            },
            "include": {
                "README.md": "README.md"
            },
            "build_path": "build"
        })
        .to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,

    #[serde(rename = "package")]
    pub package_name: String,

    pub version: String,
    pub authors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub binary: String,
    pub entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desktop {
    pub name: String,
    pub icon: String,

    #[serde(rename = "exec")]
    pub exec_cmd: String,
}
