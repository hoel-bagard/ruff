use ruff_diagnostics::AlwaysFixableViolation;
use ruff_diagnostics::Diagnostic;
use ruff_diagnostics::Edit;
use ruff_diagnostics::Fix;
use ruff_python_parser::Tok;
use std::iter::Flatten;
use std::slice::Iter;

use ruff_macros::{derive_message_formats, violation};
use ruff_python_codegen::Stylist;
use ruff_python_parser::lexer::LexResult;
use ruff_python_parser::lexer::LexicalError;
use ruff_python_parser::TokenKind;
use ruff_source_file::Locator;

use ruff_text_size::TextRange;
use ruff_text_size::TextSize;

use crate::checkers::logical_lines::expand_indent;
use crate::line_width::IndentWidth;

/// Number of blank lines around top level classes and functions.
const BLANK_LINES_TOP_LEVEL: u32 = 2;
/// Number of blank lines around methods and nested classes and functions.
const BLANK_LINES_METHOD_LEVEL: u32 = 1;

/// ## What it does
/// Checks for missing blank lines between methods of a class.
///
/// ## Why is this bad?
/// PEP 8 recommends the use of blank lines as follows:
/// - Two blank lines are expected between functions and classes
/// - One blank line is expected between methods of a class.
///
/// ## Example
/// ```python
/// class MyClass(object):
///     def func1():
///         pass
///     def func2():
///         pass
/// ```
///
/// Use instead:
/// ```python
/// class MyClass(object):
///     def func1():
///         pass
///
///     def func2():
///         pass
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E301.html)
#[violation]
pub struct BlankLineBetweenMethods {
    actual_blank_lines: u32,
}

impl AlwaysFixableViolation for BlankLineBetweenMethods {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLineBetweenMethods {
            actual_blank_lines: nb_blank_lines,
        } = self;
        format!("Expected {BLANK_LINES_METHOD_LEVEL:?} blank line, found {nb_blank_lines}")
    }

    fn fix_title(&self) -> String {
        "Add missing blank line(s)".to_string()
    }
}

/// ## What it does
/// Checks for missing blank lines between top level functions and classes.
///
/// ## Why is this bad?
/// PEP 8 recommends the use of blank lines as follows:
/// - Two blank lines are expected between functions and classes
/// - One blank line is expected between methods of a class.
///
/// ## Example
/// ```python
/// def func1():
///     pass
/// def func2():
///     pass
/// ```
///
/// Use instead:
/// ```python
/// def func1():
///     pass
///
///
/// def func2():
///     pass
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E302.html)
#[violation]
pub struct BlankLinesTopLevel {
    actual_blank_lines: u32,
}

impl AlwaysFixableViolation for BlankLinesTopLevel {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesTopLevel {
            actual_blank_lines: nb_blank_lines,
        } = self;

        format!("Expected {BLANK_LINES_TOP_LEVEL:?} blank lines, found {nb_blank_lines}")
    }

    fn fix_title(&self) -> String {
        "Add missing blank line(s)".to_string()
    }
}

/// ## What it does
/// Checks for extraneous blank lines.
///
/// ## Why is this bad?
/// PEP 8 recommends using blank lines as follows:
/// - No more than two blank lines between top-level statements.
/// - No more than one blank line between non-top-level statements.
///
/// ## Example
/// ```python
/// def func1():
///     pass
///
///
///
/// def func2():
///     pass
/// ```
///
/// Use instead:
/// ```python
/// def func1():
///     pass
///
///
/// def func2():
///     pass
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E303.html)
#[violation]
pub struct TooManyBlankLines {
    actual_blank_lines: u32,
}

impl AlwaysFixableViolation for TooManyBlankLines {
    #[derive_message_formats]
    fn message(&self) -> String {
        let TooManyBlankLines {
            actual_blank_lines: nb_blank_lines,
        } = self;
        format!("Too many blank lines ({nb_blank_lines})")
    }

    fn fix_title(&self) -> String {
        "Remove extraneous blank line(s)".to_string()
    }
}

/// ## What it does
/// Checks for missing blank line after function decorator.
///
/// ## Why is this bad?
/// PEP 8 recommends the use of blank lines as follows:
/// - Two blank lines are expected between functions and classes
/// - One blank line is expected between methods of a class.
///
/// ## Example
/// ```python
/// class User(object):
///
///     @property
///
///     def name(self):
///         pass
/// ```
///
/// Use instead:
/// ```python
/// class User(object):
///
///     @property
///     def name(self):
///         pass
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E304.html)
#[violation]
pub struct BlankLineAfterDecorator;

