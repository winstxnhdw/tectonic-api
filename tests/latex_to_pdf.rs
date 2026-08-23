use tectonic_api::latex_to_pdf;

#[test]
fn compiles_latex_to_pdf() {
    let source = r#"
        \documentclass{article}
        \begin{document}
        Hello, world!
        \end{document}
    "#;

    let pdf = latex_to_pdf(source).expect("valid LaTeX should compile to a PDF");

    assert!(pdf.starts_with(b"%PDF-"));
    assert!(pdf.windows(b"%%EOF".len()).any(|window| window == b"%%EOF"));
}
