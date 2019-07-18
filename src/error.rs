use failure::Fail;

#[derive(Debug, Fail)]
pub enum CompilationErrorKind {
    #[fail(display = "alias `{}` shadows recipe defined on line `{}`", alias, recipe_line)]
    AliasShadowsRecipe {
        alias: String,
        recipe_line: usize,
    }
}

#[derive(Debug, Fail)]
#[fail(display = "{} at {}:{}:{}", kind, filename, line, column)]
pub struct CompilationError {
    pub line: usize,
    pub column: usize,
    pub filename: String,
    pub kind: CompilationErrorKind,
}