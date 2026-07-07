use super::*;

impl Lexer {
    /// Peeks at the next character in `self.chars`.
    pub(super) fn peek(&self) -> Option<&char> {
        self.chars.get(self.char + 1)
    }

    /// Pushes a token span to `self.tokens`.
    /// Automatically attaches span details to the given token.
    pub(super) fn push_token(&mut self, tok: Token, end: usize) {
        self.tokens.push(tok.at(self.line, self.token_start, end));
    }

    /// Pushes an operator token to `self.tokens`.
    /// Automatically attaches span details to the given token.
    pub(super) fn push_operator(&mut self, operator: Token, end: usize) {
        self.tokens
            .push(operator.at(self.line, self.operator_start, end));
    }

    /// Flushes the pending keyword register into `self.tokens` as a keyword (if it matches one)
    /// or an identifier, then clears the register. A no-op if the register is empty.
    pub(super) fn flush_keyword(&mut self) {
        if let Some(token) = Token::from_keyword(&self.keyword_register) {
            self.push_token(token, self.char);
        } else if !self.keyword_register.is_empty() {
            self.push_token(
                Token::Identifier(intern!(self.keyword_register.clone())),
                self.char,
            );
        }
        self.keyword_register.clear();
    }
}
