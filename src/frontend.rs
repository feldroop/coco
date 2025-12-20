#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Html,
    Css,
    Js,
    JsMap,
    Base64Ico,
}

impl FileKind {
    pub fn content_type(&self) -> &'static str {
        match self {
            FileKind::Html => "text/html; charset=utf-8",
            FileKind::Css => "text/css; charset=utf-8",
            FileKind::Js => "text/javascript; charset=utf-8",
            FileKind::JsMap => "application/json; charset=utf-8",
            FileKind::Base64Ico => "image/x-icon; base64",
        }
    }
}

pub struct FrontEndFile {
    pub kind: FileKind,
    pub name: &'static str,
    pub path: &'static str,
    pub content: &'static [u8],
}

pub const FILES: &[FrontEndFile] = &[
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
        kind: FileKind::JsMap,
        name: "login.js.map",
        path: "/login.js.map",
        content: include_bytes!("../frontend/out/login.js.map"),
    },
    FrontEndFile {
        kind: FileKind::Base64Ico,
        name: "favicon.ico",
        path: "/favicon.ico",
        content: include_bytes!("../frontend/assets/favicon.ico"),
    },
];
