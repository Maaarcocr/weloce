use std::collections::HashMap;
use wasmparser::Parser;
use anyhow::Result;

mod vm;

pub use vm::Vm;

// Value Types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

impl From<wasmparser::ValType> for ValType {
    fn from(val_type: wasmparser::ValType) -> Self {
        match val_type {
            wasmparser::ValType::I32 => ValType::I32,
            wasmparser::ValType::I64 => ValType::I64,
            wasmparser::ValType::F32 => ValType::F32,
            wasmparser::ValType::F64 => ValType::F64,
            _ => unimplemented!("Unsupported value type"),
        }
    }
}

// Instructions
#[derive(Debug)]
pub(crate) enum Instruction {
    I32Add,
    I32Sub,
    I32Mul,
    I32Div,
    I32Rem,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32Const(i32),
    Call(u32),
    LocalGet(u32),
    LocalSet(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    End,
    Return,
}

impl<'a> From<wasmparser::Operator<'a>> for Instruction {
    fn from(operator: wasmparser::Operator<'a>) -> Self {
        match operator {
            wasmparser::Operator::I32Add => Instruction::I32Add,
            wasmparser::Operator::I32Sub => Instruction::I32Sub,
            wasmparser::Operator::I32Mul => Instruction::I32Mul,
            wasmparser::Operator::I32Const { value } => Instruction::I32Const(value),
            wasmparser::Operator::LocalGet { local_index } => Instruction::LocalGet(local_index),
            wasmparser::Operator::LocalSet { local_index } => Instruction::LocalSet(local_index),
            wasmparser::Operator::End => Instruction::End,
            wasmparser::Operator::Return => Instruction::Return,
            wasmparser::Operator::Call{function_index} => Instruction::Call(function_index),
            _ => todo!("Operator {:?} not implemented yet", operator),
        }
    }
}

// Function Types
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncType {
    params: Vec<ValType>,
    returns: Vec<ValType>,
}

impl FuncType {
    pub fn new(params: Vec<ValType>, returns: Vec<ValType>) -> Self {
        Self { params, returns }
    }
}

impl From<wasmparser::FuncType> for FuncType {
    fn from(func_type: wasmparser::FuncType) -> Self {
        Self::new(
            func_type.params().iter().map(|&param| param.into()).collect(),
            func_type.results().iter().map(|&result| result.into()).collect()
        )
    }
}

// Function Definitions and Kinds
#[derive(Debug)]
struct FunctionDefinition {
    locals: Vec<ValType>,
    body: Vec<Instruction>,
}

impl FunctionDefinition {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            body: Vec::new(),
        }
    }
}

#[derive(Debug)]
enum FunctKind {
    Import{index: u32},
    Definition(FunctionDefinition),
}

#[derive(Debug)]
struct Function {
    func_type: FuncType,
    kind: FunctKind,
}

impl Function {
    fn new(func_type: FuncType, kind: FunctKind) -> Self {
        Self {
            func_type, 
            kind,
        }
    }

    fn add_local(&mut self, local: ValType) {
        match self.kind {
            FunctKind::Definition(ref mut function_definition) => {
                function_definition.locals.push(local);
            },
            _ => panic!("Cannot add local to import function"),
        }
    }

    fn add_instruction(&mut self, instruction: Instruction) {
        match self.kind {
            FunctKind::Definition(ref mut function_definition) => {
                function_definition.body.push(instruction);
            },
            _ => panic!("Cannot add instruction to import function"),
        }
    }
}

// Exports
#[derive(Debug)]
enum ExportKind {
    Function,
    Table,
    Memory,
    Global,
    Tag,
}

#[derive(Debug)]
struct Export {
    kind: ExportKind,
    index: u32,
}

impl From<wasmparser::ExternalKind> for ExportKind {
    fn from(kind: wasmparser::ExternalKind) -> Self {
        match kind {
            wasmparser::ExternalKind::Func => ExportKind::Function,
            wasmparser::ExternalKind::Table => ExportKind::Table,
            wasmparser::ExternalKind::Memory => ExportKind::Memory,
            wasmparser::ExternalKind::Global => ExportKind::Global,
            wasmparser::ExternalKind::Tag => ExportKind::Tag,
        }
    }
}

impl<'a> From<wasmparser::Export<'a>> for Export {
    fn from(export: wasmparser::Export<'a>) -> Self {
        Self {
            kind: export.kind.into(),
            index: export.index,
        }
    }
}

// Bytecode Builder
#[derive(Debug)]
struct BytecodeBuilder {
    function_types: Vec<FuncType>,
    functions: Vec<Function>,
    exports: Exports,
    first_function_index: Option<usize>,
    current_function_index: usize,
}

