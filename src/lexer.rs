use std::error::Error;
use std::fmt;

/// Enum representing different types of tokens.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Int, Float, If, Else, While, Return, 
    
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    
    // Identifiers
    Identifier(String),
    
    // Operators
    Plus, Minus, Multiply, Divide, Assign,
    Equal, NotEqual, LessThan, GreaterThan,
    
    // Punctuation
    LeftParen, RightParen, 
    LeftBrace, RightBrace,
    Semicolon, Comma,
    
    // Special
    EOF,
}

/// Struct representing a token, along with its line and column in the source.
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

/// Custom error for the lexer.
#[derive(Debug)]
pub struct LexerError {
    message: String,
    line: usize,
    column: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl Error for LexerError {}

/// Lexer struct that holds state while tokenizing input.
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Creates a new Lexer instance from an input string.
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Tokenizes the input into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, Box<dyn Error>> {
        let mut tokens = Vec::new();
        
        while self.position < self.input.len() {
            let c = self.current_char();
            
            match c {
                // Whitespace characters
                ' ' | '\t' | '\r' => self.advance(),

                // Newline: increment line count
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.advance();
                }

                // Numeric literal
                '0'..='9' => tokens.push(self.number()?),

                // Identifier or keyword
                'a'..='z' | 'A'..='Z' | '_' => tokens.push(self.identifier()?),

                // String literal
                '"' => tokens.push(self.string_literal()?),

                // Operators
                '+' => {
                    tokens.push(self.create_token(TokenType::Plus));
                    self.advance();
                },
                '-' => {
                    tokens.push(self.create_token(TokenType::Minus));
                    self.advance();
                },
                '*' => {
                    tokens.push(self.create_token(TokenType::Multiply));
                    self.advance();
                },
                '/' => {
                    // Handle comments
                    if self.peek() == '/' {
                        self.advance();
                        self.advance();
                        self.skip_line_comment();
                    } else if self.peek() == '*' {
                        self.advance();
                        self.advance();
                        self.skip_block_comment()?;
                    } else {
                        tokens.push(self.create_token(TokenType::Divide));
                        self.advance();
                    }
                },
                '=' => {
                    if self.peek() == '=' {
                        self.advance();
                        self.advance();
                        tokens.push(self.create_token(TokenType::Equal));
                    } else {
                        tokens.push(self.create_token(TokenType::Assign));
                        self.advance();
                    }
                },
                '!' => {
                    if self.peek() == '=' {
                        self.advance();
                        self.advance();
                        tokens.push(self.create_token(TokenType::NotEqual));
                    } else {
                        return Err(Box::new(LexerError {
                            message: format!("Unexpected character: !"),
                            line: self.line,
                            column: self.column,
                        }));
                    }
                },
                '<' => {
                    tokens.push(self.create_token(TokenType::LessThan));
                    self.advance();
                },
                '>' => {
                    tokens.push(self.create_token(TokenType::GreaterThan));
                    self.advance();
                },

                // Punctuation
                '(' => {
                    tokens.push(self.create_token(TokenType::LeftParen));
                    self.advance();
                },
                ')' => {
                    tokens.push(self.create_token(TokenType::RightParen));
                    self.advance();
                },
                '{' => {
                    tokens.push(self.create_token(TokenType::LeftBrace));
                    self.advance();
                },
                '}' => {
                    tokens.push(self.create_token(TokenType::RightBrace));
                    self.advance();
                },
                ';' => {
                    tokens.push(self.create_token(TokenType::Semicolon));
                    self.advance();
                },
                ',' => {
                    tokens.push(self.create_token(TokenType::Comma));
                    self.advance();
                },

                // Any other character is unexpected
                _ => {
                    return Err(Box::new(LexerError {
                        message: format!("Unexpected character: {}", c),
                        line: self.line,
                        column: self.column,
                    }));
                }
            }
        }
        
        // Add EOF token at the end
        tokens.push(Token {
            token_type: TokenType::EOF,
            line: self.line,
            column: self.column,
        });
        
