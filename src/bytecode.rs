use crate::lexer::TokenType;
use crate::parser::ASTNode;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack operations
    Constant(Value),
    Pop,

    // Variables
    GetLocal(usize),
    SetLocal(usize),
    GetGlobal(String),
    SetGlobal(String),
    DefineGlobal(String),

    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,

    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,

    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    Call(usize), // argument count
    Return,

    // Debug
    Print,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug)]
pub struct BytecodeGeneratorError {
    message: String,
}

impl fmt::Display for BytecodeGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bytecode generator error: {}", self.message)
    }
}

impl Error for BytecodeGeneratorError {}

struct LocalVariable {
    name: String,
    depth: usize,
}

pub struct BytecodeGenerator {
    code: Vec<OpCode>,
    #[allow(dead_code)]
    constants: Vec<Value>,
    locals: Vec<LocalVariable>,
    scope_depth: usize,
    #[allow(dead_code)]
    global_variables: HashMap<String, usize>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        BytecodeGenerator {
            code: Vec::new(),
            constants: Vec::new(),
            locals: Vec::new(),
            scope_depth: 0,
            global_variables: HashMap::new(),
        }
    }

    pub fn generate(&mut self, ast: ASTNode) -> Result<Vec<OpCode>, Box<dyn Error>> {
        match ast {
            ASTNode::Program(statements) => {
                for statement in statements {
                    self.generate_statement(statement)?;
                }
            }
            _ => self.generate_statement(ast)?,
        }

        Ok(self.code.clone())
    }

    fn generate_statement(&mut self, node: ASTNode) -> Result<(), Box<dyn Error>> {
        match node {
            ASTNode::VarDeclaration {
                var_type: _,
                name,
                initializer,
            } => {
                if let Some(init) = initializer {
                    self.generate_expression(*init)?;
                } else {
                    // Push null as default value
                    self.emit(OpCode::Constant(Value::Null));
                }

                self.declare_variable(name)?;
            }
            ASTNode::Block(statements) => {
                self.begin_scope();

                for statement in statements {
                    self.generate_statement(statement)?;
                }

                self.end_scope();
            }
            ASTNode::ExpressionStatement(expr) => {
                self.generate_expression(*expr)?;
                self.emit(OpCode::Pop); // Discard the result
            }
            ASTNode::IfStatement {
                condition,
                then_branch,
                else_branch,
            } => {
                self.generate_expression(*condition)?;

                // Jump to else branch if condition is false
                let jump_if_false = self.emit_jump(OpCode::JumpIfFalse(0));

                // Compile then branch
                self.generate_statement(*then_branch)?;

                // Jump over else branch
                let jump = self.emit_jump(OpCode::Jump(0));

                // Patch jump_if_false to point to else branch or end
                self.patch_jump(jump_if_false);

                // Compile else branch if present
                if let Some(else_stmt) = else_branch {
                    self.generate_statement(*else_stmt)?;
                }

                // Patch jump to point to end
                self.patch_jump(jump);
            }
            ASTNode::WhileStatement { condition, body } => {
                let loop_start = self.code.len();

                // Compile condition
                self.generate_expression(*condition)?;

                // Jump out of loop if condition is false
                let exit_jump = self.emit_jump(OpCode::JumpIfFalse(0));

                // Compile loop body
                self.generate_statement(*body)?;

                // Jump back to condition
                self.emit(OpCode::Jump(loop_start));

                // Patch exit jump
                self.patch_jump(exit_jump);
            }
            ASTNode::ReturnStatement(value) => {
                if let Some(expr) = value {
                    self.generate_expression(*expr)?;
                } else {
                    self.emit(OpCode::Constant(Value::Null));
                }

                self.emit(OpCode::Return);
            }
            _ => {
                return Err(Box::new(BytecodeGeneratorError {
                    message: format!("Unexpected node type in statement context: {:?}", node),
                }));
            }
        }

        Ok(())
    }

    fn generate_expression(&mut self, node: ASTNode) -> Result<(), Box<dyn Error>> {
        match node {
             ASTNode::BinaryExpression {
                left,
                operator,
                right,
            } => {
                self.generate_expression(*left)?;
                self.generate_expression(*right)?;

                match operator {
                    TokenType::Plus => {
                        _ = self.emit(OpCode::Add);
                    }
                    TokenType::Minus => {
                        _ = self.emit(OpCode::Subtract);
                    }
                    TokenType::Multiply => {
                        _ = self.emit(OpCode::Multiply);
                    }
                    TokenType::Divide => {
                        _ = self.emit(OpCode::Divide);
                    }
                    TokenType::Equal => {
                        _ = self.emit(OpCode::Equal);
                    }
                    TokenType::NotEqual => {
                        _ = self.emit(OpCode::NotEqual);
                    }
                    TokenType::LessThan => {
                        _ = self.emit(OpCode::LessThan);
                    }
                    TokenType::GreaterThan => {
                        _ = self.emit(OpCode::GreaterThan);
                    }
                    _ => {
                        return Err(Box::new(BytecodeGeneratorError {
                            message: format!("Unsupported binary operator: {:?}", operator),
                        }));
                    }
                }
            }
            ASTNode::UnaryExpression { operator, operand } => {
                self.generate_expression(*operand)?;

                match operator {
                    TokenType::Minus => {
                        _ = self.emit(OpCode::Negate);
                    }
                    _ => {
                        return Err(Box::new(BytecodeGeneratorError {
                            message: format!("Unsupported unary operator: {:?}", operator),
                        }));
                    }
                }
            }
            ASTNode::CallExpression { callee, arguments } => {
                // Generate code for the callee
                self.generate_expression(*callee)?;

                // Generate code for the arguments
                for arg in &arguments {
                    self.generate_expression(arg.clone())?;
                }

                // Emit call instruction with arg count
                self.emit(OpCode::Call(arguments.len()));
            }
            ASTNode::AssignmentExpression { name, value } => {
                self.generate_expression(*value)?;

                // Check if it's a local variable
                if let Some(index) = self.resolve_local(&name) {
                    self.emit(OpCode::SetLocal(index));
                } else {
                    // Use global variable
                    self.emit(OpCode::SetGlobal(name));
                }
            }
            ASTNode::IntLiteral(value) => {
                self.emit(OpCode::Constant(Value::Int(value)));
            }
            ASTNode::FloatLiteral(value) => {
                self.emit(OpCode::Constant(Value::Float(value)));
            }
            ASTNode::StringLiteral(value) => {
                self.emit(OpCode::Constant(Value::String(value)));
            }
            ASTNode::Identifier(name) => {
                // Check if it's a local variable
                if let Some(index) = self.resolve_local(&name) {
                    self.emit(OpCode::GetLocal(index));
                } else {
                    // Use global variable
                    self.emit(OpCode::GetGlobal(name));
                }
            }
            _ => {
                return Err(Box::new(BytecodeGeneratorError {
                    message: format!("Unexpected node type in expression context: {:?}", node),
                }));
            }
        }

        Ok(())
    }

    fn emit(&mut self, op_code: OpCode) -> usize {
        self.code.push(op_code);
        self.code.len() - 1
    }

    fn emit_jump(&mut self, op_code: OpCode) -> usize {
        self.emit(op_code)
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump_offset = self.code.len();

        // Update the jump instruction with the correct offset
        match &mut self.code[offset] {
            OpCode::JumpIfFalse(to) => *to = jump_offset,
            OpCode::Jump(to) => *to = jump_offset,
            _ => panic!("Tried to patch a non-jump instruction"),
        }
    }

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Pop locals in reverse order
        while !self.locals.is_empty() && self.locals.last().unwrap().depth > self.scope_depth {
            self.emit(OpCode::Pop);
            self.locals.pop();
        }
    }

    fn declare_variable(&mut self, name: String) -> Result<(), Box<dyn Error>> {
        if self.scope_depth == 0 {
            // It's a global variable
            self.emit(OpCode::DefineGlobal(name));
        } else {
            // It's a local variable
            // Check for variable redeclaration in the same scope
            for i in (0..self.locals.len()).rev() {
                let local = &self.locals[i];
                if local.depth < self.scope_depth {
                    break;
                }

                if local.name == name {
                    return Err(Box::new(BytecodeGeneratorError {
                        message: format!("Variable '{}' already declared in this scope", name),
                    }));
                }
            }

            self.add_local(name);
        }

        Ok(())
    }

    fn add_local(&mut self, name: String) {
        self.locals.push(LocalVariable {
            name,
            depth: self.scope_depth,
        });
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }

        None
    }
}
