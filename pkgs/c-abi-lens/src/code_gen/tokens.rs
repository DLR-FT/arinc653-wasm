use super::RepresentableCType;

/// Generic representation of a snippet of C-Code
pub enum CSnippet {
    Include(CInclude),
    Func(CFunc),
    Section(CSection),
    Newline,
}

impl From<CInclude> for CSnippet {
    fn from(value: CInclude) -> Self {
        Self::Include(value)
    }
}

impl From<CFunc> for CSnippet {
    fn from(value: CFunc) -> Self {
        Self::Func(value)
    }
}

impl From<CSection> for CSnippet {
    fn from(value: CSection) -> Self {
        Self::Section(value)
    }
}

/// Representation of an include
pub enum CInclude {
    /// Using `<...>`
    System(String),
    /// Using `"..."`
    #[allow(dead_code)]
    Library(String),
}

impl CInclude {
    pub fn generate(&self) -> String {
        match self {
            Self::System(h) => format!("#include<{h}>\n"),
            Self::Library(h) => format!(r#"#include"{h}"\n"#),
        }
    }
}

/// Representation of a function in C
pub struct CFunc {
    pub comment: String,
    pub return_type: RepresentableCType,
    pub name: String,
    pub arguments: Vec<(RepresentableCType, String)>,
    pub body: String,
}

impl CFunc {
    /// Generate function code
    ///
    /// # Arguments
    ///
    /// - `emit_comment`: whether to also emit a comment explaining the function up-front
    /// - `emit_body`: whether to emit the body of the function, or only a prototyp/forward declartion of the function signature
    /// - `func_decl_prefix`: any possible function prefix to be emitted up front
    pub fn generate(
        &self,
        emit_comment: bool,
        emit_body: bool,
        func_decl_prefix: Option<&str>,
    ) -> String {
        let Self {
            return_type, name, ..
        } = self;

        let comment = if emit_comment {
            self.format_comment()
        } else {
            Default::default()
        };

        let func_decl_prefix = Self::format_func_prefix(func_decl_prefix);

        let args = self.format_args();

        let body = if emit_body {
            format!("{{\n{}\n}}", self.format_body())
        } else {
            ";\n".to_owned()
        };

        format!("{comment}{func_decl_prefix}{return_type} {name}({args}){body}")
    }

    /// Format a function prefix followed by a space, or an empty string
    fn format_func_prefix(maybe_prefix: Option<&str>) -> String {
        maybe_prefix.map(|x| format!("{x} ")).unwrap_or_default()
    }

    /// Formats a comment string into what C actually considers a comment (e.g. each line prefixed with `// `)
    fn format_comment(&self) -> String {
        let space = " ";
        let mut result = String::from("/*");
        for line in self.comment.lines() {
            if !line.is_empty() {
                result.push_str(space);
                result.push_str(line);
            }
            result.push_str("\n *");
        }
        result.push_str("/\n");
        result
    }

    /// Formats a function body
    ///
    /// No trailing newline
    fn format_body(&self) -> String {
        let indentation_token = "\t";
        let mut result = String::new();
        for line in self.body.lines() {
            if !line.is_empty() {
                result.push_str(indentation_token);
                result.push_str(line);
            }
            result.push('\n');
        }
        result.remove(result.len() - 1); // remove the final `'\n'`
        result
    }

    /// Formats the arugments into an argument list
    fn format_args(&self) -> String {
        let arg_sep_token = ", ";
        let mut vec: Vec<_> = self
            .arguments
            .iter()
            .map(|(type_, name)| type_.format_as_type(Some(name)))
            .collect();

        // no argumnts? Then the function must have `void` in the parenthesis of the declaration!
        if vec.is_empty() {
            vec.push("void".to_owned());
        }

        vec.join(arg_sep_token)
    }
}

/// Representation of a section in C
pub struct CSection {
    pub title: String,
    pub comment: String,
}

impl CSection {
    /// Generate section code
    ///
    /// # Arguments
    ///
    /// - `emit_comment`: whether to also emit a comment explaining the function up-front
    /// - `emit_body`: whether to emit the body of the function, or only a prototyp/forward declartion of the function signature
    /// - `func_decl_prefix`: any possible function prefix to be emitted up front
    pub fn generate(&self, width: u16) -> String {
        let Self { title, comment } = self;
        let width = width as usize - 4;
        let mut result = format!("/*{title:*^width$}*/\n");

        if comment.is_empty() {
            return result;
        }
        result.push('/');

        for line in comment.lines() {
            result.push_str(&format!("* {line}\n "));
        }
        result.push_str("*/\n");

        result
    }
}
