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

    /// Pushes an equality span to `self.tokens`.
    /// Automatically attaches span details to the given equality.
    pub(super) fn push_equality(&mut self, equality: Token, end: usize) {
        self.tokens
            .push(equality.at(self.line, self.equality_start, end));
    }

    /// Handles long operators, such as `++`, `--`, `==`, & `//`.
    ///
    /// # Arguments
    ///
    /// `single_token` is pushed if this operator is the short variant (`=`, `+`)
    /// `double_token` is pushed if this operator is the long variant (`==`, `++`)
    pub(super) fn long_op(
        &mut self,
        expected_char: char,
        double_token: Token,
        single_token: Token,
    ) {
        // flush any pending identifier/keyword so it is emitted *before* this operator
        // (e.x., `a++` lexes as `[Identifier(a), Increment]`, not the reverse)
        self.flush_keyword();

        let start = self.char;
        if let Some(peek_char) = self.chars.get(self.char + 1) {
            if *peek_char == expected_char {
                // both the leading and trailing chars here are ASCII, so 1+1 byte span
                self.tokens
                    .push(double_token.at(self.line, start, start + 2));
                self.char += 1;
            } else {
                self.tokens
                    .push(single_token.at(self.line, start, start + 1));
            }
        } else {
            self.tokens
                .push(single_token.at(self.line, start, start + 1));
        }
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