impl BytecodeBuilder {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            function_types: Vec::new(),
            exports: Exports::new(),
            first_function_index: None,
            current_function_index: 0,
        }
    }

    fn add_function_type(&mut self, func_type: FuncType) {
        self.function_types.push(func_type);
    }

    fn get_function_type(&self, index: usize) -> Option<&FuncType> {
        self.function_types.get(index)
    }

    fn add_import(&mut self, func_type: FuncType, index: u32) {
        self.functions.push(Function::new(func_type, FunctKind::Import{index}));
    }

    fn add_function(&mut self, ty_index: usize) {
        self.first_function_index = Some(self.functions.len());
        self.functions.push(Function::new(self.function_types[ty_index].clone(), FunctKind::Definition(FunctionDefinition::new())));
    }

    fn add_export(&mut self, name: String, export: Export) {
        self.exports.add_export(name, export);
    }

    fn add_local(&mut self, local: ValType) {
        let func_index = self.current_function_index + self.first_function_index.unwrap_or(0);
        self.functions[func_index].add_local(local);
    }

    fn add_instruction(&mut self, instruction: Instruction) {
        let func_index = self.current_function_index + self.first_function_index.unwrap_or(0);
        self.functions[func_index].add_instruction(instruction);
    }

    fn next_function(&mut self) {
        self.current_function_index += 1;
    }

    fn build(self) -> Bytecode {
        Bytecode {
            functions: self.functions,
            exports: self.exports,
        }
    }
}

pub struct Import {
    func_type: FuncType,
    index: u32,
}

impl Import {
    pub fn new(func_type: FuncType, index: u32) -> Self {
        Self { func_type, index }
    }
}

#[derive(Debug)]
pub struct Value {
    val_type: ValType,
    value: i64,
}

impl Value {
    pub fn new(val_type: ValType, value: i64) -> Self {
        Self { val_type, value }
    }
}

#[derive(Debug)]
pub enum Return {
    Void,
    Single(Value),
    Multiple(Vec<Value>),
}

pub struct Imports {
    imports: HashMap<(&'static str, &'static str), Import>,
    import_fns: Vec<Box<dyn FnMut(Vec<Value>) -> Result<Return>>>,
}

impl Imports {
    pub fn new() -> Self {
        Self { imports: HashMap::new(), import_fns: Vec::new() }
    }
    
    pub fn add_import(&mut self, module: &'static str, name: &'static str, params: Vec<ValType>, returns: Vec<ValType>, import_fn: Box<dyn FnMut(Vec<Value>) -> Result<Return>>) {
        self.imports.insert((module, name), Import::new(FuncType::new(params, returns), self.import_fns.len() as u32));
        self.import_fns.push(import_fn);
    }

    fn get_import<'a>(&'a self, module: &'a str, name: &'a str) -> Option<&'a Import> {
        self.imports.get(&(module, name))
    }

    fn invoke_import(&mut self, index: usize, args: Vec<Value>) -> Result<Return> {
        (self.import_fns[index])(args)
    }
}

#[derive(Debug)]
struct Exports {
    exports: HashMap<String, Export>,
}

impl Exports {
    pub fn new() -> Self {
        Self { exports: HashMap::new() }
    }

    pub fn add_export(&mut self, name: String, export: Export) {
        self.exports.insert(name, export);
    }

    pub fn get_export(&self, name: &str) -> Option<&Export> {
        self.exports.get(name)
    }
}

#[derive(Debug)]
pub struct Bytecode {
    functions: Vec<Function>,
    exports: Exports,
}

impl Bytecode {
    pub(crate) fn get_function(&self, name: &str) -> Option<&Function> {
        self.exports.get_export(name).and_then(|export| self.functions.get(export.index as usize))
    }

    pub(crate) fn get_function_by_index(&self, index: usize) -> Option<&Function> {
        self.functions.get(index)
    }
}