impl AlwaysFixableViolation for BlankLineAfterDecorator {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("blank lines found after function decorator")
    }

    fn fix_title(&self) -> String {
        "Remove extraneous blank line(s)".to_string()
    }
}

/// ## What it does
/// Checks for missing blank lines after end of function or class.
///
/// ## Why is this bad?
/// PEP 8 recommends the using blank lines as following:
/// - Two blank lines are expected between functions and classes
/// - One blank line is expected between methods of a class.
///
/// ## Example
/// ```python
/// class User(object):
///     pass
/// user = User()
/// ```
///
/// Use instead:
/// ```python
/// class User(object):
///     pass
///
///
/// user = User()
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E305.html)
#[violation]
pub struct BlankLinesAfterFunctionOrClass {
    actual_blank_lines: u32,
}

impl AlwaysFixableViolation for BlankLinesAfterFunctionOrClass {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesAfterFunctionOrClass {
            actual_blank_lines: blank_lines,
        } = self;
        format!("expected 2 blank lines after class or function definition, found ({blank_lines})")
    }

    fn fix_title(&self) -> String {
        "Add missing blank line(s)".to_string()
    }
}

/// ## What it does
/// Checks for for 1 blank line between nested functions/classes definitions.
///
/// ## Why is this bad?
/// PEP 8 recommends the using blank lines as following:
/// - Two blank lines are expected between functions and classes
/// - One blank line is expected between methods of a class.
///
/// ## Example
/// ```python
/// def outer():
///     def inner():
///         pass
///     def inner2():
///         pass
/// ```
///
/// Use instead:
/// ```python
/// def outer():
///     def inner():
///         pass
///
///     def inner2():
///         pass
/// ```
///
/// ## References
/// - [PEP 8](https://peps.python.org/pep-0008/#blank-lines)
/// - [Flake 8 rule](https://www.flake8rules.com/rules/E306.html)
#[violation]
pub struct BlankLinesBeforeNestedDefinition {
    actual_blank_lines: u32,
}

impl AlwaysFixableViolation for BlankLinesBeforeNestedDefinition {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesBeforeNestedDefinition {
            actual_blank_lines: blank_lines,
        } = self;
        format!("Expected 1 blank line before a nested definition, found {blank_lines}")
    }

    fn fix_title(&self) -> String {
        "Add missing blank line".to_string()
    }
}

/// Returns `true` if the token is a top level token.
/// It is sufficient to test for Class and Def since the `LinePreprocessor` ignores Async tokens.
fn is_top_level_token(token: Option<TokenKind>) -> bool {
    matches!(&token, Some(TokenKind::Class | TokenKind::Def))
}

/// Returns `true` if the token is At, Async, Class or Def
fn is_top_level_token_or_decorator(token: TokenKind) -> bool {
    matches!(&token, TokenKind::Class | TokenKind::Def | TokenKind::At)
}

#[derive(Debug)]
struct LogicalLineInfo {
    kind: LogicalLineKind,
    first_token_range: TextRange,

    // The token's kind right before the newline ending the logical line.
    last_token: TokenKind,

    // The end of the logical line including the newline.
    logical_line_end: TextSize,
    is_comment_only: bool,
    is_docstring: bool,
    indent_length: usize,
    /// `blank_lines` is the straightforward amount of blank lines preceding the current line.
    blank_lines: u32,
    /// `preceding_blank_lines` is the maximum number of consecutive blank lines between the current line
    /// and the previous non-comment logical line.
    /// One of its main uses is to allow a comment to directly precede a class/function definition.
    /// It is also used to match the results of pydocstyle.
    preceding_blank_lines: u32,
    preceding_blank_characters: usize,
}

/// Iterator that processes tokens until a full logical line (or comment line) is "built".
/// It then returns characteristics of that logical line (see `LogicalLineInfo`).
struct LinePreprocessor<'a> {
    tokens: Iter<'a, Result<(Tok, TextRange), LexicalError>>,
    locator: &'a Locator<'a>,
    indent_width: IndentWidth,
    /// Maximum number of consecutive blank lines between the current line and the previous non-comment logical line.
    /// One of its main uses is to allow a comment to directly precede a class/function definition.
    /// It is also used to match the results of pydocstyle.
    preceding_blank_lines: u32,
}

impl<'a> LinePreprocessor<'a> {
    fn new(
        tokens: &'a [LexResult],
        locator: &'a Locator,
        indent_width: IndentWidth,
    ) -> LinePreprocessor<'a> {
        LinePreprocessor {
            tokens: tokens.iter(),
            locator,
            preceding_blank_lines: 0,
            indent_width,
        }
    }
}

