use crate::{Bytecode, Function, Imports, Instruction, Return, Value};
use anyhow::Result;

pub struct Vm {
    stack: Vec<i64>,
}

impl Vm {
    pub fn new() -> Self {
        Vm { stack: Vec::new() }
    }

    pub fn run(&mut self, bytecode: &Bytecode, name: &str, imports: &mut Imports) -> Result<Return> {
       let function = bytecode.get_function(name).ok_or(anyhow::anyhow!("Function not found"))?;
       self.execute_fn(bytecode, function, imports)
    }

    fn execute_fn(&mut self, bytecode: &Bytecode, function: &Function, imports: &mut Imports) -> Result<Return> {
        let args = function.func_type.params.iter().map(|val_type| Value { val_type: *val_type, value: self.stack.pop().unwrap() }).collect();
        match &function.kind {
            crate::FunctKind::Import { index } => {
                let result = imports.invoke_import(*index as usize, args)?;
                match result {
                    Return::Single(value) => {
                        self.stack.push(value.value);
                    }
                    Return::Multiple(values) => {
                        self.stack.extend(values.into_iter().map(|value| value.value));
                    }
                    Return::Void => {}
                }
            },
            crate::FunctKind::Definition(function_definition) => {
                let mut locals = function_definition.locals.iter().map(|local| Value { val_type: *local, value: 0 }).collect();
                for instruction in function_definition.body.iter() {
                    self.execute_instruction(instruction, &mut locals, bytecode, imports)?;
                }
            },
        }

        if function.func_type.returns.is_empty() {
            Ok(Return::Void)
        } else if function.func_type.returns.len() == 1 {
            Ok(Return::Single(Value { val_type: function.func_type.returns[0], value: self.stack.pop().unwrap() }))
        } else {
            Ok(Return::Multiple(function.func_type.returns.iter().map(|val_type| Value { val_type: *val_type, value: self.stack.pop().unwrap() }).collect()))
        }
    }

    fn execute_instruction(&mut self,  instruction: &Instruction, locals: &mut Vec<Value>, bytecode: &Bytecode, imports: &mut Imports) -> Result<()> {
        match instruction {
            Instruction::I32Add => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a+b) as i64);
            },
            Instruction::I32Sub => {
               let b = self.stack.pop().unwrap() as i32;
               let a = self.stack.pop().unwrap() as i32;
               self.stack.push((a-b) as i64);
            },
            Instruction::I32Mul => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a*b) as i64);
            },
            Instruction::I32Div => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a/b) as i64);
            },
            Instruction::I32Rem => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a%b) as i64);
            },
            Instruction::I32And => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a&b) as i64);
            },
            Instruction::I32Or => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a|b) as i64);
            },
            Instruction::I32Xor => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a^b) as i64);
            },
            Instruction::I32Shl => {
                let b = self.stack.pop().unwrap() as i32;
                let a = self.stack.pop().unwrap() as i32;
                self.stack.push((a<<b) as i64);
            },
            Instruction::I32Const(value) => {
                self.stack.push(*value as i64);
            },
            Instruction::Call(index) => {
                let function = bytecode.get_function_by_index(*index as usize).ok_or(anyhow::anyhow!("Function not found"))?;
                let result = self.execute_fn(bytecode, function, imports)?;
                match result {
                    Return::Single(value) => {
                        self.stack.push(value.value);
                    }
                    Return::Multiple(values) => {
                        self.stack.extend(values.into_iter().map(|value| value.value));
                    }
                    Return::Void => {}
                }
            }
            Instruction::LocalGet(index) => {
                self.stack.push(locals[*index as usize].value);
            },
            Instruction::LocalSet(index) => {
                locals[*index as usize].value = self.stack.pop().unwrap();
            },
            Instruction::GlobalGet(_) => todo!(),
            Instruction::GlobalSet(_) => todo!(),
            Instruction::End => {},    
            Instruction::Return => todo!(),
        }
    Ok(())
    }

}
