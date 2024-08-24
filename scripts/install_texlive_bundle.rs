#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
tectonic = { git = "https://github.com/tectonic-typesetting/tectonic.git", default-features = false, features = ["geturl-curl"] }
---

fn main() {
    _ = tectonic::latex_to_pdf(
        r#"\documentclass{article}\begin{document}Hello, world!\end{document}"#,
    );
}
