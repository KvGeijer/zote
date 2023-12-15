use crate::compiler;
use crate::error::{RunRes, RunResTrait, RuntimeError};
use crate::value::string::ValueString;
use crate::value::{Dictionary, List, PriorityQueue, Value};

use super::templates::BuiltinTemplate;
use super::Builtin;
use std::rc::Rc;

pub fn get_builtins() -> Vec<Rc<dyn Builtin>> {
    let mut builtins: Vec<Rc<dyn Builtin>> =
        vec![Rc::new(DictNative), Rc::new(SortNative), Rc::new(SetNative)];

    builtins.new_0arg("priority_queue", || Ok(PriorityQueue::new().into()));

    builtins.new_1arg("id", Ok);

    builtins.new_1arg("str", |value| {
        Ok(ValueString::from(value.to_string()).into())
    });

    builtins.new_1arg("list", |value| value.conv_to_list().map(Value::List));

    builtins.new_1arg("pop", |collection| collection.pop());

    builtins.new_1arg("shuffle", |value| {
        Ok(value
            .to_list()
            .ok_or(RuntimeError::bare_error(format!(
                "Can only call shuffle on a List"
            )))?
            .shuffled()
            .into())
    });

    builtins.new_1arg("read", |path| {
        let kind = path.type_of();
        let str_path = path
            .to_valuestring()
            .ok_or(RuntimeError::bare_error(format!(
                "Expect a string path to 'read', but got {kind}."
            )))?
            .to_string();

        match std::fs::read_to_string(&str_path) {
            Ok(content) => Ok(ValueString::from(content).into()),
            Err(reason) => RuntimeError::error(format!("Cannot read file: {reason}")),
        }
    });

    builtins.new_1arg("len", |value| {
        value.len().map(|usize| Value::Int(usize as i64))
    });

    builtins.new_1arg("int", |value| {
        fn parse(value: Value) -> RunRes<Value> {
            let kind = value.type_of();
            match value {
                Value::Bool(false) => Ok((0i64).into()),
                Value::Bool(true) => Ok((1i64).into()),
                Value::Int(int) => Ok(int.into()),
                Value::Float(float) => Ok((float as i64).into()),
                Value::Pointer(pointer) => parse(pointer.get_clone()),
                Value::String(string) => Ok(string.parse_int()?.into()),
                Value::Nil
                | Value::Function(_)
                | Value::Closure(_)
                | Value::Native(_)
                | Value::List(_)
                | Value::PriorityQueue(_)
                | Value::Dictionary(_) => RunRes::new_err(format!("Cannot convert {kind} to int")),
            }
        }
        parse(value)
    });

    builtins.new_1arg("float", |value| {
        fn parse(value: Value) -> RunRes<Value> {
            let kind = value.type_of();
            match value {
                Value::Bool(false) => Ok((0.0).into()),
                Value::Bool(true) => Ok((1.0).into()),
                Value::Int(int) => Ok((int as f64).into()),
                Value::Float(float) => Ok(float.into()),
                Value::Pointer(pointer) => parse(pointer.get_clone()),
                Value::String(string) => Ok(string.parse_float()?.into()),
                Value::Nil
                | Value::Function(_)
                | Value::Closure(_)
                | Value::Native(_)
                | Value::List(_)
                | Value::PriorityQueue(_)
                | Value::Dictionary(_) => {
                    RunRes::new_err(format!("Cannot convert {kind} to float"))
                }
            }
        }
        parse(value)
    });

    builtins.new_1arg("to_ascii", |value| Ok(Value::Int(value.to_char()? as i64)));
    builtins.new_1arg("from_ascii", |value| {
        Ok(ValueString::from({
            let int = value.to_int()?;
            if int >= 0 && int <= 127 {
                (int as usize as u8 as char).to_string()
            } else {
                return RunRes::new_err(format!("Cannot format {int} as ascii char"));
            }
        })
        .into())
    });

    builtins.new_1arg("values", |value| {
        let kind = value.type_of();
        let dict = value.to_dict().ok_or(RuntimeError::bare_error(format!(
            "Can only call 'values' on a dictionary, but got {kind}."
        )))?;

        Ok(List::from(dict.values()).into())
    });

    builtins.new_1arg("clone", |value| Ok(value.shallowclone()));

    builtins.new_1arg("deepclone", |value| Ok(value.deepclone()));

    // Do we actually want this? Do we want types as an actual value type?
    builtins.new_1arg("type_of", |value| {
        Ok(ValueString::from(value.type_of().to_string()).into())
    });

    // this is so horrible, we must have it! However, it will be encapsulated and not able
    // to read or write to variables. Also, of course, very inefficient.
    builtins.new_1arg("eval", |value| {
        let kind = value.type_of();
        let Some(string) = value.to_valuestring() else {
            return RunRes::new_err(format!("Can only 'eval' strings, but got {kind}"));
        };

        let Some(stmts) = parser::parse("eval-native", &string.to_string()) else {
            return RunRes::new_err("failed to parse input to eval as zote code".to_owned());
        };

        let ast = semantic_analyzer::analyze_ast(&stmts);

        let Some(chunk) = compiler::compile(&ast) else {
            return RunRes::new_err("failed to compile input to eval as zote bytecode".to_owned());
        };

        let mut vm = crate::interpreter::VM::new(Rc::new(chunk));
        match vm.run(false) {
            Ok(value) => Ok(value.unwrap_or(Value::Nil)),
            Err(reason) => RunRes::new_err(format!("Error within eval function: {reason}")),
        }
    });

    builtins.new_1arg("bit_not", |value| Ok(Value::Int(!value.to_int()?)));
    builtins.new_2arg("bit_or", |x, y| Ok(Value::Int(x.to_int()? | y.to_int()?)));
    builtins.new_2arg("bit_xor", |x, y| Ok(Value::Int(x.to_int()? ^ y.to_int()?)));
    builtins.new_2arg("bit_and", |x, y| Ok(Value::Int(x.to_int()? & y.to_int()?)));
    builtins.new_2arg("bit_lshift", |x, shift| {
        Ok(Value::Int(x.to_int()? << shift.to_int()?))
    });
    builtins.new_2arg("bit_rshift", |x, shift| {
        Ok(Value::Int(x.to_int()? >> shift.to_int()?))
    });

    builtins.new_2arg("split", |value, delimiter| match value {
        Value::String(valuestring) => Ok(List::from(
            valuestring
                .split(delimiter)?
                .into_iter()
                .map(|str| Value::from(str))
                .collect::<Vec<Value>>(),
        )
        .into()),
        Value::List(list) => Ok(list.split(delimiter).into()),
        otherwise => RunRes::new_err(format!("Cannot split {}", otherwise.type_of())),
    });

    builtins.new_2arg("push", |value, collection| {
        collection.push(value.clone())?;
        Ok(value)
    });

    builtins.new_2arg("intersect", |dict1, dict2| {
        let (t1, t2) = (dict1.type_of(), dict2.type_of());
        let (Some(d1), Some(d2)) = (dict1.to_dict(), dict2.to_dict()) else {
            return RunRes::new_err(format!(
                "intersect must take two dictionaries, but got {} and {}",
                t1, t2
            ));
        };
        Ok(d1.intersect(d2.as_ref()).into())
    });

    builtins.new_2arg("union", |dict1, dict2| {
        let (t1, t2) = (dict1.type_of(), dict2.type_of());
        let (Some(d1), Some(d2)) = (dict1.to_dict(), dict2.to_dict()) else {
            return RunRes::new_err(format!(
                "union must take two dictionaries, but got {} and {}",
                t1, t2
            ));
        };
        Ok(d1.union(d2.as_ref()).into())
    });

    builtins.new_2arg("in", |value, collection| match collection {
        Value::List(list) => Ok(list.contains(&value).into()),
        Value::String(string) => string.contains_subsequence(value).map(|b| b.into()),
        Value::Dictionary(dict) => Ok(dict.contains_key(value).into()),
        otherwise => RunRes::new_err(format!(
            "Cannot use 'in' on value of type {}",
            otherwise.type_of()
        )),
    });

    builtins.new_3arg("push_pq", |value, prio, coll| {
        let typ = coll.type_of();
        if let Value::PriorityQueue(prioq) = coll {
            prioq.push(value.clone(), prio);
            Ok(value)
        } else {
            RunRes::new_err(format!(
                "Third arg to push_pq must be a priority queue, but got {typ}"
            ))
        }
    });

    builtins.new_3arg("get_or", |key, coll, or| {
        Ok(coll.safe_read_at_index(key)?.unwrap_or(or))
    });

    builtins.new_any_arg("print", |args| {
        for arg in args.iter() {
            print!("{}", arg);
        }
        println!("");
        Ok(args.get(0).cloned().unwrap_or(Value::Nil))
    });

    builtins.new_any_arg("zip", |args| {
        let colls = args
            .into_iter()
            .map(|value| {
                value.conv_to_iter().map_err(|reason| {
                    RuntimeError::bare_error(format!("{reason} Must pass iterables to 'zip'"))
                })
            })
            .collect::<Result<Vec<Value>, RuntimeError>>()?;
        let len = colls
            .iter()
            .map(|val| val.len().expect("Iter should have len"))
            .min()
            .unwrap_or(0);

        let mut zipped = vec![];
        for i in 0..len {
            let ind: Value = (i as i64).into();
            let mut level_zips = vec![];
            for coll in colls.iter() {
                // TOOD: Inefficient?
                let indexed = coll.read_at_index(ind.clone())?;
                level_zips.push(indexed);
            }
            zipped.push(List::from(level_zips).into());
        }

        Ok(List::from(zipped).into())
    });

    builtins
}

