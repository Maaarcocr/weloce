use std::collections::HashMap;

use weloce::{compile_wasm, FuncType, Import, Imports, Return, ValType, Value, Vm};

const WASM: &[u8] = include_bytes!("simple.wasm");

fn main() {
    let mut imports = Imports::new();
    imports.add_import("env", "get_number", vec![], vec![ValType::I32], Box::new(|args| {
        Ok(Return::Single(Value::new(ValType::I32, 42)))
    }));
    let bytecode = compile_wasm(WASM, &imports).unwrap();
    let mut vm = Vm::new();
    let result = vm.run(&bytecode, "add_five_to_imported", &mut imports).unwrap();
    println!("Result: {:?}", result);
}