// Main compilation function
pub fn compile_wasm(wasm: &[u8], imports: &Imports) -> Result<Bytecode> {
    let parser = Parser::new(0);
    let mut bytecode_builder = BytecodeBuilder::new();
    for payload in parser.parse_all(wasm) {
        let payload = payload?;
        match payload {
            wasmparser::Payload::Version { num, encoding, range } => {
                println!("Version: {:?}, encoding: {:?}, range: {:?}", num, encoding, range);
            },
            wasmparser::Payload::TypeSection(section_limited) => {
                for (i, ty) in section_limited.into_iter_err_on_gc_types().enumerate() {
                    let func_type = ty?;
                    bytecode_builder.add_function_type(func_type.into());
                }
            },
            wasmparser::Payload::ImportSection(section_limited) => {
                for import in section_limited.into_iter() {
                    let import = import?;
                    match import.ty {
                        wasmparser::TypeRef::Func(index) => {
                            let func_type = bytecode_builder.get_function_type(index as usize).ok_or(anyhow::anyhow!("Invalid function type index"))?;
                            let import = imports.get_import(import.module, import.name).ok_or(anyhow::anyhow!("Import not found in HashMap"))?;
                            if *func_type != import.func_type {
                                return Err(anyhow::anyhow!("Import function type does not match declared function type"));
                            }
                            bytecode_builder.add_import(import.func_type.clone(), import.index);
                        },
                        _ => todo!(),
                    } 
                }
            },
            wasmparser::Payload::FunctionSection(section_limited) => {
                for func in section_limited.into_iter() {
                    let ty_index = func? as usize;
                    bytecode_builder.add_function(ty_index);
                }
            },
            wasmparser::Payload::TableSection(section_limited) => {
                println!("Table Section: {:?}", section_limited);
            },
            wasmparser::Payload::MemorySection(section_limited) => {
                println!("Memory Section: {:?}", section_limited);
            },
            wasmparser::Payload::TagSection(section_limited) => {
                println!("Tag Section: {:?}", section_limited);
            },
            wasmparser::Payload::GlobalSection(section_limited) => {
                println!("Global Section: {:?}", section_limited);
            },
            wasmparser::Payload::ExportSection(section_limited) => {
                for export in section_limited.into_iter() {
                    let export = export?;
                    bytecode_builder.add_export(export.name.to_string(), export.into());
                }
            },
            wasmparser::Payload::StartSection { func, range } => {
                println!("Start Section: func: {:?}, range: {:?}", func, range);
            },
            wasmparser::Payload::ElementSection(section_limited) => {
                println!("Element Section: {:?}", section_limited);
            },
            wasmparser::Payload::DataCountSection { count, range } => {
                println!("Data Count Section: count: {:?}, range: {:?}", count, range);
            },
            wasmparser::Payload::DataSection(section_limited) => {
                println!("Data Section: {:?}", section_limited);
            },
            wasmparser::Payload::CodeSectionStart { count: _, range: _, size: _ } => {},
            wasmparser::Payload::CodeSectionEntry(function_body) => {
                let locals_reader = function_body.get_locals_reader()?;
                for local in locals_reader.into_iter() {
                    let (n, local) = local?;
                    for _ in 0..n {
                        bytecode_builder.add_local(local.into());
                    }
                }
                let operators_reader = function_body.get_operators_reader()?;
                for operator in operators_reader.into_iter() {
                    let operator = operator?;
                    bytecode_builder.add_instruction(operator.into());
                }
                bytecode_builder.next_function();
            },
            wasmparser::Payload::ModuleSection { parser, unchecked_range } => {
                println!("Module Section: parser: {:?}, unchecked_range: {:?}", parser, unchecked_range);
            },
            wasmparser::Payload::InstanceSection(section_limited) => {
                println!("Instance Section: {:?}", section_limited);
            },
            wasmparser::Payload::CoreTypeSection(section_limited) => {
                println!("Core Type Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentSection { parser, unchecked_range } => {
                println!("Component Section: parser: {:?}, unchecked_range: {:?}", parser, unchecked_range);
            },
            wasmparser::Payload::ComponentInstanceSection(section_limited) => {
                println!("Component Instance Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentAliasSection(section_limited) => {
                println!("Component Alias Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentTypeSection(section_limited) => {
                println!("Component Type Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentCanonicalSection(section_limited) => {
                println!("Component Canonical Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentStartSection { start, range } => {
                println!("Component Start Section: start: {:?}, range: {:?}", start, range);
            },
            wasmparser::Payload::ComponentImportSection(section_limited) => {
                println!("Component Import Section: {:?}", section_limited);
            },
            wasmparser::Payload::ComponentExportSection(section_limited) => {
                println!("Component Export Section: {:?}", section_limited);
            },
            wasmparser::Payload::CustomSection(custom_section_reader) => {
                println!("Custom Section: {:?}", custom_section_reader);
            },
            wasmparser::Payload::UnknownSection { id, contents, range } => {
                println!("Unknown Section: id: {:?}, contents: {:?}, range: {:?}", id, contents, range);
            },
            wasmparser::Payload::End(_) => {
                println!("End of WebAssembly binary");
            },
        }
    }
    Ok(bytecode_builder.build())
}