impl<'a> Iterator for LinePreprocessor<'a> {
    type Item = LogicalLineInfo;

    fn next(&mut self) -> Option<LogicalLineInfo> {
        let mut line_is_comment_only = true;
        let mut is_docstring = false;
        // Number of consecutive blank lines.
        let mut current_blank_lines = 0u32;
        // Number of blank characters in the blank lines (\n vs \r\n for example).
        let mut current_blank_characters: usize = 0;
        let mut logical_line_start: Option<(LogicalLineKind, TextRange)> = None;
        let mut last_token: TokenKind = TokenKind::EndOfFile;
        let mut parens = 0u32;

        while let Some(result) = self.tokens.next() {
            let Ok((token, range)) = result else {
                continue;
            };

            if matches!(token, Tok::Indent | Tok::Dedent) {
                continue;
            }

            let token_kind = TokenKind::from_token(token);

            let (logical_line_kind, first_token_range) = if let Some(first_token_range) =
                logical_line_start
            {
                first_token_range
            }
            // At the start of the line...
            else {
                // An empty line
                if token_kind == TokenKind::NonLogicalNewline {
                    current_blank_lines += 1;
                    current_blank_characters += range.len().to_usize();
                    // self.current_blank_characters +=
                    //     range.end().to_usize() - first_range.start().to_usize() + 1;

                    continue;
                }

                is_docstring = token_kind == TokenKind::String;

                let logical_line_kind = match token_kind {
                    TokenKind::Class => LogicalLineKind::Class,
                    TokenKind::Comment => LogicalLineKind::Comment,
                    TokenKind::At => LogicalLineKind::Decorator,
                    TokenKind::Def => LogicalLineKind::Function,
                    TokenKind::Async
                        if matches!(self.tokens.as_slice().first(), Some(Ok((Tok::Def, _)))) =>
                    {
                        LogicalLineKind::Function
                    }
                    _ => LogicalLineKind::Other,
                };

                logical_line_start = Some((logical_line_kind, *range));

                (logical_line_kind, *range)
            };

            if !token_kind.is_trivia() {
                line_is_comment_only = false;
            }

            // A docstring line is composed only of the docstring (TokenKind::String) and trivia tokens.
            // (If a comment follows a docstring, we still count the line as a docstring)
            if token_kind != TokenKind::String && !token_kind.is_trivia() {
                is_docstring = false;
            }

            match token_kind {
                TokenKind::Lbrace | TokenKind::Lpar | TokenKind::Lsqb => {
                    parens = parens.saturating_add(1);
                }
                TokenKind::Rbrace | TokenKind::Rpar | TokenKind::Rsqb => {
                    parens = parens.saturating_sub(1);
                }
                TokenKind::Newline | TokenKind::NonLogicalNewline if parens == 0 => {
                    let last_token_end = range.end();

                    let range = TextRange::new(
                        self.locator.line_start(first_token_range.start()),
                        first_token_range.start(),
                    );

                    let indent_length = expand_indent(self.locator.slice(range), self.indent_width);

                    if self.preceding_blank_lines < current_blank_lines {
                        self.preceding_blank_lines = current_blank_lines;
                    }

                    let logical_line = LogicalLineInfo {
                        kind: logical_line_kind,
                        first_token_range,
                        last_token,
                        logical_line_end: last_token_end,
                        is_comment_only: line_is_comment_only,
                        is_docstring,
                        indent_length,
                        blank_lines: current_blank_lines,
                        preceding_blank_lines: self.preceding_blank_lines,
                        preceding_blank_characters: current_blank_characters,
                    };

                    if !line_is_comment_only {
                        self.preceding_blank_lines = 0;
                    }
                    return Some(logical_line);
                }
                _ => {}
            }

            last_token = token_kind;
        }

        None
    }
}

#[derive(Copy, Clone, Debug, Default)]
enum Follows {
    #[default]
    Other,
    Decorator,
    Def,
    Docstring,
}

#[derive(Copy, Clone, Debug, Default)]
enum Status {
    /// Stores the indent level where the nesting started.
    Inside(usize),
    /// This is used to rectify a Inside switched to a Outside because of a dedented comment.
    CommentAfter(usize),
    #[default]
    Outside,
}

/// Contains variables used for the linting of blank lines.
#[derive(Debug, Default)]
pub(crate) struct BlankLinesChecker {
    follows: Follows,
    fn_status: Status,
    class_status: Status,
    /// First line that is not a comment.
    is_not_first_logical_line: bool,
    /// Used for the fix in case a comment separates two non-comment logical lines to make the comment "stick"
    /// to the second line instead of the first.
    last_non_comment_line_end: TextSize,
    previous_unindented_line_kind: Option<LogicalLineKind>,
}