        Ok(tokens)
    }
    
    /// Returns the current character.
    fn current_char(&self) -> char {
        self.input[self.position]
    }
    
    /// Peeks ahead to the next character without advancing.
    fn peek(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }
    
    /// Advances the lexer by one character.
    fn advance(&mut self) {
        self.position += 1;
        self.column += 1;
    }
    
    /// Helper to create a token at the current position.
    fn create_token(&self, token_type: TokenType) -> Token {
        Token {
            token_type,
            line: self.line,
            column: self.column,
        }
    }
    
    /// Parses a number (integer or float).
    fn number(&mut self) -> Result<Token, Box<dyn Error>> {
        let start_pos = self.position;
        let mut is_float = false;
        
        while self.position < self.input.len() {
            let c = self.current_char();
            
            if c.is_digit(10) {
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }
        
        let number_str: String = self.input[start_pos..self.position].iter().collect();
        
        let token_type = if is_float {
            match number_str.parse::<f64>() {
                Ok(value) => TokenType::FloatLiteral(value),
                Err(_) => return Err(Box::new(LexerError {
                    message: format!("Invalid float literal: {}", number_str),
                    line: self.line,
                    column: self.column - number_str.len(),
                })),
            }
        } else {
            match number_str.parse::<i64>() {
                Ok(value) => TokenType::IntLiteral(value),
                Err(_) => return Err(Box::new(LexerError {
                    message: format!("Invalid integer literal: {}", number_str),
                    line: self.line,
                    column: self.column - number_str.len(),
                })),
            }
        };
        
        Ok(Token {
            token_type,
            line: self.line,
            column: self.column - number_str.len(),
        })
    }
    
    /// Parses an identifier or keyword.
    fn identifier(&mut self) -> Result<Token, Box<dyn Error>> {
        let start_pos = self.position;
        
        while self.position < self.input.len() {
            let c = self.current_char();
            
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        let ident: String = self.input[start_pos..self.position].iter().collect();
        let column = self.column - ident.len();
        
        // Check if it's a keyword
        let token_type = match ident.as_str() {
            "int" => TokenType::Int,
            "float" => TokenType::Float,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "return" => TokenType::Return,
            _ => TokenType::Identifier(ident),
        };
        
        Ok(Token {
            token_type,
            line: self.line,
            column,
        })
    }
    
    /// Parses a string literal.
    fn string_literal(&mut self) -> Result<Token, Box<dyn Error>> {
        self.advance(); // Skip opening quote
        let start_pos = self.position;
        
        while self.position < self.input.len() && self.current_char() != '"' {
            if self.current_char() == '\n' {
                return Err(Box::new(LexerError {
                    message: "Unterminated string literal".to_string(),
                    line: self.line,
                    column: self.column,
                }));
            }

            // Handle escaped characters like \" or \n
            if self.current_char() == '\\' && self.position + 1 < self.input.len() {
                self.advance(); // Skip backslash
            }
            
            self.advance();
        }
        
        if self.position >= self.input.len() {
            return Err(Box::new(LexerError {
                message: "Unterminated string literal".to_string(),
                line: self.line,
                column: self.column,
            }));
        }
        
        let string_content: String = self.input[start_pos..self.position].iter().collect();
        let column = self.column - string_content.len() - 1; // account for opening quote
        
        self.advance(); // Skip closing quote
        
        Ok(Token {
            token_type: TokenType::StringLiteral(string_content),
            line: self.line,
            column,
        })
    }
    
    /// Skips a single-line comment.
    fn skip_line_comment(&mut self) {
        while self.position < self.input.len() && self.current_char() != '\n' {
            self.advance();
        }
    }
    
    /// Skips a block comment (/* ... */).
    fn skip_block_comment(&mut self) -> Result<(), Box<dyn Error>> {
        while self.position + 1 < self.input.len() {
            if self.current_char() == '*' && self.peek() == '/' {
                self.advance(); // Skip '*'
                self.advance(); // Skip '/'
                return Ok(());
            }
            
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            
            self.advance();
        }
        
        Err(Box::new(LexerError {
            message: "Unterminated block comment".to_string(),
            line: self.line,
            column: self.column,
        }))
    }
}
