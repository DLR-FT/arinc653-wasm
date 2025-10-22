use color_eyre::Result;
use color_eyre::eyre::bail;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::mpsc;

/// Compile the `input_file` with a given set of `clang_args`
///
/// Returns `Result::Err(_)` if the compilation fails or yields any diagnostics
///
/// The final arguments passed to clang are from environment variables suffixed by `clang_args`. The
/// reason for this choice is: Environment variables allow the user to configure sysroot etc. for
/// all tests, so that clang can actually compile things. The `clang_args` allow the testsuite's
/// individual tests to modify the compilation as they see fit.
pub fn check_c_file_parses<F: Into<PathBuf>, T: AsRef<str>>(
    input_file: F,
    clang_args: &[T],
) -> Result<()> {
    let (backchannel_tx, backchannel_rx) = mpsc::channel();
    let submission = ClangSubmission {
        input_path_buf: input_file.into(),
        clang_args: clang_args.iter().map(|s| s.as_ref().to_string()).collect(),
        back_channel: backchannel_tx,
    };
    CLANG_COMPILER_THREAD
        .send(submission)
        .expect("clang compiler thread should be initialized by now");

    backchannel_rx.recv().expect("clang compiler thread died")
}

static CLANG_COMPILER_THREAD: LazyLock<mpsc::Sender<ClangSubmission>> = LazyLock::new(|| {
    let (c_file_tx, c_file_rx) = mpsc::channel();
    std::thread::spawn(move || -> ! {
        let clang_singleton = clang::Clang::new().unwrap();
        loop {
            let submission: ClangSubmission = c_file_rx
                .recv()
                .expect("it's ok to fail this thread when the channel dies");
            let result = submission.process(&clang_singleton);

            submission
                .back_channel
                .send(result)
                .expect("it's ok to fail this thread when the channel dies");
        }
    });
    c_file_tx
});

struct ClangSubmission {
    pub input_path_buf: PathBuf,
    pub clang_args: Vec<String>,
    pub back_channel: mpsc::Sender<Result<()>>,
}
impl ClangSubmission {
    fn process(&self, clang: &clang::Clang) -> Result<()> {
        let Self {
            input_path_buf,
            clang_args,
            ..
        } = self;

        // clang ini boilerplate
        let index = clang::Index::new(clang, false, true);

        // check file exists, clang itself does not provide a useful error message if the input file
        // does not exists
        if !std::fs::exists(input_path_buf)? {
            bail!("file {input_path_buf:?}");
        }

        // prepare the final clang_arguments, by extend the environment variable based ones with the
        // `clang_args` passed to this function
        let mut default_clang_args = c_abi_lens::clang_args_from_env();
        default_clang_args.extend(clang_args.iter().map(|s| s.to_string()));
        let clang_args = default_clang_args;
        eprintln!("clang args: {clang_args:#?}");

        // parse and translate the file
        let mut parser = index.parser(input_path_buf);
        parser.arguments(&clang_args).keep_going(true);
        let tu = parser.parse()?;

        // collect all diagnostics
        let diagnostics = tu
            .get_diagnostics()
            .into_iter()
            .map(|d| d.formatter().format())
            .fold(String::new(), |mut acc, d| {
                acc.push('\n');
                acc.push_str(&d);
                acc
            });

        // fail if there are any diagnostics
        if !diagnostics.is_empty() {
            bail!("{diagnostics}")
        }
        Ok(())
    }
}