impl BlankLinesChecker {
    /// E301, E302, E303, E304, E305, E306
    pub(crate) fn check_lines(
        &mut self,
        tokens: &[LexResult],
        locator: &Locator,
        stylist: &Stylist,
        indent_width: IndentWidth,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut prev_indent_length: Option<usize> = None;
        let line_preprocessor = LinePreprocessor::new(tokens, locator, indent_width);

        for logical_line in line_preprocessor {
            self.check_line(
                &logical_line,
                prev_indent_length,
                locator,
                stylist,
                diagnostics,
            );
            if !logical_line.is_comment_only {
                prev_indent_length = Some(logical_line.indent_length);
            }
        }
    }

    #[allow(clippy::nonminimal_bool)]
    fn check_line(
        &mut self,
        line: &LogicalLineInfo,
        prev_indent_length: Option<usize>,
        locator: &Locator,
        stylist: &Stylist,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match self.class_status {
            Status::Inside(nesting_indent) => {
                if line.indent_length <= nesting_indent {
                    if line.is_comment_only {
                        self.class_status = Status::CommentAfter(nesting_indent);
                    } else {
                        self.class_status = Status::Outside;
                    }
                }
            }
            Status::CommentAfter(indent) => {
                if !line.is_comment_only {
                    if line.indent_length > indent {
                        self.class_status = Status::Inside(indent);
                    }
                    self.class_status = Status::Outside;
                }
            }
            Status::Outside => {
                // Nothing to do
            }
        }

        if let Status::Inside(nesting_indent) = self.fn_status {
            if line.indent_length <= nesting_indent {
                if line.is_comment_only {
                    self.fn_status = Status::CommentAfter(nesting_indent);
                } else {
                    self.fn_status = Status::Outside;
                }
            }
        }

        // A comment can be de-indented while still being in a class/function, in that case
        // we need to revert the variables.
        if !line.is_comment_only {
            if let Status::CommentAfter(indent) = self.fn_status {
                if line.indent_length > indent {
                    self.fn_status = Status::Inside(indent);
                } else {
                    self.fn_status = Status::Outside;
                }
            }
        }

        // Don't expect blank lines before the first non comment line.
        if self.is_not_first_logical_line {
            if line.preceding_blank_lines == 0
                // Only applies to methods.
                && matches!(line.kind,  LogicalLineKind::Function)
                && matches!(self.class_status, Status::Inside(_))
                // The class/parent method's docstring can directly precede the def.
                && !matches!(self.follows, Follows::Docstring)
                // Do not trigger when the def follows an if/while/etc...
                && prev_indent_length.is_some_and(|prev_indent_length| prev_indent_length >= line.indent_length)
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !matches!(self.follows, Follows::Decorator)
            {
                // E301
                let mut diagnostic = Diagnostic::new(
                    BlankLineBetweenMethods {
                        actual_blank_lines: line.preceding_blank_lines,
                    },
                    line.first_token_range,
                );
                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist.line_ending().as_str().to_string(),
                    locator.line_start(self.last_non_comment_line_end),
                )));

