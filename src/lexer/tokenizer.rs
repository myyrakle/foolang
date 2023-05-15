use std::error::Error;

use crate::{error::all_error::AllError, utils::logger::Logger};

use super::{
    general::GeneralToken, keyword::Keyword, operator::OperatorToken, primary::PrimaryToken,
    token::Token,
};

#[derive(Debug)]
pub struct Tokenizer {
    buffer: Vec<char>,
    buffer_index: usize,
    last_char: char,
}

impl Tokenizer {
    pub fn new(text: String) -> Self {
        Logger::info(format!("echo: {:?}", text));
        Self {
            last_char: ' ',
            buffer: text.chars().collect(),
            buffer_index: 0,
        }
    }

    pub fn is_whitespace(&self) -> bool {
        self.last_char == ' ' || self.last_char == '\n' || self.last_char == '\t'
    }

    pub fn is_digit(&self) -> bool {
        self.last_char.is_ascii_digit()
    }

    pub fn is_alphabet(&self) -> bool {
        self.last_char.is_alphabetic()
    }

    pub fn is_alphabet_or_number(&self) -> bool {
        self.last_char.is_alphanumeric()
    }

    pub fn is_underscore(&self) -> bool {
        self.last_char == '_'
    }

    pub fn is_backslash(&self) -> bool {
        self.last_char == '\\'
    }

    pub fn is_special_character(&self) -> bool {
        [
            '+', '-', '*', '/', '%', '|', ',', '>', '<', '=', '!', '\\', '.', '&', '^', '~', '?',
        ]
        .contains(&self.last_char)
    }

    pub fn is_quote(&self) -> bool {
        ['\'', '"'].contains(&self.last_char)
    }

    pub fn is_semicolon(&self) -> bool {
        self.last_char == ';'
    }

    pub fn is_dot(&self) -> bool {
        self.last_char == '.'
    }

    pub fn is_backtick(&self) -> bool {
        self.last_char == '`'
    }

    pub fn is_parentheses(&self) -> bool {
        self.last_char == '(' || self.last_char == ')'
    }

    pub fn is_eof(&self) -> bool {
        self.buffer_index >= self.buffer.len()
    }

    // 버퍼에서 문자 하나를 읽어서 last_char에 보관합니다.
    pub fn read_char(&mut self) {
        if self.buffer_index >= self.buffer.len() {
            self.last_char = ' ';
        } else {
            self.last_char = self.buffer[self.buffer_index];
            self.buffer_index += 1;
        }
    }

    // 보관했던 문자 하나를 다시 버퍼에 돌려놓습니다.
    pub fn unread_char(&mut self) {
        if self.buffer_index == 0 {
            self.last_char = ' ';
        } else {
            self.buffer_index -= 1;
            self.last_char = self.buffer[self.buffer_index];
        }
    }

