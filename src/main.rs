use actix_cors::Cors;
use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use actix_files as fs;
use serde::{Deserialize, Serialize};
use std::error::Error;

mod lexer;
mod parser;
mod bytecode;
mod vm;

use lexer::Lexer;
use parser::Parser;
use bytecode::BytecodeGenerator;
use vm::VirtualMachine;
use vm::Instruction;

#[derive(Deserialize, Serialize)]
struct CodeInput {
    source: String,
    language: String, // Keeping this for backward compatibility, but we'll always use "custom"
}

#[derive(Serialize)]
struct CodeOutput {
    result: String,
    bytecode: Vec<String>,
    error: Option<String>,
}

#[post("/compile")]
async fn compile(code_input: web::Json<CodeInput>) -> impl Responder {
    let result = process_code(&code_input.source, &code_input.language).await;
    
    match result {
        Ok((output, bytecode)) => {
            HttpResponse::Ok().json(CodeOutput {
                result: output,
                bytecode,
                error: None,
            })
        },
        Err(e) => {
            HttpResponse::Ok().json(CodeOutput {
                result: String::new(),
                bytecode: Vec::new(),
                error: Some(format!("Error: {}", e)),
            })
        }
    }
}

async fn process_code(source: &str, _language: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    // We're only supporting a single language, so we ignore the language parameter
    
    // 1. Lexical analysis
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    
    // 2. Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser.parse()?;
    
    // 3. Bytecode generation
    let mut bytecode_gen = BytecodeGenerator::new();
    let bytecode = bytecode_gen.generate(ast)?;
    
    // Convert OpCode to Instruction
    let instructions: Vec<Instruction> = bytecode.iter().map(convert_to_instruction).collect();
    
    // 4. VM execution
    let mut vm = VirtualMachine::new();
    let output = vm.execute(&instructions)?;
    
    // Convert bytecode to string representation for frontend display
    let bytecode_strings = instructions.iter()
        .map(|instr| format!("{:?}", instr))
        .collect();
    
    Ok((output, bytecode_strings))
}

// Converts OpCode to Instruction for the VM
fn convert_to_instruction(op: &bytecode::OpCode) -> Instruction {
    use bytecode::OpCode;
    use bytecode::Value as BytecodeValue;
    use vm::Value as VMValue;
    
    match op {
        OpCode::Constant(value) => {
            // Convert bytecode Value to VM Value
            let vm_value = match value {
                BytecodeValue::Int(i) => VMValue::Number(*i as f64),
                BytecodeValue::Float(f) => VMValue::Number(*f),
                BytecodeValue::String(s) => VMValue::String(s.clone()),
                BytecodeValue::Bool(b) => VMValue::Boolean(*b),
                BytecodeValue::Null => VMValue::Null,
            };
            Instruction::Push(vm_value)
        },
        OpCode::Add => Instruction::Add,
        OpCode::Subtract => Instruction::Subtract,
        OpCode::Multiply => Instruction::Multiply,
        OpCode::Divide => Instruction::Divide,
        OpCode::Negate => Instruction::Negate,
        OpCode::Equal => Instruction::Equal,
        OpCode::NotEqual => Instruction::NotEqual,
        OpCode::LessThan => Instruction::LessThan,
        OpCode::GreaterThan => Instruction::GreaterThan,
        OpCode::Jump(offset) => Instruction::Jump(*offset),
        OpCode::JumpIfFalse(offset) => Instruction::JumpIfFalse(*offset),
        OpCode::Call(arg_count) => Instruction::Call("<unknown>".to_string(), *arg_count),
        OpCode::Return => Instruction::Return,
        OpCode::Print => Instruction::Print,
        OpCode::Pop => Instruction::Pop,
        OpCode::DefineGlobal(name) => Instruction::StoreVariable(name.clone()),
        OpCode::GetGlobal(name) => Instruction::LoadVariable(name.clone()),
        OpCode::SetGlobal(name) => Instruction::StoreVariable(name.clone()),
        OpCode::GetLocal(_) => Instruction::LoadVariable("<local>".to_string()),
        OpCode::SetLocal(_) => Instruction::StoreVariable("<local>".to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080");
    println!("Visit http://127.0.0.1:8080 in your browser to access the compiler interface");
    
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
            
        App::new()
            .wrap(cors)
            .service(compile)
            .service(fs::Files::new("/", "./").index_file("index.html"))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}