                diagnostics.push(diagnostic);
            }

            if line.preceding_blank_lines < BLANK_LINES_TOP_LEVEL
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !matches!(self.follows, Follows::Decorator)
                // Allow groups of one-liners.
                && !(matches!(self.follows, Follows::Def) && !matches!(line.last_token, TokenKind::Colon))
                // Only trigger on non-indented classes and functions (for example functions within an if are ignored)
                && line.indent_length == 0
                // Only apply to functions or classes.
                && line.kind.is_top_level()
            {
                // E302
                let mut diagnostic = Diagnostic::new(
                    BlankLinesTopLevel {
                        actual_blank_lines: line.preceding_blank_lines,
                    },
                    line.first_token_range,
                );
                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist
                        .line_ending()
                        .as_str()
                        .to_string()
                        .repeat((BLANK_LINES_TOP_LEVEL - line.preceding_blank_lines) as usize),
                    locator.line_start(self.last_non_comment_line_end),
                )));

                diagnostics.push(diagnostic);
            }

            if line.blank_lines > BLANK_LINES_TOP_LEVEL
                || (line.indent_length > 0 && line.blank_lines > BLANK_LINES_METHOD_LEVEL)
            {
                // E303
                let mut diagnostic = Diagnostic::new(
                    TooManyBlankLines {
                        actual_blank_lines: line.blank_lines,
                    },
                    line.first_token_range,
                );

                let chars_to_remove = if line.indent_length > 0 {
                    u32::try_from(line.preceding_blank_characters)
                        .expect("Number of blank characters to be small.")
                        - BLANK_LINES_METHOD_LEVEL
                } else {
                    u32::try_from(line.preceding_blank_characters)
                        .expect("Number of blank characters to be small.")
                        - BLANK_LINES_TOP_LEVEL
                };
                let end = locator.line_start(line.first_token_range.start());
                let start = end - TextSize::new(chars_to_remove);
                diagnostic.set_fix(Fix::safe_edit(Edit::deletion(start, end)));

                diagnostics.push(diagnostic);
            }

            if matches!(self.follows, Follows::Decorator) && line.preceding_blank_lines > 0 {
                // E304
                let mut diagnostic =
                    Diagnostic::new(BlankLineAfterDecorator, line.first_token_range);

                let range = line.first_token_range;
                diagnostic.set_fix(Fix::safe_edit(Edit::deletion(
                    locator.line_start(range.start())
                        - TextSize::new(
                            line.preceding_blank_characters
                                .try_into()
                                .expect("Number of blank characters to be small."),
                        ),
                    locator.line_start(range.start()),
                )));

                diagnostics.push(diagnostic);
            }

            if line.preceding_blank_lines < BLANK_LINES_TOP_LEVEL
                && self
                    .previous_unindented_line_kind
                    .is_some_and(|kind| kind.is_top_level())
                && line.indent_length == 0
                && !line.is_comment_only
                && !line.kind.is_top_level()
            {
                // E305
                let mut diagnostic = Diagnostic::new(
                    BlankLinesAfterFunctionOrClass {
                        actual_blank_lines: line.blank_lines,
                    },
                    line.first_token_range,
                );

                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist
                        .line_ending()
                        .as_str()
                        .to_string()
                        .repeat((BLANK_LINES_TOP_LEVEL - line.blank_lines) as usize),
                    locator.line_start(line.first_token_range.start()),
                )));

                diagnostics.push(diagnostic);
            }

            if line.preceding_blank_lines == 0
            // Only apply to nested functions.
                && matches!(self.fn_status, Status::Inside(_))
                && line.kind.is_top_level()
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !matches!(self.follows, Follows::Decorator)
                // The class's docstring can directly precede the first function.
                && !matches!(self.follows, Follows::Docstring)
                // Do not trigger when the def/class follows an "indenting token" (if/while/etc...).
                && prev_indent_length.is_some_and(|prev_indent_length| prev_indent_length >= line.indent_length)
                // Allow groups of one-liners.
                && !(matches!(self.follows, Follows::Def) && line.last_token != TokenKind::Colon)
            {
                // E306
                let mut diagnostic = Diagnostic::new(
                    BlankLinesBeforeNestedDefinition {
                        actual_blank_lines: line.blank_lines,
                    },
                    line.first_token_range,
                );

                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist.line_ending().as_str().to_string(),
                    locator.line_start(line.first_token_range.start()),
                )));

                diagnostics.push(diagnostic);
            }
        }

        match line.kind {
            LogicalLineKind::Class => {
                if matches!(self.class_status, Status::Outside) {
                    self.class_status = Status::Inside(line.indent_length);
                }
                self.follows = Follows::Other;
            }
            LogicalLineKind::Decorator => {
                self.follows = Follows::Decorator;
            }
            LogicalLineKind::Function => {
                if matches!(self.fn_status, Status::Outside) {
                    self.fn_status = Status::Inside(line.indent_length);
                }
                self.follows = Follows::Def;
            }
            LogicalLineKind::Comment => {}
            LogicalLineKind::Other => {
                self.follows = Follows::Other;
            }
        }

        if line.is_docstring {
            self.follows = Follows::Docstring;
        }

        if !line.is_comment_only {
            self.is_not_first_logical_line = true;

            self.last_non_comment_line_end = line.logical_line_end;

            if line.indent_length == 0 {
                self.previous_unindented_line_kind = Some(line.kind);
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum LogicalLineKind {
    /// The clause header of a class definition
    Class,
    /// A decorator
    Decorator,
    /// The clause header of a function
    Function,
    /// A comment only line
    Comment,
    /// Any other statement or clause header
    Other,
}

impl LogicalLineKind {
    fn is_top_level(self) -> bool {
        matches!(
            self,
            LogicalLineKind::Class | LogicalLineKind::Function | LogicalLineKind::Decorator
        )
    }
}
