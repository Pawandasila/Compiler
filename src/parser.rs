use std::error::Error;
use std::fmt;
use crate::lexer::{Token, TokenType};

#[derive(Debug, Clone)]
pub enum ASTNode {
    Program(Vec<ASTNode>),
      // Declarations
    VarDeclaration {
        #[allow(dead_code)]
        var_type: String,
        name: String,
        initializer: Option<Box<ASTNode>>,
    },
    
    // Statements
    Block(Vec<ASTNode>),
    ExpressionStatement(Box<ASTNode>),
    IfStatement {
        condition: Box<ASTNode>,
        then_branch: Box<ASTNode>,
        else_branch: Option<Box<ASTNode>>,
    },
    WhileStatement {
        condition: Box<ASTNode>,
        body: Box<ASTNode>,
    },
    ReturnStatement(Option<Box<ASTNode>>),
    
    // Expressions
    BinaryExpression {
        left: Box<ASTNode>,
        operator: TokenType,
        right: Box<ASTNode>,
    },
    UnaryExpression {
        operator: TokenType,
        operand: Box<ASTNode>,
    },
    CallExpression {
        callee: Box<ASTNode>,
        arguments: Vec<ASTNode>,
    },
    AssignmentExpression {
        name: String,
        value: Box<ASTNode>,
    },
    
    // Literals
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    Identifier(String),
}

#[derive(Debug)]
pub struct ParserError {
    message: String,
    line: usize,
    column: usize,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Parser error at {}:{}: {}", self.line, self.column, self.message)
    }
}

impl Error for ParserError {}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }
    
    pub fn parse(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        Ok(ASTNode::Program(statements))
    }
    
    fn declaration(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        if self.match_token(&[TokenType::Int, TokenType::Float]) {
            return self.var_declaration();
        }
        
        self.statement()
    }
    
    fn var_declaration(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        // Save the type token
        let var_type = match &self.previous().token_type {
            TokenType::Int => "int".to_string(),
            TokenType::Float => "float".to_string(),
            _ => unreachable!(),
        };
        
        // Expect an identifier
        if let TokenType::Identifier(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            
            // Check for initializer
            let initializer = if self.match_token(&[TokenType::Assign]) {
                Some(Box::new(self.expression()?))
            } else {
                None
            };
            
            // Expect semicolon
            self.consume(TokenType::Semicolon, "Expected ';' after variable declaration")?;
            
            Ok(ASTNode::VarDeclaration {
                var_type,
                name,
                initializer,
            })
        } else {
            Err(self.error("Expected identifier"))
        }
    }
    
    fn statement(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }
    
    fn if_statement(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after if condition")?;
        
        let then_branch = self.statement()?;
        
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(ASTNode::IfStatement {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }
    
    fn while_statement(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after while condition")?;
        
        let body = self.statement()?;
        
        Ok(ASTNode::WhileStatement {
            condition: Box::new(condition),
            body: Box::new(body),
        })
    }
    
    fn return_statement(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        
        self.consume(TokenType::Semicolon, "Expected ';' after return value")?;
        
        Ok(ASTNode::ReturnStatement(value))
    }
    
    fn block(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        
        Ok(ASTNode::Block(statements))
    }
    
    fn expression_statement(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression")?;
        
        Ok(ASTNode::ExpressionStatement(Box::new(expr)))
    }
    
    fn expression(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        self.assignment()
    }
    
    fn assignment(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let expr = self.equality()?;
        
        if self.match_token(&[TokenType::Assign]) {
            if let ASTNode::Identifier(name) = expr {
                let value = self.assignment()?;
                return Ok(ASTNode::AssignmentExpression {
                    name,
                    value: Box::new(value),
                });
            }
            
            return Err(self.error("Invalid assignment target"));
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut expr = self.comparison()?;
        
        while self.match_token(&[TokenType::Equal, TokenType::NotEqual]) {
            let operator = self.previous().token_type.clone();
            let right = self.comparison()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn comparison(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut expr = self.term()?;
        
        while self.match_token(&[TokenType::LessThan, TokenType::GreaterThan]) {
            let operator = self.previous().token_type.clone();
            let right = self.term()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut expr = self.factor()?;
        
        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous().token_type.clone();
            let right = self.factor()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut expr = self.unary()?;
        
        while self.match_token(&[TokenType::Multiply, TokenType::Divide]) {
            let operator = self.previous().token_type.clone();
            let right = self.unary()?;
            expr = ASTNode::BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        if self.match_token(&[TokenType::Minus]) {
            let operator = self.previous().token_type.clone();
            let operand = self.unary()?;
            return Ok(ASTNode::UnaryExpression {
                operator,
                operand: Box::new(operand),
            });
        }
        
        self.call()
    }
    
    fn call(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        
        Ok(expr)
    }
    
    fn finish_call(&mut self, callee: ASTNode) -> Result<ASTNode, Box<dyn Error>> {
        let mut arguments = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        
        Ok(ASTNode::CallExpression {
            callee: Box::new(callee),
            arguments,
        })
    }
    
    fn primary(&mut self) -> Result<ASTNode, Box<dyn Error>> {
        if self.match_token(&[TokenType::IntLiteral(0)]) {
            if let TokenType::IntLiteral(value) = &self.previous().token_type {
                return Ok(ASTNode::IntLiteral(*value));
            }
            unreachable!(); // Should never reach here
        }
        
        if self.match_token(&[TokenType::FloatLiteral(0.0)]) {
            if let TokenType::FloatLiteral(value) = &self.previous().token_type {
                return Ok(ASTNode::FloatLiteral(*value));
            }
            unreachable!(); // Should never reach here
        }
        
        if self.match_token(&[TokenType::StringLiteral(String::new())]) {
            if let TokenType::StringLiteral(value) = &self.previous().token_type {
                return Ok(ASTNode::StringLiteral(value.clone()));
            }
            unreachable!(); // Should never reach here
        }
        
        if self.match_token(&[TokenType::Identifier(String::new())]) {
            if let TokenType::Identifier(name) = &self.previous().token_type {
                return Ok(ASTNode::Identifier(name.clone()));
            }
            unreachable!(); // Should never reach here
        }
        
        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression")?;
            return Ok(expr);
        }
        
        Err(self.error(&format!("Expected expression, got {:?}", self.peek().token_type)))
    }
    
    // Helper methods
    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        
        false
    }
    
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        
        match (token_type, &self.peek().token_type) {
            (TokenType::IntLiteral(_), TokenType::IntLiteral(_)) => true,
            (TokenType::FloatLiteral(_), TokenType::FloatLiteral(_)) => true,
            (TokenType::StringLiteral(_), TokenType::StringLiteral(_)) => true,
            (TokenType::Identifier(_), TokenType::Identifier(_)) => true,
            _ => std::mem::discriminant(token_type) == std::mem::discriminant(&self.peek().token_type),
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::EOF)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
    
    fn current_token(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, Box<dyn Error>> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }
    
    fn error(&self, message: &str) -> Box<dyn Error> {
        let token = self.peek();
        Box::new(ParserError {
            message: message.to_string(),
            line: token.line,
            column: token.column,
        })
    }
}