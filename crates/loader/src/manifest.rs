#![allow(dead_code, unused_mut, unused_variables)]

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AuthProviders<'a> {
    #[serde(borrow)]
    auth: Vec<&'a str>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionMod<'a> {
    #[serde(flatten, borrow)]
    info: ModInfo<'a>,
    #[serde(borrow)]
    providers: Option<AuthProviders<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ModInfo<'a> {
    key: &'a str,
    handler: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MacroMod<'a> {
    #[serde(flatten, borrow)]
    info: ModInfo<'a>,
    title: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Webtrigger<'a>(#[serde(borrow)] ModInfo<'a>);

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgeModules<'a> {
    #[serde(rename = "macro", default, borrow)]
    macros: Vec<MacroMod<'a>>,
    #[serde(rename = "function", default, borrow)]
    pub functions: Vec<FunctionMod<'a>>,
    #[serde(rename = "webtrigger", default, borrow)]
    webtriggers: Vec<Webtrigger<'a>>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Content<'a> {
    #[serde(default, borrow)]
    scripts: Vec<&'a str>,
    #[serde(default, borrow)]
    styles: Vec<&'a str>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Perms<'a> {
    #[serde(default, borrow)]
    scopes: Vec<&'a str>,
    #[serde(default, borrow)]
    content: Content<'a>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct AppInfo<'a> {
    name: Option<&'a str>,
    id: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ForgeManifest<'a> {
    #[serde(borrow)]
    app: AppInfo<'a>,
    #[serde(borrow)]
    pub(crate) modules: ForgeModules<'a>,
    #[serde(borrow)]
    permissions: Perms<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Resolved;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Unresolved;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct FunctionRef<'a, S = Unresolved> {
    func: &'a str,
    key: &'a str,
    path: PathBuf,
    status: S,
}

impl<S> FunctionRef<'_, S> {
    const VALID_EXTS: [&str; 4] = [".js", ".jsx", ".ts", ".tsx"];

    #[inline]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[inline]
    pub fn func(&self) -> &str {
        self.func
    }
}

impl<'a> FunctionRef<'a> {
    #[inline]
    fn new(func: &'a str, key: &'a str, path: PathBuf) -> Self {
        Self {
            func,
            key,
            path,
            status: Unresolved,
        }
    }

    fn try_resolve(self, paths: HashSet<&Path>) -> Result<FunctionRef<'a, Resolved>, &'static str> {
        Self::VALID_EXTS
            .iter()
            .find_map(|&ext| {
                let path = self.path.with_extension(ext);
                paths.contains(&*path).then_some(FunctionRef {
                    func: self.func,
                    key: self.key,
                    path,
                    status: Resolved,
                })
            })
            .ok_or("No valid file found")
    }
}

impl<'a> TryFrom<FunctionMod<'a>> for FunctionRef<'a> {
    type Error = String;

    fn try_from(func_handler: FunctionMod<'a>) -> Result<Self, Self::Error> {
        let handler_info = func_handler.info;
        let (file, func) = handler_info
            .handler
            .splitn(2, '.')
            .collect_tuple()
            .ok_or_else(|| format!("handler {} does not contain a period", handler_info.key))?;
        let mut path = PathBuf::from("src");
        path.push(file);
        Ok(Self {
            func,
            key: handler_info.key,
            path,
            status: Unresolved,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "app": {
                "name": "My App",
                "id": "my-app"
            },
            "modules": {
                "macro": [
                {
                    "key": "my-macro",
                    "handler": "my-macro-handler",
                    "title": "My Macro"
                }
                ],
                "function": [
                {
                    "key": "my-function",
                    "handler": "my-function-handler",
                    "providers": {
                        "auth": ["my-auth-provider"]
                    }
                }
                ],
                "webtrigger": [
                {
                    "key": "my-webtrigger",
                    "handler": "my-webtrigger-handler"
                }
                ]
            },
            "permissions": {
                "scopes": [
                    "my-scope"
                ],
                "content": {
                    "scripts": [
                        "my-script.js"
                    ],
                    "styles": [
                        "my-style.css"
                    ]
                }
            }
        }"#;
        let manifest: ForgeManifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.app.name, Some("My App"));
        assert_eq!(manifest.app.id, "my-app");
        assert_eq!(manifest.modules.macros.len(), 1);
        assert_eq!(manifest.modules.macros[0].title, "My Macro");
        assert_eq!(manifest.modules.macros[0].info.key, "my-macro");
        assert_eq!(manifest.modules.macros[0].info.handler, "my-macro-handler");
        assert_eq!(manifest.modules.functions.len(), 1);
        assert_eq!(
            manifest.modules.functions[0],
            FunctionMod {
                info: ModInfo {
                    key: "my-function",
                    handler: "my-function-handler",
                },
                providers: Some(AuthProviders {
                    auth: vec!["my-auth-provider"]
                }),
            }
        );
    }

    #[test]
    fn test_function_handler_parsing() {
        let func_handler = FunctionMod {
            info: ModInfo {
                key: "my-function",
                handler: "my-function-handler",
            },
            providers: Some(AuthProviders {
                auth: vec!["my-auth-provider"],
            }),
        };
        let func_ref: FunctionRef = func_handler.try_into().unwrap();
        assert_eq!(
            func_ref,
            FunctionRef {
                func: "app",
                key: "my-function",
                path: "src/my-function-handler".into(),
                status: Unresolved,
            }
        );
    }
}
