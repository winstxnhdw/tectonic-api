use tectonic::driver;
use tectonic::errors;

pub fn latex_to_pdf<T: AsRef<str>>(latex: T) -> errors::Result<Vec<u8>> {
    let name = "texput";
    let config = tectonic::config::PersistentConfig::open(false)?;
    let mut builder = driver::ProcessingSessionBuilder::default();

    builder
        .bundle(config.default_bundle(true)?)
        .primary_input_buffer(latex.as_ref().as_bytes())
        .tex_input_name(&format!("{name}.tex"))
        .format_name("latex")
        .format_cache_path(config.format_cache_path()?)
        .keep_logs(false)
        .keep_intermediates(false)
        .print_stdout(false)
        .output_format(driver::OutputFormat::Pdf)
        .do_not_write_output_files();

    let mut status = tectonic::status::NoopStatusBackend::default();
    let mut session = builder.create(&mut status)?;

    session.run(&mut status)?;
    session
        .into_file_data()
        .remove(&format!("{name}.pdf"))
        .ok_or_else(|| errors::Error::from("LaTeX didn't report failure, but no PDF was created (??)"))
        .map(|file| file.data)
}