    // 주어진 텍스트에서 토큰을 순서대로 획득해 반환합니다.
    // 끝을 만날 경우 Token::EOF를 반환합니다.
    pub fn get_token(&mut self) -> Result<Token, AllError> {
        // 화이트 스페이스 삼킴
        while self.is_whitespace() && !self.is_eof() {
            self.read_char();
        }

        // 첫번째 글짜가 알파벳일 경우 식별자 및 키워드로 인식
        let token = if self.is_alphabet() || self.is_underscore() {
            let mut identifier = vec![self.last_char];

            self.read_char();
            while self.is_alphabet_or_number() || self.is_underscore() {
                identifier.push(self.last_char);
                self.read_char();
            }

            let identifier: String = identifier.into_iter().collect::<String>();

            let token = match identifier.as_str() {
                "let" => Token::Keyword(Keyword::Let).into(),
                "const" => Token::Keyword(Keyword::Const).into(),
                "mut" => Token::Keyword(Keyword::Mut).into(),
                "static" => Token::Keyword(Keyword::Static).into(),
                "fn" => Token::Keyword(Keyword::Fn).into(),
                "return" => Token::Keyword(Keyword::Return).into(),
                "if" => Token::Keyword(Keyword::If).into(),
                "else" => Token::Keyword(Keyword::Else).into(),
                "match" => Token::Keyword(Keyword::Match).into(),
                "break" => Token::Keyword(Keyword::Break).into(),
                "continue" => Token::Keyword(Keyword::Continue).into(),
                "as" => Token::Keyword(Keyword::As).into(),
                "in" => Token::Keyword(Keyword::In).into(),
                "for" => Token::Keyword(Keyword::For).into(),
                "while" => Token::Keyword(Keyword::While).into(),
                "loop" => Token::Keyword(Keyword::Loop).into(),
                "async" => Token::Keyword(Keyword::Async).into(),
                "await" => Token::Keyword(Keyword::Await).into(),
                "use" => Token::Keyword(Keyword::Use).into(),
                "struct" => Token::Keyword(Keyword::Struct).into(),
                "class" => Token::Keyword(Keyword::Class).into(),
                "impl" => Token::Keyword(Keyword::Impl).into(),
                "true" => Token::Keyword(Keyword::True).into(),
                "false" => Token::Keyword(Keyword::False).into(),
                "where" => Token::Keyword(Keyword::Where).into(),
                "type" => Token::Keyword(Keyword::Type).into(),
                "unsafe" => Token::Keyword(Keyword::Unsafe).into(),
                "void" => Token::Keyword(Keyword::Void).into(),
                "self" => Token::Keyword(Keyword::_Self).into(),
                "Self" => Token::Keyword(Keyword::_SelfType).into(),
                _ => PrimaryToken::Identifier(identifier).into(),
            };

            return Ok(token);
        }
        // 첫번째 글자가 숫자일 경우 정수 및 실수값으로 인식
        else if self.is_digit() {
            let mut number_string = vec![self.last_char];

            // 숫자나 .이 나올 때까지만 버퍼에서 읽어서 number_string에 저장
            loop {
                self.read_char();
                if self.is_digit() || self.is_dot() {
                    number_string.push(self.last_char);
                    continue;
                } else if self.is_eof() {
                    break;
                } else {
                    self.unread_char();
                    break;
                }
            }

            let number_string: String =
                number_string.into_iter().collect::<String>().to_uppercase();

            // .이 있을 경우 실수, 아닌 경우 정수로 인식
            if number_string.contains('.') {
                let number = number_string.parse::<f64>();

                match number {
                    Ok(number) => PrimaryToken::Float(number).into(),
                    Err(_) => {
                        return Err(AllError::LexerError(format!(
                            "invalid floating point number format: {}",
                            number_string
                        )))
                    }
                }
            } else {
                let number = number_string.parse::<i64>();

                match number {
                    Ok(number) => PrimaryToken::Integer(number).into(),
                    Err(_) => {
                        return Err(AllError::LexerError(format!(
                            "invalid integer number format: {}",
                            number_string
                        )))
                    }
                }
            }
        }
        // 특수문자일 경우
        else if self.is_special_character() {
            match self.last_char {
                ',' => GeneralToken::Comma.into(),
                '-' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::MinusAssign.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Minus.into()
                    }
                }
                '/' => {
                    // 다음 문자가 *일 경우 블록 단위 주석으로 처리

                    self.read_char();

                    if self.last_char == '*' {
                        let mut comment = vec![];

                        self.read_char();
                        while !self.is_eof() {
                            if self.last_char == '*' {
                                self.read_char();
                                if self.last_char == '/' {
                                    break;
                                }
                            } else {
                                comment.push(self.last_char);
                            }

                            self.read_char();
                        }

                        let comment: String = comment.into_iter().collect();
                        PrimaryToken::Comment(comment).into()
                    } else if self.last_char == '=' {
                        OperatorToken::SlashAssign.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Slash.into()
                    }
                }
                '+' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::PlusAssign.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Plus.into()
                    }
                }
                '*' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::StarAssign.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Star.into()
                    }
                }
                '!' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::NotEqual.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Not.into()
                    }
                }
                '=' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::Equal.into()
                    } else {
                        self.unread_char();
                        OperatorToken::Assign.into()
                    }
                }
                '<' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::LessThanOrEqual.into()
                    } else if self.last_char == '<' {
                        self.read_char();

                        if self.last_char == '=' {
                            OperatorToken::LeftShiftAssign.into()
                        } else {
                            self.unread_char();
                            OperatorToken::LeftShift.into()
                        }
                    } else {
                        self.unread_char();
                        OperatorToken::LessThan.into()
                    }
                }
                '>' => {
                    self.read_char();

                    if self.last_char == '=' {
                        OperatorToken::GreaterThanOrEqual.into()
                    } else if self.last_char == '>' {
                        self.read_char();

                        if self.last_char == '=' {
                            OperatorToken::RightShiftAssign.into()
                        } else {
                            self.unread_char();
                            OperatorToken::RightShift.into()
                        }
                    } else {
                        self.unread_char();
                        OperatorToken::GreaterThan.into()
                    }
                }
                _ => {
                    return Err(AllError::LexerError(format!(
                        "unexpected operator: {:?}",
                        self.last_char
                    )))
                }
            }
        }
        // 따옴표일 경우 처리
        else if self.is_quote() {
            if self.last_char == '"' {
                let mut identifier = vec![];

                self.read_char();
                while self.last_char != '"' {
                    identifier.push(self.last_char);
                    self.read_char();
                }

                let identifier: String = identifier.into_iter().collect::<String>();

                PrimaryToken::Identifier(identifier).into()
            } else if self.last_char == '\'' {
                let mut string = vec![];

                self.read_char();
                while !self.is_eof() {
                    if self.last_char == '\'' {
                        self.read_char();

                        // '' 의 형태일 경우 '로 이스케이프
                        // 아닐 경우 문자열 종료
                        if self.last_char == '\'' {
                            string.push(self.last_char);
                        } else {
                            self.unread_char();
                            break;
                        }
                    } else {
                        string.push(self.last_char);
                    }

                    self.read_char();
                }

                let string: String = string.into_iter().collect::<String>();

                PrimaryToken::String(string).into()
            } else {
                Token::UnknownCharacter(self.last_char)
            }
        } else if self.is_backtick() {
            let mut string = vec![];

            self.read_char();
            while !self.is_eof() {
                if self.last_char == '`' {
                    self.read_char();

                    // `` 의 형태일 경우 `로 이스케이프
                    // 아닐 경우 문자열 종료
                    if self.last_char == '`' {
                        string.push(self.last_char);
                    } else {
                        self.unread_char();
                        break;
                    }
                } else {
                    string.push(self.last_char);
                }

                self.read_char();
            }

            let string: String = string.into_iter().collect::<String>();

            PrimaryToken::Identifier(string).into()
        }
        // 세미콜론
        else if self.is_semicolon() {
            GeneralToken::SemiColon.into()
        }
        // 괄호
        else if self.is_parentheses() {
            if self.last_char == '(' {
                GeneralToken::LeftParentheses.into()
            } else {
                GeneralToken::RightParentheses.into()
            }
        }
        // 아무것도 해당되지 않을 경우 예외처리
        else if self.is_eof() {
            Token::EOF
        } else {
            return Err(AllError::LexerError(format!(
                "unexpected character: {:?}",
                self.last_char
            )));
        };

        self.last_char = ' ';

        Ok(token)
    }

    // Tokenizer 생성 없이 토큰 목록을 가져올 수 있는 boilerplate 함수입니다.
    pub fn string_to_tokens(text: String) -> Result<Vec<Token>, AllError> {
        let mut tokenizer = Tokenizer::new(text);

        let mut tokens = vec![];

        while !tokenizer.is_eof() {
            tokens.push(tokenizer.get_token()?);
        }

        Ok(tokens)
    }
}

impl std::fmt::Display for Tokenizer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Tokenizer: {:?}", self)
    }
}
