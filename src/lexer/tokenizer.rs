use crate::{error::all_error::AllError, utils::logger::Logger};

use super::{
    general::GeneralToken, keyword::Keyword, operator::OperatorToken, primary::PrimaryToken,
    token::Token,
};

#[derive(Debug)]
pub struct Tokenizer {
    buffer: Vec<char>,
    buffer_index: Option<usize>,
    last_char: Option<char>,
}

impl Tokenizer {
    pub fn new(text: String) -> Self {
        Logger::info(format!("echo: {:?}", text));
        Self {
            last_char: None,
            buffer: text.chars().collect(),
            buffer_index: None,
        }
    }

    fn is_whitespace(&self) -> bool {
        match self.last_char {
            Some(' ') | Some('\n') | Some('\t') => true,
            _ => false,
        }
    }

    fn is_digit(&self) -> bool {
        match self.last_char {
            Some(c) => c.is_ascii_digit(),
            _ => false,
        }
    }

    fn is_alphabet(&self) -> bool {
        match self.last_char {
            Some(c) => c.is_ascii_alphabetic(),
            _ => false,
        }
    }

    fn is_alphabet_or_number(&self) -> bool {
        match self.last_char {
            Some(c) => c.is_ascii_alphanumeric(),
            _ => false,
        }
    }

    fn is_underscore(&self) -> bool {
        match self.last_char {
            Some('_') => true,
            _ => false,
        }
    }

    fn is_operator_character(&self) -> bool {
        match self.last_char {
            Some(c) => [
                '+', '-', '*', '/', '%', '|', ',', '>', '<', '=', '!', '\\', '.', '&', '^', '~',
                '?',
            ]
            .contains(&c),
            _ => false,
        }
    }

    fn is_general_syntax_character(&self) -> bool {
        match self.last_char {
            Some(c) => [
                '(', ')', '{', '}', '[', ']', ',', ';', ':', '@', '`', '$', '#',
            ]
            .contains(&c),
            _ => false,
        }
    }

    fn is_quote(&self) -> bool {
        match self.last_char {
            Some(c) => ['\'', '"'].contains(&c),
            _ => false,
        }
    }

    fn is_dot(&self) -> bool {
        match self.last_char {
            Some('.') => true,
            _ => false,
        }
    }

    fn is_eof(&self) -> bool {
        match self.buffer_index {
            Some(index) => index >= self.buffer.len(),
            _ => false,
        }
    }

    // 버퍼에서 문자 하나를 읽어서 last_char에 보관합니다.
    fn read_char(&mut self) {
        let buffer_index = match self.buffer_index {
            Some(index) => index + 1,
            None => 0,
        };

        self.buffer_index = Some(buffer_index);

        self.last_char = self.buffer.get(buffer_index).map(|e| e.to_owned());
    }

    // 보관했던 문자 하나를 다시 버퍼에 돌려놓습니다.
    fn unread_char(&mut self) {
        if self.is_eof() {
            return;
        }

        let buffer_index = match self.buffer_index {
            Some(index) => index - 1,
            None => {
                return;
            }
        };
        self.buffer_index = Some(buffer_index);
        self.last_char = self.buffer.get(buffer_index).map(|e| e.to_owned());
    }

