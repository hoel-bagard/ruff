use ruff_diagnostics::AlwaysFixableViolation;
use ruff_diagnostics::Diagnostic;
use ruff_diagnostics::Edit;
use ruff_diagnostics::Fix;

use ruff_macros::{derive_message_formats, violation};
use ruff_python_codegen::Stylist;
use ruff_python_parser::TokenKind;
use ruff_source_file::Locator;
use ruff_text_size::TextSize;

use crate::checkers::logical_lines::LogicalLinesContext;

use super::LogicalLine;

/// Contains variables used for the linting of blank lines.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub(crate) struct BlankLinesTrackingVars {
    follows_decorator: bool,
    follows_def: bool,
    follows_docstring: bool,
    is_in_class: bool,
    /// The indent level where the class started.
    class_indent_level: usize,
    is_in_fn: bool,
    /// The indent level where the function started.
    fn_indent_level: usize,
    /// First line that is not a comment.
    is_first_logical_line: bool,
    /// This needs to be tracked between lines since the `is_in_class` and `is_in_fn` are set to
    /// false when a comment is set dedented, but E305 should trigger on the next non-comment line.
    follows_comment_after_fn: bool,
    follows_comment_after_class: bool,
    /// Used for the fix in case a comment separates two non-comment logical lines to make the comment "stick"
    /// to the second line instead of the first.
    last_non_comment_line_end: TextSize,
    previous_unindented_token: Option<TokenKind>,
}

impl Default for BlankLinesTrackingVars {
    fn default() -> BlankLinesTrackingVars {
        BlankLinesTrackingVars {
            follows_decorator: false,
            follows_def: false,
            follows_docstring: false,
            is_in_class: false,
            class_indent_level: 0,
            is_in_fn: false,
            fn_indent_level: 0,
            is_first_logical_line: true,
            follows_comment_after_fn: false,
            follows_comment_after_class: false,
            last_non_comment_line_end: TextSize::new(0),
            previous_unindented_token: None,
        }
    }
}

/// Number of blank lines between various code parts.
struct BlankLinesConfig;

impl BlankLinesConfig {
    /// Number of blank lines around top level classes and functions.
    const TOP_LEVEL: u32 = 2;
    /// Number of blank lines around methods and nested classes and functions.
    const METHOD: u32 = 1;
}

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
pub struct BlankLineBetweenMethods(pub u32);

