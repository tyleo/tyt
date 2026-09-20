use crate::{
    Error, ParseFailure, Result,
    function::Function,
    lexer::{Token, TokenKind},
    parser::{
        BinaryOperator, ComparisonOperator, LogicalOperator, SyntaxBinding, SyntaxNode,
        UnaryOperator,
    },
};
use std::ops::Range;

/// Parses a token stream as a program of `;`-terminated bindings.
///
/// # Arguments
/// - `tokens`: the lexed text.
/// - `end`: the text's byte length, where an unexpected end reports.
pub(crate) fn parse_tokens(tokens: &[Token], end: usize) -> Result<Vec<SyntaxBinding>> {
    let mut parser = Parser {
        tokens,
        position: 0,
        end,
    };
    let mut bindings = Vec::new();

    while let Some(token) = parser.peek() {
        if token.kind == TokenKind::Semicolon {
            parser.position += 1;
            continue;
        }

        bindings.push(parser.binding()?);
    }

    Ok(bindings)
}

/// Parses a token stream as one expression spanning the whole text.
///
/// # Arguments
/// - `tokens`: the lexed text.
/// - `end`: the text's byte length, where an unexpected end reports.
pub(crate) fn parse_expression_tokens(tokens: &[Token], end: usize) -> Result<SyntaxNode> {
    let mut parser = Parser {
        tokens,
        position: 0,
        end,
    };
    let expression = parser.expression()?;

    match parser.peek() {
        None => Ok(expression),
        Some(token) => Err(unexpected(token, "the end of the expression")),
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
    end: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.position)
    }

    fn next(&mut self, expected: &'static str) -> Result<&'a Token> {
        let token = self
            .tokens
            .get(self.position)
            .ok_or_else(|| self.unexpected_end(expected))?;

        self.position += 1;

        Ok(token)
    }

    fn unexpected_end(&self, expected: &'static str) -> Error {
        Error::Parse {
            range: self.end..self.end,
            failure: ParseFailure::UnexpectedEnd { expected },
        }
    }

    fn expect(&mut self, kind: TokenKind, expected: &'static str) -> Result<Range<usize>> {
        let token = self.next(expected)?;

        if token.kind == kind {
            Ok(token.range.clone())
        } else {
            Err(unexpected(token, expected))
        }
    }

    fn binding(&mut self) -> Result<SyntaxBinding> {
        let token = self.next("a binding")?;
        let start = token.range.start;
        let name = match &token.kind {
            TokenKind::Identifier(name) | TokenKind::QuotedName(name) => name.clone(),

            TokenKind::Function(_) | TokenKind::True | TokenKind::False => {
                return Err(reserved(token));
            }

            _ => {
                return Err(Error::Parse {
                    range: token.range.clone(),
                    failure: ParseFailure::ExpectedBinding,
                });
            }
        };

        match self.peek() {
            Some(token) if token.kind == TokenKind::Assign => self.position += 1,

            Some(token) => {
                return Err(Error::Parse {
                    range: start..token.range.end,
                    failure: ParseFailure::ExpectedBinding,
                });
            }

            None => return Err(self.unexpected_end("`=`")),
        }

        let expression = self.expression()?;

        self.expect(TokenKind::Semicolon, "`;`")?;

        Ok(SyntaxBinding { name, expression })
    }

    fn expression(&mut self) -> Result<SyntaxNode> {
        let mut left = self.xor()?;

        while self.peek().is_some_and(|token| token.kind == TokenKind::Or) {
            self.position += 1;

            let right = self.xor()?;

            left = logical(LogicalOperator::Or, left, right);
        }

        Ok(left)
    }

    fn xor(&mut self) -> Result<SyntaxNode> {
        let mut left = self.and()?;

        while self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Xor)
        {
            self.position += 1;

            let right = self.and()?;

            left = logical(LogicalOperator::Xor, left, right);
        }

        Ok(left)
    }

    fn and(&mut self) -> Result<SyntaxNode> {
        let mut left = self.comparison()?;

        while self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::And)
        {
            self.position += 1;

            let right = self.comparison()?;

            left = logical(LogicalOperator::And, left, right);
        }

        Ok(left)
    }

    fn comparison(&mut self) -> Result<SyntaxNode> {
        let mut left = self.additive()?;

        while let Some(operator) = self
            .peek()
            .and_then(|token| comparison_operator(&token.kind))
        {
            self.position += 1;

            let right = self.additive()?;

            left = SyntaxNode::Comparison {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn additive(&mut self) -> Result<SyntaxNode> {
        let mut left = self.term()?;

        loop {
            let operator = match self.peek().map(|token| &token.kind) {
                Some(TokenKind::Plus) => BinaryOperator::Add,
                Some(TokenKind::Minus) => BinaryOperator::Subtract,
                _ => return Ok(left),
            };

            self.position += 1;

            let right = self.term()?;

            left = binary(operator, left, right);
        }
    }

    fn term(&mut self) -> Result<SyntaxNode> {
        let mut left = self.unary()?;

        loop {
            let operator = match self.peek().map(|token| &token.kind) {
                Some(TokenKind::Star) => BinaryOperator::Multiply,
                Some(TokenKind::Slash) => BinaryOperator::Divide,
                _ => return Ok(left),
            };

            self.position += 1;

            let right = self.unary()?;

            left = binary(operator, left, right);
        }
    }

    fn unary(&mut self) -> Result<SyntaxNode> {
        let operator = match self.peek().map(|token| &token.kind) {
            Some(TokenKind::Minus) => UnaryOperator::Negate,
            Some(TokenKind::Not) => UnaryOperator::Not,
            _ => return self.postfix(),
        };

        self.position += 1;

        let operand = self.unary()?;

        Ok(SyntaxNode::Unary {
            operator,
            operand: Box::new(operand),
        })
    }

    fn postfix(&mut self) -> Result<SyntaxNode> {
        let mut source = self.primary()?;

        loop {
            match self.peek().map(|token| &token.kind) {
                Some(TokenKind::Dot) => {
                    self.position += 1;

                    let token = self.next("a swizzle member")?;
                    let member = match &token.kind {
                        TokenKind::Identifier(name) => name.clone(),
                        TokenKind::Function(function) => function.name().to_owned(),
                        TokenKind::True => "true".to_owned(),
                        TokenKind::False => "false".to_owned(),
                        _ => return Err(unexpected(token, "a swizzle member")),
                    };

                    source = SyntaxNode::Swizzle {
                        source: Box::new(source),
                        member,
                    };
                }

                Some(TokenKind::LeftBracket) => {
                    self.position += 1;

                    let index = self.expression()?;

                    self.expect(TokenKind::RightBracket, "`]`")?;

                    source = SyntaxNode::Index {
                        source: Box::new(source),
                        index: Box::new(index),
                    };
                }

                _ => return Ok(source),
            }
        }
    }

    fn primary(&mut self) -> Result<SyntaxNode> {
        let token = self.next("an expression")?;
        let start = token.range.start;

        match &token.kind {
            TokenKind::Number(literal) => Ok(SyntaxNode::Number(literal.clone())),
            TokenKind::True => Ok(SyntaxNode::Bool(true)),
            TokenKind::False => Ok(SyntaxNode::Bool(false)),
            TokenKind::StringLiteral(text) => Ok(SyntaxNode::StringLiteral(text.clone())),

            TokenKind::Identifier(name) | TokenKind::QuotedName(name) => {
                Ok(SyntaxNode::Name(name.clone()))
            }

            TokenKind::LeftParen => {
                let inner = self.expression()?;

                self.expect(TokenKind::RightParen, "`)`")?;

                Ok(inner)
            }

            TokenKind::Function(function) => self.call(*function, start),
            _ => Err(unexpected(token, "an expression")),
        }
    }

    fn call(&mut self, function: Function, start: usize) -> Result<SyntaxNode> {
        self.expect(TokenKind::LeftParen, "`(`")?;

        if function == Function::Default {
            let token = self.next("a name")?;
            let name = match &token.kind {
                TokenKind::Identifier(name) | TokenKind::QuotedName(name) => name.clone(),

                TokenKind::Function(_) | TokenKind::True | TokenKind::False => {
                    return Err(reserved(token));
                }

                _ => return Err(unexpected(token, "a name")),
            };

            self.expect(TokenKind::Comma, "`,`")?;

            let fallback = self.expression()?;

            self.expect(TokenKind::RightParen, "`)`")?;

            return Ok(SyntaxNode::Default {
                name,
                fallback: Box::new(fallback),
            });
        }

        let mut arguments = Vec::new();
        let end = if self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::RightParen)
        {
            self.next("`)`")?.range.end
        } else {
            loop {
                arguments.push(self.expression()?);

                let token = self.next("`,` or `)`")?;

                match token.kind {
                    TokenKind::Comma => continue,
                    TokenKind::RightParen => break token.range.end,
                    _ => return Err(unexpected(token, "`,` or `)`")),
                }
            }
        };

        if !function.takes(arguments.len()) {
            return Err(Error::Parse {
                range: start..end,
                failure: ParseFailure::Arity {
                    function: function.name(),
                    expected: function.arity(),
                    found: arguments.len(),
                },
            });
        }

        Ok(SyntaxNode::Call {
            function,
            arguments,
        })
    }
}

fn binary(operator: BinaryOperator, left: SyntaxNode, right: SyntaxNode) -> SyntaxNode {
    SyntaxNode::Binary {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn logical(operator: LogicalOperator, left: SyntaxNode, right: SyntaxNode) -> SyntaxNode {
    SyntaxNode::Logical {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn comparison_operator(kind: &TokenKind) -> Option<ComparisonOperator> {
    match kind {
        TokenKind::Equal => Some(ComparisonOperator::Equal),
        TokenKind::Greater => Some(ComparisonOperator::Greater),
        TokenKind::GreaterEqual => Some(ComparisonOperator::GreaterEqual),
        TokenKind::Less => Some(ComparisonOperator::Less),
        TokenKind::LessEqual => Some(ComparisonOperator::LessEqual),
        TokenKind::NotEqual => Some(ComparisonOperator::NotEqual),
        _ => None,
    }
}

fn unexpected(token: &Token, expected: &'static str) -> Error {
    Error::Parse {
        range: token.range.clone(),
        failure: ParseFailure::Unexpected {
            found: token.kind.describe(),
            expected,
        },
    }
}

fn reserved(token: &Token) -> Error {
    let name = match &token.kind {
        TokenKind::Function(function) => function.name().to_owned(),
        TokenKind::True => "true".to_owned(),
        TokenKind::False => "false".to_owned(),
        kind => unreachable!("only a keyword is reserved: {}", kind.describe()),
    };

    Error::Parse {
        range: token.range.clone(),
        failure: ParseFailure::ReservedName { name },
    }
}