    // 주어진 텍스트에서 토큰을 순서대로 획득해 반환합니다.
    // 끝을 만날 경우 Token::EOF를 반환합니다.
    pub fn get_token(&mut self) -> Result<Token, AllError> {
        self.read_char();

        // 화이트 스페이스 삼킴
        while self.is_whitespace() && !self.is_eof() {
            self.read_char();
        }

        // 첫번째 글짜가 알파벳일 경우 식별자 및 키워드로 인식
        let token = if self.is_alphabet() || self.is_underscore() {
            let mut identifier = vec![self.last_char.unwrap()];

            self.read_char();
            loop {
                if self.is_alphabet_or_number() || self.is_underscore() {
                    identifier.push(self.last_char.unwrap());
                    self.read_char();
                } else {
                    break;
                }
            }

            if self.is_general_syntax_character() || self.is_operator_character() {
                self.unread_char();
            }

            let identifier: String = identifier.into_iter().collect::<String>();

            let token = match identifier.as_str() {
                "let" => Token::Keyword(Keyword::Let),
                "const" => Token::Keyword(Keyword::Const),
                "mut" => Token::Keyword(Keyword::Mut),
                "static" => Token::Keyword(Keyword::Static),
                "fn" => Token::Keyword(Keyword::Fn),
                "return" => Token::Keyword(Keyword::Return),
                "if" => Token::Keyword(Keyword::If),
                "else" => Token::Keyword(Keyword::Else),
                "match" => Token::Keyword(Keyword::Match),
                "break" => Token::Keyword(Keyword::Break),
                "continue" => Token::Keyword(Keyword::Continue),
                "as" => Token::Keyword(Keyword::As),
                "in" => Token::Keyword(Keyword::In),
                "for" => Token::Keyword(Keyword::For),
                "while" => Token::Keyword(Keyword::While),
                "loop" => Token::Keyword(Keyword::Loop),
                "async" => Token::Keyword(Keyword::Async),
                "await" => Token::Keyword(Keyword::Await),
                "use" => Token::Keyword(Keyword::Use),
                "struct" => Token::Keyword(Keyword::Struct),
                "class" => Token::Keyword(Keyword::Class),
                "impl" => Token::Keyword(Keyword::Impl),
                "true" => Token::Primary(PrimaryToken::Boolean(true)),
                "false" => Token::Primary(PrimaryToken::Boolean(false)),
                "where" => Token::Keyword(Keyword::Where),
                "type" => Token::Keyword(Keyword::Type),
                "unsafe" => Token::Keyword(Keyword::Unsafe),
                "void" => Token::Keyword(Keyword::Void),
                "self" => Token::Keyword(Keyword::_Self),
                "Self" => Token::Keyword(Keyword::_SelfType),
                _ => PrimaryToken::Identifier(identifier).into(),
            };

            return Ok(token);
        }
        // 첫번째 글자가 숫자일 경우 정수 및 실수값으로 인식
        else if self.is_digit() {
            let mut number_string = vec![self.last_char.unwrap()];

            // 숫자나 .이 나올 때까지만 버퍼에서 읽어서 number_string에 저장
            loop {
                if self.is_eof() {
                    break;
                }

                self.read_char();
                if self.is_digit() || self.is_dot() {
                    number_string.push(self.last_char.unwrap());
                    continue;
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
        else if self.is_operator_character() {
            match self.last_char.unwrap() {
                ',' => GeneralToken::Comma.into(),
                '-' => {
                    self.read_char();

                    match self.last_char {
                        Some('>') => GeneralToken::Arrow.into(),
                        Some('=') => OperatorToken::MinusAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Minus.into()
                        }
                    }
                }
                '/' => {
                    // 다음 문자가 *일 경우 블록 단위 주석으로 처리

                    self.read_char();

                    match self.last_char {
                        Some('*') => {
                            let mut comment = vec![];

                            self.read_char();
                            while !self.is_eof() {
                                match self.last_char {
                                    Some('*') => {
                                        self.read_char();
                                        if self.last_char == Some('/') {
                                            break;
                                        }
                                    }
                                    Some(c) => {
                                        comment.push(c);
                                    }
                                    None => {
                                        return Err(AllError::LexerError(
                                            "unexpected EOF".to_string(),
                                        ));
                                    }
                                }

                                self.read_char();
                            }

                            let comment: String = comment.into_iter().collect();
                            PrimaryToken::Comment(comment).into()
                        }
                        Some('/') => {
                            let mut comment = vec![];

                            while self.has_next() {
                                self.read_char();

                                match self.last_char {
                                    Some('\n') => {
                                        break;
                                    }
                                    Some(c) => {
                                        comment.push(c);
                                    }
                                    None => {
                                        return Err(AllError::LexerError(
                                            "unexpected EOF".to_string(),
                                        ));
                                    }
                                }
                            }

                            let comment: String = comment.into_iter().collect();
                            PrimaryToken::Comment(comment).into()
                        }
                        Some('=') => OperatorToken::SlashAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Slash.into()
                        }
                    }

                    // if self.last_char == '*' {
                    //     let mut comment = vec![];

                    //     self.read_char();
                    //     while !self.is_eof() {
                    //         if self.last_char == '*' {
                    //             self.read_char();
                    //             if self.last_char == '/' {
                    //                 break;
                    //             }
                    //         } else {
                    //             comment.push(self.last_char);
                    //         }

                    //         self.read_char();
                    //     }

                    //     let comment: String = comment.into_iter().collect();
                    //     PrimaryToken::Comment(comment).into()
                    // } else if self.last_char == '/' {
                    //     let mut comment = vec![];

                    //     while !self.is_eof() {
                    //         self.read_char();

                    //         if self.last_char == '\n' {
                    //             break;
                    //         } else {
                    //             comment.push(self.last_char);
                    //         }
                    //     }

                    //     let comment: String = comment.into_iter().collect();
                    //     PrimaryToken::Comment(comment).into()
                    // } else if self.last_char == '=' {
                    //     OperatorToken::SlashAssign.into()
                    // } else {
                    //     self.unread_char();
                    //     OperatorToken::Slash.into()
                    // }
                }
                '%' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::ModuloAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Modulo.into()
                        }
                    }
                }
                '+' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::PlusAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Plus.into()
                        }
                    }
                }
                '*' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::StarAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Star.into()
                        }
                    }
                }
                '!' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::NotEqual.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Not.into()
                        }
                    }
                }
                '?' => OperatorToken::Question.into(),
                '.' => {
                    self.read_char();

                    match self.last_char {
                        Some('.') => OperatorToken::Range.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Dot.into()
                        }
                    }
                }
                '=' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::Equal.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Assign.into()
                        }
                    }
                }
                '<' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::LessThanOrEqual.into(),
                        Some('<') => {
                            self.read_char();

                            match self.last_char {
                                Some('=') => OperatorToken::LeftShiftAssign.into(),
                                _ => {
                                    self.unread_char();
                                    OperatorToken::LeftShift.into()
                                }
                            }
                        }
                        _ => {
                            self.unread_char();
                            OperatorToken::LessThan.into()
                        }
                    }
                }
                '>' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::GreaterThanOrEqual.into(),
                        Some('>') => {
                            self.read_char();

                            match self.last_char {
                                Some('=') => OperatorToken::RightShiftAssign.into(),
                                _ => {
                                    self.unread_char();
                                    OperatorToken::RightShift.into()
                                }
                            }
                        }
                        _ => {
                            self.unread_char();
                            OperatorToken::GreaterThan.into()
                        }
                    }
                }
                '&' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::AndAssign.into(),
                        Some('&') => OperatorToken::And.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::Ampersand.into()
                        }
                    }
                }
                '|' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::OrAssign.into(),
                        Some('|') => OperatorToken::Or.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::BitwiseOr.into()
                        }
                    }
                }
                '^' => {
                    self.read_char();

                    match self.last_char {
                        Some('=') => OperatorToken::XorAssign.into(),
                        _ => {
                            self.unread_char();
                            OperatorToken::BitwiseXor.into()
                        }
                    }
                }
                '~' => OperatorToken::BitwiseNot.into(),
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
            if let Some('"') = self.last_char {
                let mut identifier = vec![];

                self.read_char();
                loop {
                    if let Some('"') = self.last_char {
                        break;
                    }

                    identifier.push(self.last_char.unwrap());
                    self.read_char();
                }

                let identifier: String = identifier.into_iter().collect::<String>();

                PrimaryToken::String(identifier).into()
            } else if let Some('\'') = self.last_char {
                let mut string = vec![];

                self.read_char();
                while !self.is_eof() {
                    if let Some('\'') = self.last_char {
                        self.read_char();

                        // '' 의 형태일 경우 '로 이스케이프
                        // 아닐 경우 문자열 종료
                        if let Some('\'') = self.last_char {
                            string.push('\'');
                        } else {
                            self.unread_char();
                            break;
                        }
                    } else if let Some(c) = self.last_char {
                        string.push(c);
                    }

                    self.read_char();
                }

                let string: String = string.into_iter().collect::<String>();

                PrimaryToken::String(string).into()
            } else {
                return Err(AllError::LexerError(format!(
                    "unexpected character: {:?}",
                    self.last_char
                )));
            }
        }
        // 기타 문자 부호들 처리
        else if self.is_general_syntax_character() {
            match self.last_char.unwrap() {
                '(' => GeneralToken::LeftParentheses.into(),
                ')' => GeneralToken::RightParentheses.into(),
                '{' => GeneralToken::LeftBrace.into(),
                '}' => GeneralToken::RightBrace.into(),
                '[' => GeneralToken::LeftBracket.into(),
                ']' => GeneralToken::RightBracket.into(),
                ';' => GeneralToken::SemiColon.into(),
                ':' => GeneralToken::Colon.into(),
                '@' => GeneralToken::At.into(),
                '`' => GeneralToken::Backtick.into(),
                ',' => GeneralToken::Comma.into(),
                _ => {
                    return Err(AllError::LexerError(format!(
                        "unexpected token: {:?}",
                        self.last_char
                    )))
                }
            }
        }
        // 아무것도 해당되지 않을 경우 예외처리
        else if self.is_eof() {
            Token::Eof
        } else {
            return Err(AllError::LexerError(format!(
                "unexpected character: {:?}",
                self.last_char
            )));
        };

        self.last_char = None;

        Ok(token)
    }

    pub fn has_next(&self) -> bool {
        match self.buffer_index {
            Some(buffer_index) => buffer_index + 1 < self.buffer.len(),
            None => true,
        }
    }

    // Tokenizer 생성 없이 토큰 목록을 가져올 수 있는 boilerplate 함수입니다.
    pub fn string_to_tokens(text: String) -> Result<Vec<Token>, AllError> {
        let mut tokenizer = Tokenizer::new(text);

        let mut tokens = vec![];

        while tokenizer.has_next() {
            let token = tokenizer.get_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }
}

impl std::fmt::Display for Tokenizer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Tokenizer: {:?}", self)
    }
}
