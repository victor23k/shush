/// This lexer performs lexing on the fly, in order to make syntax highlighting possible
#[allow(dead_code)]


#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    OutGreaterThan,
    EnvVar,
    Item,
    EOF,
}

#[derive(Debug)]
pub struct TokenShush {
    token: Token,
    content: Option<String>,
}

impl TokenShush {
    pub fn new(token: Token, content: Option<String>) -> TokenShush {
        TokenShush { token, content }
    }
}

#[derive(Debug)]
pub enum LexerState {
    General,
    InsideItem,
    EnvVar,
}

#[derive(Debug)]
pub struct Lexer {
    state: LexerState,
    lexed: Vec<TokenShush>,
    acc: String,
}

impl Lexer {
    pub fn new() -> Lexer {
        Lexer {
            state: LexerState::General,
            lexed: Vec::new(),
            acc: String::new(),
        }
    }

    pub fn lex(&mut self, next_char: char) {
        match next_char {
            '>' => {
                self.state = LexerState::General;
                self.lexed
                    .push(TokenShush::new(Token::OutGreaterThan, None));
            }
            '$' => {
                self.state = LexerState::EnvVar;
                self.acc = String::new();
            }
            ' ' => match self.state {
                LexerState::InsideItem => {
                    self.state = LexerState::General;
                    if !self.acc.is_empty() {
                        self.lexed
                            .push(TokenShush::new(Token::Item, Some(self.acc.clone())));
                    }
                }
                LexerState::General => (),
                LexerState::EnvVar => {
                    self.state = LexerState::General;
                    if !self.acc.is_empty() {
                        self.lexed
                            .push(TokenShush::new(Token::EnvVar, Some(self.acc.clone())));
                    }
                }
            },
            '\n' => {
                match self.state {
                    LexerState::General => (),
                    LexerState::InsideItem => {
                        if !self.acc.is_empty() {
                            self.lexed
                                .push(TokenShush::new(Token::Item, Some(self.acc.clone())));
                        }
                    }
                    LexerState::EnvVar => {
                        if !self.acc.is_empty() {
                            self.lexed
                                .push(TokenShush::new(Token::EnvVar, Some(self.acc.clone())));
                        }
                    }
                };
                self.lexed.push(TokenShush::new(Token::EOF, None));
            }
            _ => match self.state {
                LexerState::General => {
                    self.state = LexerState::InsideItem;
                    self.acc = String::new();
                    self.acc.push(next_char);
                }
                LexerState::InsideItem => {
                    self.acc.push(next_char);
                }
                LexerState::EnvVar => {
                    self.acc.push(next_char);
                }
            },
        };
    }
}

#[cfg(test)]
mod tests {
    use super::Lexer;
    use super::Token;

    #[test]
    fn it_does_lex() {
        let mut lexer = Lexer::new();

        let input: String = "ls $PWD\n".to_string();
        for char in input.chars() {
            lexer.lex(char);
        }
        assert_eq!(3, lexer.lexed.len());
        let mut iter_lexed = lexer.lexed.iter();
        let first_token = &iter_lexed.next().unwrap().token;
        assert_eq!(first_token, &Token::Item);
        let second_token = &iter_lexed.next().unwrap();
        assert_eq!(second_token.content.as_ref().unwrap(), "PWD");
        let third_token = &iter_lexed.next().unwrap().token;
        assert_eq!(third_token, &Token::EOF);
    }
}
