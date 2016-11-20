quick_error! {
    #[derive(Debug, PartialEq)]
    pub enum Error {
        UnbalancedBrace(s: String) {
            description("unbalanced braces")
                display("unbalanced braces in {}", s)
        }
        MissingNameOrPattern(s: String) {
            description("missing name or pattern")
                display("missing name or pattern in {}", s)
        }
        StartWithSlash(s: String) {
            description("path must start with a slash")
                display("path must start with a slash, got {}", s)
        }
    }
}
