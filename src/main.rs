// External crates
use actix_cors::Cors;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::error::Error;

// Local module declarations
mod lexer;
mod parser;
mod bytecode;
mod vm;

// Use statements for convenience
use lexer::Lexer;
use parser::Parser;
use bytecode::BytecodeGenerator;
use vm::VirtualMachine;
use vm::Instruction;

// Struct to deserialize incoming JSON from frontend
#[derive(Deserialize, Serialize)]
struct CodeInput {
    source: String,       // The actual code to compile
    language: String,     // Currently unused, but kept for future use or backward compatibility
}

// Struct to serialize the output back to frontend
#[derive(Serialize)]
struct CodeOutput {
    result: String,            // Result of code execution
    bytecode: Vec<String>,     // Human-readable version of bytecode instructions
    error: Option<String>,     // Error message if something goes wrong
}

// Route handler for POST /compile
#[post("/compile")]
async fn compile(code_input: web::Json<CodeInput>) -> impl Responder {
    // Process the input code and handle result or error
    let result = process_code(&code_input.source, &code_input.language).await;
    
    match result {
        Ok((output, bytecode)) => {
            // On success, return execution result and bytecode
            HttpResponse::Ok().json(CodeOutput {
                result: output,
                bytecode,
                error: None,
            })
        },
        Err(e) => {
            // On error, return the error message
            HttpResponse::Ok().json(CodeOutput {
                result: String::new(),
                bytecode: Vec::new(),
                error: Some(format!("Error: {}", e)),
            })
        }
    }
}

// Function to process and compile the source code
async fn process_code(source: &str, _language: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    // Step 1: Lexical analysis - tokenize the input source code
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    
    // Step 2: Parsing - convert tokens into an AST
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    // Step 3: Bytecode generation - turn AST into bytecode
    let mut bytecode_gen = BytecodeGenerator::new();
    let bytecode = bytecode_gen.generate(ast)?;
    
    // Step 4: Convert bytecode to VM instructions
    let instructions: Vec<Instruction> = bytecode.iter().map(convert_to_instruction).collect();
    
    // Step 5: Execute instructions on a virtual machine
    let mut vm = VirtualMachine::new();
    let output = vm.execute(&instructions)?;
    
    // Convert each instruction into a string for debugging/display
    let bytecode_strings = instructions.iter()
        .map(|instr| format!("{:?}", instr))
        .collect();
    
    Ok((output, bytecode_strings))
}

// Convert a bytecode OpCode to a VM Instruction
fn convert_to_instruction(op: &bytecode::OpCode) -> Instruction {
    use bytecode::OpCode;
    use bytecode::Value as BytecodeValue;
    use vm::Value as VMValue;
    
    match op {
        OpCode::Constant(value) => {
            // Map bytecode constants to VM runtime values
            let vm_value = match value {
                BytecodeValue::Int(i) => VMValue::Number(*i as f64),
                BytecodeValue::Float(f) => VMValue::Number(*f),
                BytecodeValue::String(s) => VMValue::String(s.clone()),
                BytecodeValue::Bool(b) => VMValue::Boolean(*b),
                BytecodeValue::Null => VMValue::Null,
            };
            Instruction::Push(vm_value)
        },
        // Arithmetic operations
        OpCode::Add => Instruction::Add,
        OpCode::Subtract => Instruction::Subtract,
        OpCode::Multiply => Instruction::Multiply,
        OpCode::Divide => Instruction::Divide,
        OpCode::Negate => Instruction::Negate,
        
        // Comparison operations
        OpCode::Equal => Instruction::Equal,
        OpCode::NotEqual => Instruction::NotEqual,
        OpCode::LessThan => Instruction::LessThan,
        OpCode::GreaterThan => Instruction::GreaterThan,
        
        // Control flow
        OpCode::Jump(offset) => Instruction::Jump(*offset),
        OpCode::JumpIfFalse(offset) => Instruction::JumpIfFalse(*offset),
        OpCode::Return => Instruction::Return,
        
        // Function call
        OpCode::Call(arg_count) => Instruction::Call("<unknown>".to_string(), *arg_count),
        
        // Output and cleanup
        OpCode::Print => Instruction::Print,
        OpCode::Pop => Instruction::Pop,
        
        // Variable operations
        OpCode::DefineGlobal(name) => Instruction::StoreVariable(name.clone()),
        OpCode::GetGlobal(name) => Instruction::LoadVariable(name.clone()),
        OpCode::SetGlobal(name) => Instruction::StoreVariable(name.clone()),
        
        // Local variables (not fully implemented, placeholder names)
        OpCode::GetLocal(_) => Instruction::LoadVariable("<local>".to_string()),
        OpCode::SetLocal(_) => Instruction::StoreVariable("<local>".to_string()),
    }
}

// Main function to start the Actix Web server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080");
    println!("Visit http://127.0.0.1:8080 in your browser to access the compiler interface");
    
    // Create HTTP server
    HttpServer::new(|| {
        // Enable CORS for local frontend development
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(cors)
            .service(compile) // Register the /compile endpoint
            .service(fs::Files::new("/", "./").index_file("index.html")) // Serve frontend files
    })
    .bind("0.0.0.0:8080")? // Bind server to all network interfaces
    .run()
    .await
}