impl AlwaysFixableViolation for BlankLineBetweenMethods {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLineBetweenMethods(nb_blank_lines) = self;
        format!(
            "Expected {:?} blank line, found {nb_blank_lines}",
            BlankLinesConfig::METHOD
        )
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
pub struct BlankLinesTopLevel(pub u32);

impl AlwaysFixableViolation for BlankLinesTopLevel {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesTopLevel(nb_blank_lines) = self;
        format!(
            "Expected {:?} blank lines, found {nb_blank_lines}",
            BlankLinesConfig::TOP_LEVEL
        )
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
pub struct TooManyBlankLines(pub u32);

impl AlwaysFixableViolation for TooManyBlankLines {
    #[derive_message_formats]
    fn message(&self) -> String {
        let TooManyBlankLines(nb_blank_lines) = self;
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
pub struct BlankLinesAfterFunctionOrClass(pub u32);

impl AlwaysFixableViolation for BlankLinesAfterFunctionOrClass {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesAfterFunctionOrClass(blank_lines) = self;
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
///
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
pub struct BlankLinesBeforeNestedDefinition(pub u32);

impl AlwaysFixableViolation for BlankLinesBeforeNestedDefinition {
    #[derive_message_formats]
    fn message(&self) -> String {
        let BlankLinesBeforeNestedDefinition(blank_lines) = self;
        format!("Expected 1 blank line before a nested definition, found {blank_lines}")
    }

    fn fix_title(&self) -> String {
        "Add missing blank line".to_string()
    }
}

/// Returns `true` if line is a docstring only line.
fn is_docstring(line: Option<&LogicalLine>) -> bool {
    line.is_some_and(|line| {
        !line.tokens_trimmed().is_empty()
            && line
                .tokens_trimmed()
                .iter()
                .all(|token| matches!(token.kind(), TokenKind::String))
    })
}

/// Returns `true` if the token is Async, Class or Def
fn is_top_level_token(token: Option<TokenKind>) -> bool {
    token.is_some_and(|token| matches!(token, TokenKind::Class | TokenKind::Def | TokenKind::Async))
}

/// Returns `true` if the token is At, Async, Class or Def
fn is_top_level_token_or_decorator(token: TokenKind) -> bool {
    matches!(
        token,
        TokenKind::Class | TokenKind::Def | TokenKind::Async | TokenKind::At
    )
}

/// E301, E302, E303, E304, E305, E306
#[allow(clippy::too_many_arguments)]
pub(crate) fn blank_lines(
    line: &LogicalLine,
    tracked_vars: &mut BlankLinesTrackingVars,
    prev_indent_level: Option<usize>,
    indent_level: usize,
    indent_size: usize,
    locator: &Locator,
    stylist: &Stylist,
    context: &mut LogicalLinesContext,
) {
    let line_is_comment_only = line.is_comment_only();

    if indent_level < tracked_vars.class_indent_level && tracked_vars.is_in_class {
        tracked_vars.is_in_class = false;
        if line_is_comment_only {
            tracked_vars.follows_comment_after_class = true;
        }
    }

    if indent_level < tracked_vars.fn_indent_level && tracked_vars.is_in_fn {
        tracked_vars.is_in_fn = false;
        if line_is_comment_only {
            tracked_vars.follows_comment_after_fn = true;
        }
    }

    // A comment can be de-indented while still being in a class/function, in that case
    // we need to revert the variables.
    if tracked_vars.follows_comment_after_fn && !line_is_comment_only {
        if indent_level == tracked_vars.fn_indent_level {
            tracked_vars.is_in_fn = true;
        }
        tracked_vars.follows_comment_after_fn = false;
    }

    if tracked_vars.follows_comment_after_class && !line_is_comment_only {
        if indent_level == tracked_vars.class_indent_level {
            tracked_vars.is_in_class = true;
        }
        tracked_vars.follows_comment_after_class = false;
    }

    for token in line.tokens() {
        if matches!(token.kind, TokenKind::Indent | TokenKind::Dedent) {
            continue;
        }

        // Don't expect blank lines before the first non comment line.
        if !tracked_vars.is_first_logical_line {
            if line.line.preceding_blank_lines == 0
                // Only applies to methods.
                && token.kind() == TokenKind::Def
                && tracked_vars.is_in_class
                // The class/parent method's docstring can directly precede the def.
                && !tracked_vars.follows_docstring
                // Do not trigger when the def follows an if/while/etc...
                && prev_indent_level.is_some_and(|prev_indent_level| prev_indent_level >= indent_level)
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !tracked_vars.follows_decorator
            {
                // E301
                let mut diagnostic = Diagnostic::new(
                    BlankLineBetweenMethods(line.line.preceding_blank_lines),
                    token.range,
                );
                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist.line_ending().as_str().to_string(),
                    locator.line_start(tracked_vars.last_non_comment_line_end),
                )));

                context.push_diagnostic(diagnostic);
            }

            if line.line.preceding_blank_lines < BlankLinesConfig::TOP_LEVEL
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !tracked_vars.follows_decorator
                // Allow groups of one-liners.
                && !(tracked_vars.follows_def
                    && line
                    .tokens_trimmed()
                    .last()
                    .map_or(false, |token| !matches!(token.kind(), TokenKind::Colon))
                )
                // Only trigger on non-indented classes and functions (for example functions within an if are ignored)
                && indent_level == 0
                // Only apply to functions or classes.
                && is_top_level_token_or_decorator(token.kind)
            {
                // E302
                let mut diagnostic = Diagnostic::new(
                    BlankLinesTopLevel(line.line.preceding_blank_lines),
                    token.range,
                );
                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist.line_ending().as_str().to_string().repeat(
                        (BlankLinesConfig::TOP_LEVEL - line.line.preceding_blank_lines) as usize,
                    ),
                    locator.line_start(tracked_vars.last_non_comment_line_end),
                )));

                context.push_diagnostic(diagnostic);
            }

            if line.line.blank_lines > BlankLinesConfig::TOP_LEVEL
                || (indent_level > 0 && line.line.blank_lines > BlankLinesConfig::METHOD)
            {
                // E303
                let mut diagnostic =
                    Diagnostic::new(TooManyBlankLines(line.line.blank_lines), token.range);

                let chars_to_remove = if indent_level > 0 {
                    line.line.preceding_blank_characters - BlankLinesConfig::METHOD
                } else {
                    line.line.preceding_blank_characters - BlankLinesConfig::TOP_LEVEL
                };
                let end = locator.line_start(token.range.start());
                let start = end - TextSize::new(chars_to_remove);
                diagnostic.set_fix(Fix::safe_edit(Edit::deletion(start, end)));

                context.push_diagnostic(diagnostic);
            }

