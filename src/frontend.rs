use std::{collections::HashMap, sync::LazyLock};

const INCLUDE_SOURCEMAPS_AND_TS: bool = true;

pub static FRONTEND_FILES: LazyLock<HashMap<&'static str, &'static FrontEndFile>> =
    LazyLock::new(|| {
        FILE_DATA
            .iter()
            .filter_map(|file_data| {
                if !INCLUDE_SOURCEMAPS_AND_TS
                    && (file_data.kind == FileKind::JsMap || file_data.kind == FileKind::Ts)
                {
                    None
                } else {
                    Some((file_data.path, file_data))
                }
            })
            .collect()
    });

pub struct FrontEndFile {
    pub kind: FileKind,
    pub name: &'static str,
    pub path: &'static str,
    pub content: &'static [u8],
}

const FILE_DATA: &[FrontEndFile] = &[
    FrontEndFile {
        kind: FileKind::Html,
        name: "index.html",
        path: "/",
        content: include_bytes!("../frontend/index.html"),
    },
    FrontEndFile {
        kind: FileKind::Js,
        name: "index.js",
        path: "/index.js",
        content: include_bytes!("../frontend/out/index.js"),
    },
    FrontEndFile {
        kind: FileKind::Ts,
        name: "index.ts",
        path: "/index.ts",
        content: include_bytes!("../frontend/index.ts"),
    },
    FrontEndFile {
        kind: FileKind::JsMap,
        name: "index.js.map",
        path: "/index.js.map",
        content: include_bytes!("../frontend/out/index.js.map"),
    },
    FrontEndFile {
        kind: FileKind::Html,
        name: "login.html",
        path: "/login",
        content: include_bytes!("../frontend/login.html"),
    },
    FrontEndFile {
        kind: FileKind::Js,
        name: "login.js",
        path: "/login.js",
        content: include_bytes!("../frontend/out/login.js"),
    },
    FrontEndFile {
        kind: FileKind::Ts,
        name: "login.ts",
        path: "/login.ts",
        content: include_bytes!("../frontend/login.ts"),
    },
    FrontEndFile {
        kind: FileKind::JsMap,
        name: "login.js.map",
        path: "/login.js.map",
        content: include_bytes!("../frontend/out/login.js.map"),
    },
    FrontEndFile {
        kind: FileKind::Ico,
        name: "favicon.ico",
        path: "/favicon.ico",
        content: include_bytes!("../frontend/assets/favicon.ico"),
    },
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Html,
    Css,
    Js,
    Ts,
    JsMap,
    Ico,
}

impl FileKind {
    pub fn content_type(&self) -> &'static str {
        match self {
            FileKind::Html => "text/html; charset=utf-8",
            FileKind::Css => "text/css; charset=utf-8",
            FileKind::Js => "text/javascript; charset=utf-8",
            FileKind::Ts => "text/typescript; charset=utf-8",
            FileKind::JsMap => "application/json; charset=utf-8",
            FileKind::Ico => "image/x-icon",
        }
    }
}