struct DictNative;
impl Builtin for DictNative {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        if let Some(value) = args.into_iter().next() {
            let kind = value.type_of();
            let list: Rc<List> = value.to_list().ok_or(RuntimeError::bare_error(format!("The function 'dict' takes a single list as argument with all its pairs, or no list, but got {kind}")))?;
            let dict: Dictionary = list.as_ref().try_into_dict()?;
            Ok(dict.into())
        } else {
            Ok(Dictionary::new().into())
        }
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [0, 1].contains(&arity)
    }

    fn name(&self) -> &str {
        "dict"
    }

    fn arity(&self) -> &str {
        "[0, 1]"
    }
}

struct SetNative;
impl Builtin for SetNative {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        if let Some(value) = args.into_iter().next() {
            let list: Rc<List> = value.conv_to_list().map_err(|reason| {
                RuntimeError::bare_error(format!(
                    "{reason} When trying to create a set from a list"
                ))
            })?;
            let dict: Dictionary = list.as_ref().try_into_set()?;
            Ok(dict.into())
        } else {
            Ok(Dictionary::new().into())
        }
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [0, 1].contains(&arity)
    }

    fn name(&self) -> &str {
        "set"
    }

    fn arity(&self) -> &str {
        "[0, 1]"
    }
}

struct SortNative;
impl Builtin for SortNative {
    fn run(&self, args: Vec<Value>) -> RunRes<Value> {
        let mut arg_iter = args.into_iter();
        let sorting = arg_iter.next().unwrap();
        let kind = sorting.type_of();
        let list = sorting.to_list().ok_or(RuntimeError::bare_error(format!(
            "Expect a list as first argument to sort, but got {kind}."
        )))?;

        let sorted = if let Some(comparator) = arg_iter.next() {
            let kind = comparator.type_of();
            let cmp = comparator
                .to_closure()
                .ok_or(RuntimeError::bare_error(format!(
                    "Expect a closure as optional second argument to sort, but got {kind}."
                )))?;
            list.sort_by(cmp.as_ref())?
        } else {
            list.sort()?
        };

        Ok(sorted.into())
    }

    fn accept_arity(&self, arity: usize) -> bool {
        [1, 2].contains(&arity)
    }

    fn name(&self) -> &str {
        "sort"
    }

    fn arity(&self) -> &str {
        "[1, 2]"
    }
}