            if tracked_vars.follows_decorator && line.line.preceding_blank_lines > 0 {
                // E304
                let mut diagnostic = Diagnostic::new(BlankLineAfterDecorator, token.range);

                let range = token.range;
                diagnostic.set_fix(Fix::safe_edit(Edit::deletion(
                    locator.line_start(range.start())
                        - TextSize::new(line.line.preceding_blank_characters),
                    locator.line_start(range.start()),
                )));

                context.push_diagnostic(diagnostic);
            }

            if line.line.preceding_blank_lines < BlankLinesConfig::TOP_LEVEL
                && is_top_level_token(tracked_vars.previous_unindented_token)
                && indent_level == 0
                && !line_is_comment_only
                && !is_top_level_token_or_decorator(token.kind)
            {
                // E305
                let mut diagnostic = Diagnostic::new(
                    BlankLinesAfterFunctionOrClass(line.line.blank_lines),
                    token.range,
                );

                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist
                        .line_ending()
                        .as_str()
                        .to_string()
                        .repeat((BlankLinesConfig::TOP_LEVEL - line.line.blank_lines) as usize),
                    locator.line_start(token.range.start()),
                )));

                context.push_diagnostic(diagnostic);
            }

            if line.line.preceding_blank_lines == 0
                // Only apply to nested functions.
                && tracked_vars.is_in_fn
                && is_top_level_token_or_decorator(token.kind)
                // Allow following a decorator (if there is an error it will be triggered on the first decorator).
                && !tracked_vars.follows_decorator
                // The class's docstring can directly precede the first function.
                && !tracked_vars.follows_docstring
                // Do not trigger when the def/class follows an "indenting token" (if/while/etc...), unless that "indenting token" is a def.
                && prev_indent_level.is_some_and(|prev_indent_level| prev_indent_level >= indent_level || tracked_vars.follows_def)
                // Allow groups of one-liners.
                && !(tracked_vars.follows_def
                     && line
                     .tokens_trimmed()
                     .last()
                     .map_or(false, |token| !matches!(token.kind(), TokenKind::Colon))
                )
            {
                // E306
                let mut diagnostic = Diagnostic::new(
                    BlankLinesBeforeNestedDefinition(line.line.blank_lines),
                    token.range,
                );

                diagnostic.set_fix(Fix::safe_edit(Edit::insertion(
                    stylist.line_ending().as_str().to_string(),
                    locator.line_start(token.range.start()),
                )));

                context.push_diagnostic(diagnostic);
            }
        }

        match token.kind() {
            TokenKind::Class => {
                if !tracked_vars.is_in_class {
                    tracked_vars.class_indent_level = indent_level + indent_size;
                }
                tracked_vars.is_in_class = true;
                tracked_vars.follows_decorator = false;
                tracked_vars.follows_def = false;
                break;
            }
            TokenKind::At => {
                tracked_vars.follows_decorator = true;
                tracked_vars.follows_def = false;
                break;
            }
            TokenKind::Def | TokenKind::Async => {
                if !tracked_vars.is_in_fn {
                    tracked_vars.fn_indent_level = indent_level + indent_size;
                }
                tracked_vars.is_in_fn = true;
                tracked_vars.follows_def = true;
                tracked_vars.follows_decorator = false;
                break;
            }
            TokenKind::Comment => {
                break;
            }
            _ => {
                tracked_vars.follows_decorator = false;
                tracked_vars.follows_def = false;
                break;
            }
        }
    }

    if !line_is_comment_only {
        if tracked_vars.is_first_logical_line {
            tracked_vars.is_first_logical_line = false;
        }

        tracked_vars.follows_docstring = is_docstring(Some(line));

        tracked_vars.last_non_comment_line_end = line
            .tokens()
            .last()
            .expect("Line to contain at least one token.")
            .range
            .end();

        if indent_level == 0 && !line.tokens_trimmed().is_empty() {
            tracked_vars.previous_unindented_token = Some(
                line.tokens_trimmed()
                    .first()
                    .expect("Previously checked.")
                    .kind,
            );
        }
    }
}