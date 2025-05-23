use std::collections::HashMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    // Stack operations
    Push(Value),
    Pop,
    Duplicate,
    
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    
    // Comparison operations
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    
    // Variable operations
    StoreVariable(String),
    LoadVariable(String),
    
    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    Call(String, usize),
    Return,
    
    // I/O operations
    Print,
    
    // End of program
    Halt,
}

#[derive(Debug)]
pub struct VirtualMachine {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    output_buffer: String,
    call_stack: Vec<usize>,
    functions: HashMap<String, usize>,
    last_popped_value: Option<Value>, // Track the last popped value
}

impl VirtualMachine {    pub fn new() -> Self {
        VirtualMachine {
            stack: Vec::new(),
            variables: HashMap::new(),
            output_buffer: String::new(),
            call_stack: Vec::new(),
            functions: HashMap::new(),
            last_popped_value: None,
        }
    }
      pub fn execute(&mut self, bytecode: &[Instruction]) -> Result<String, Box<dyn Error>> {
        self.stack.clear();
        self.variables.clear();
        self.output_buffer.clear();
        self.call_stack.clear();
        self.last_popped_value = None;
        
        // First pass: register function addresses
        for (i, instruction) in bytecode.iter().enumerate() {
            if let Instruction::StoreVariable(name) = instruction {
                if name.starts_with("fn_") {
                    self.functions.insert(name[3..].to_string(), i);
                }
            }
        }
        
        let mut ip = 0; // Instruction pointer
        
        while ip < bytecode.len() {
            match &bytecode[ip] {
                Instruction::Push(value) => {
                    self.stack.push(value.clone());
                    ip += 1;
                }                Instruction::Pop => {
                    // Pop the value off the stack but capture it first
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    
                    // Save the value in case it's from the last expression
                    self.last_popped_value = Some(value);
                    
                    ip += 1;
                }
                Instruction::Duplicate => {
                    if let Some(value) = self.stack.last() {
                        self.stack.push(value.clone());
                    } else {
                        return Err("Cannot duplicate from empty stack".into());
                    }
                    ip += 1;
                }
                Instruction::Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Number(a_val + b_val));
                        }
                        (Value::String(a_val), Value::String(b_val)) => {
                            self.stack.push(Value::String(a_val + &b_val));
                        }
                        _ => return Err("Type error in addition".into()),
                    }
                    ip += 1;
                }
                Instruction::Subtract => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Number(a_val - b_val));
                        }
                        _ => return Err("Type error in subtraction".into()),
                    }
                    ip += 1;
                }
                Instruction::Multiply => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Number(a_val * b_val));
                        }
                        _ => return Err("Type error in multiplication".into()),
                    }
                    ip += 1;
                }
                Instruction::Divide => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            if b_val == 0.0 {
                                return Err("Division by zero".into());
                            }
                            self.stack.push(Value::Number(a_val / b_val));
                        }
                        _ => return Err("Type error in division".into()),
                    }
                    ip += 1;
                }
                Instruction::Negate => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match value {
                        Value::Number(val) => {
                            self.stack.push(Value::Number(-val));
                        }
                        _ => return Err("Type error in negation".into()),
                    }
                    ip += 1;
                }
                Instruction::Equal => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (&a, &b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Boolean(a_val == b_val));
                        }
                        (Value::String(a_val), Value::String(b_val)) => {
                            self.stack.push(Value::Boolean(a_val == b_val));
                        }
                        (Value::Boolean(a_val), Value::Boolean(b_val)) => {
                            self.stack.push(Value::Boolean(a_val == b_val));
                        }
                        _ => self.stack.push(Value::Boolean(false)),
                    }
                    ip += 1;
                }
                Instruction::NotEqual => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (&a, &b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Boolean(a_val != b_val));
                        }
                        (Value::String(a_val), Value::String(b_val)) => {
                            self.stack.push(Value::Boolean(a_val != b_val));
                        }
                        (Value::Boolean(a_val), Value::Boolean(b_val)) => {
                            self.stack.push(Value::Boolean(a_val != b_val));
                        }
                        _ => self.stack.push(Value::Boolean(true)),
                    }
                    ip += 1;
                }
                Instruction::GreaterThan => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Boolean(a_val > b_val));
                        }
                        _ => return Err("Type error in greater than comparison".into()),
                    }
                    ip += 1;
                }
                Instruction::LessThan => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match (a, b) {
                        (Value::Number(a_val), Value::Number(b_val)) => {
                            self.stack.push(Value::Boolean(a_val < b_val));
                        }
                        _ => return Err("Type error in less than comparison".into()),
                    }
                    ip += 1;
                }
                Instruction::StoreVariable(name) => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    self.variables.insert(name.clone(), value);
                    ip += 1;
                }
                Instruction::LoadVariable(name) => {
                    if let Some(value) = self.variables.get(name) {
                        self.stack.push(value.clone());
                    } else {
                        return Err(format!("Undefined variable: {}", name).into());
                    }
                    ip += 1;
                }
                Instruction::Jump(address) => {
                    ip = *address;
                }
                Instruction::JumpIfFalse(address) => {
                    let condition = self.stack.pop().ok_or("Stack underflow")?;
                    
                    match condition {
                        Value::Boolean(false) => ip = *address,
                        _ => ip += 1,
                    }
                }
                Instruction::Call(func_name, _arg_count) => {
                    if let Some(&func_address) = self.functions.get(func_name) {
                        self.call_stack.push(ip + 1);
                        ip = func_address;
                    } else {
                        return Err(format!("Undefined function: {}", func_name).into());
                    }
                }
                Instruction::Return => {
                    if let Some(return_address) = self.call_stack.pop() {
                        ip = return_address;
                    } else {
                        ip += 1;
                    }
                }
                Instruction::Print => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    self.output_buffer.push_str(&format!("{}\n", value));
                    ip += 1;
                }
                Instruction::Halt => {
                    break;
                }            }
        }
          // Add the final value on the stack to the output if there is one
        if let Some(final_value) = self.stack.last() {
            // Only add a newline if we already have output and don't have a trailing one
            if !self.output_buffer.is_empty() && !self.output_buffer.ends_with('\n') {
                self.output_buffer.push('\n');
            }
            self.output_buffer.push_str(&format!("{}", final_value));
        } 
        // If nothing on the stack but we had a last popped value (likely from the last expression)
        else if let Some(last_value) = &self.last_popped_value {
            // Only add a newline if we already have output and don't have a trailing one
            if !self.output_buffer.is_empty() && !self.output_buffer.ends_with('\n') {
                self.output_buffer.push('\n');
            }
            self.output_buffer.push_str(&format!("{}", last_value));
        }
        
        Ok(self.output_buffer.clone())
    }
}