use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap, HashSet},
    fmt,
    path::PathBuf,
    rc::Rc,
};

use crate::{
    ast::{
        BinaryOperator, ExportStatement, Expression, Function as FunctionAst, ObjectField,
        ObjectPatternField, Pattern, Program, Statement, StringSegment, StringTemplate,
        UseStatement,
    },
    error::{LangError, LangResult},
    lexer::Lexer,
    parser::Parser,
};

#[derive(Clone)]
pub enum Value {
    Number(i64),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Function(Rc<FunctionValue>),
    Builtin(Rc<BuiltinFunction>),
    Null,
    Unit,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(values) => write!(f, "{:?}", values),
            Value::Object(fields) => write!(f, "{:?}", fields),
            Value::Function(func) => write!(f, "<fn {}>", func.name),
            Value::Builtin(b) => write!(f, "<builtin {}>", b.name),
            Value::Null => write!(f, "null"),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::Lexer, parser::Parser};

    fn run_source(source: &str) -> LangResult<Interpreter> {
        let tokens = Lexer::new(source).lex()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;
        let mut interpreter = Interpreter::new();
        interpreter.eval_program(&program)?;
        Ok(interpreter)
    }

    #[test]
    fn assignment_and_function_call() -> LangResult<()> {
        let source = r#"
            y: 2
            f: (x) { x + 1 }
            result: f(y)
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("result")
            .expect("result should be defined");
        match value {
            Value::Number(n) => assert_eq!(n, 3),
            other => panic!("expected number 3, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn string_interpolation_with_expression() -> LangResult<()> {
        let source = r#"
            name: "Filip"
            age: 35
            sentence: "My name is <name> and next year I'll be <age + 1>"
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("sentence")
            .expect("sentence should be defined");
        match value {
            Value::String(text) => {
                assert_eq!(text, "My name is Filip and next year I'll be 36")
            }
            other => panic!("expected interpolated string, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn impure_function_allows_logging() -> LangResult<()> {
        let source = r#"
            imp!: (x) { log!(x) }
            result: imp!(42)
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("result")
            .expect("result should be defined");
        match value {
            Value::Null => Ok(()),
            other => panic!("expected null from impure function, got {:?}", other),
        }
    }

    #[test]
    fn pure_function_cannot_call_impure_builtin() {
        let source = r#"
            f: (x) { log!(x) }
            value: f(10)
        "#;
        let err = match run_source(source) {
            Ok(_) => panic!("expected runtime error for impure call"),
            Err(err) => err,
        };
        match err {
            LangError::Runtime(message, None) => {
                assert!(message.contains("Function 'f' must be declared impure"));
            }
            other => panic!("expected runtime error, got {:?}", other),
        }
    }

    #[test]
    fn composable_block_applies_functions_in_sequence() -> LangResult<()> {
        let source = r#"
            f: (x) {
                x
                increment
                increment
                identity
            }

            result: f(1)
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("result")
            .expect("result should be defined");
        match value {
            Value::Number(n) => assert_eq!(n, 3),
            other => panic!("expected number 3, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn equality_evaluates_to_boolean() -> LangResult<()> {
        let source = r#"
            truth: { 1 = 1 }
            lie: { 1 = 2 }
            same-strings: { "foo" = "foo" }
        "#;
        let interpreter = run_source(source)?;

        let truth = interpreter
            .global
            .get("truth")
            .expect("truth should be defined");
        assert!(matches!(truth, Value::Boolean(true)));

        let lie = interpreter
            .global
            .get("lie")
            .expect("lie should be defined");
        assert!(matches!(lie, Value::Boolean(false)));

        let same_strings = interpreter
            .global
            .get("same-strings")
            .expect("same-strings should be defined");
        assert!(matches!(same_strings, Value::Boolean(true)));

        Ok(())
    }

    #[test]
    fn anonymous_functions_can_be_called() -> LangResult<()> {
        let source = r#"
            truth: ((){ 1 = 1 })()
            adder: (x) { x + 1 }
            value: adder(41)
        "#;
        let interpreter = run_source(source)?;
        let truth = interpreter
            .global
            .get("truth")
            .expect("truth should be defined");
        assert!(matches!(truth, Value::Boolean(true)));

        let value = interpreter
            .global
            .get("value")
            .expect("value should be defined");
        match value {
            Value::Number(n) => assert_eq!(n, 42),
            other => panic!("expected number 42, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn core_builtins_are_available() -> LangResult<()> {
        let source = r#"
            original: identity(5)
            incremented: increment(original)
            decremented: decrement(incremented)
        "#;
        let interpreter = run_source(source)?;
        let original = interpreter
            .global
            .get("original")
            .expect("original should exist");
        match original {
            Value::Number(n) => assert_eq!(n, 5),
            other => panic!("expected number 5, got {:?}", other),
        }

        let incremented = interpreter
            .global
            .get("incremented")
            .expect("incremented should exist");
        match incremented {
            Value::Number(n) => assert_eq!(n, 6),
            other => panic!("expected number 6, got {:?}", other),
        }

        let decremented = interpreter
            .global
            .get("decremented")
            .expect("decremented should exist");
        match decremented {
            Value::Number(n) => assert_eq!(n, 5),
            other => panic!("expected number 5, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn objects_can_be_constructed() -> LangResult<()> {
        let source = r#"
            person: {
                name: "Filip",
                age: 35
            }
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("person")
            .expect("person should exist");
        match value {
            Value::Object(map) => {
                let name = map.get("name").expect("name field missing");
                assert!(matches!(name, Value::String(s) if s == "Filip"));
                let age = map.get("age").expect("age field missing");
                match age {
                    Value::Number(n) => assert_eq!(*n, 35),
                    other => panic!("expected numeric age, got {:?}", other),
                }
            }
            other => panic!("expected object value, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn lists_can_be_constructed() -> LangResult<()> {
        let source = r#"
            numbers: [1, 2, 3]
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("numbers")
            .expect("numbers should exist");
        match value {
            Value::List(values) => {
                let expected = [1, 2, 3];
                assert_eq!(values.len(), expected.len());
                for (value, expected_number) in values.iter().zip(expected.iter()) {
                    match value {
                        Value::Number(n) => assert_eq!(*n, *expected_number),
                        other => panic!("expected number, got {:?}", other),
                    }
                }
            }
            other => panic!("expected list value, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn map_transforms_list() -> LangResult<()> {
        let source = r#"
            numbers: [1, 2, 3]
            doubled: map((n) { n + n }, numbers)
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("doubled")
            .expect("doubled should exist");
        match value {
            Value::List(values) => {
                let expected = vec![Value::Number(2), Value::Number(4), Value::Number(6)];
                assert_eq!(values.len(), expected.len());
                for (actual, expected_val) in values.iter().zip(expected.iter()) {
                    assert!(
                        Interpreter::values_equal(actual, expected_val),
                        "Expected {:?}, got {:?}",
                        expected_val,
                        actual
                    );
                }
            }
            other => panic!("expected list of numbers, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn reduce_combines_list() -> LangResult<()> {
        let source = r#"
            numbers: [1, 2, 3]
            total: reduce((acc, n) { acc + n }, 0, numbers)
        "#;
        let interpreter = run_source(source)?;
        let total = interpreter.global.get("total").expect("total should exist");
        match total {
            Value::Number(n) => assert_eq!(n, 6),
            other => panic!("expected numeric sum, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn filter_keeps_matching_items() -> LangResult<()> {
        let source = r#"
            numbers: [1, 2, 3, 4]
            is-two-or-four?: (n) { (n = 2) | (n = 4) }
            filtered: filter(is-two-or-four?, numbers)
        "#;
        let interpreter = run_source(source)?;
        let filtered = interpreter
            .global
            .get("filtered")
            .expect("filtered should exist");
        match filtered {
            Value::List(values) => {
                let expected = vec![Value::Number(2), Value::Number(4)];
                assert_eq!(values.len(), expected.len());
                for (actual, expected_val) in values.iter().zip(expected.iter()) {
                    assert!(
                        Interpreter::values_equal(actual, expected_val),
                        "Expected {:?}, got {:?}",
                        expected_val,
                        actual
                    );
                }
            }
            other => panic!("expected filtered list, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn boolean_builtins_work() -> LangResult<()> {
        let source = r#"
            both: and?(true, true)
            either: or?(false, true)
        "#;
        let interpreter = run_source(source)?;
        let both = interpreter.global.get("both").expect("both should exist");
        assert!(matches!(both, Value::Boolean(true)));
        let either = interpreter
            .global
            .get("either")
            .expect("either should exist");
        assert!(matches!(either, Value::Boolean(true)));
        Ok(())
    }

    #[test]
    fn boolean_suffix_requires_boolean_return() {
        let source = r#"
            bad?: (x) { x }
            value: bad?(1)
        "#;
        let err = match run_source(source) {
            Ok(_) => panic!("expected runtime error when boolean function returns non-boolean"),
            Err(err) => err,
        };
        match err {
            LangError::Runtime(message, None) => {
                assert!(message.contains("must return a boolean value"));
            }
            other => panic!("expected runtime error, got {:?}", other),
        }
    }

    #[test]
    fn impure_suffix_without_impure_call_errors() {
        let source = r#"
            bad!: (x) { x }
        "#;
        let err = match run_source(source) {
            Ok(_) => panic!("expected runtime error for impure suffix without impure call"),
            Err(err) => err,
        };
        match err {
            LangError::Runtime(message, None) => {
                assert!(message.contains("marked impure"));
            }
            other => panic!("expected runtime error, got {:?}", other),
        }
    }

    #[test]
    fn logical_operators_require_boolean_operands() {
        let source = r#"
            value: 1 & true
        "#;
        let err = match run_source(source) {
            Ok(_) => panic!("expected runtime error for invalid logical operands"),
            Err(err) => err,
        };
        match err {
            LangError::Runtime(message, None) => {
                assert!(message.contains("must be boolean"));
            }
            other => panic!("expected runtime error, got {:?}", other),
        }
    }

    #[test]
    fn logical_operators_work() -> LangResult<()> {
        let source = r#"
            result-and: true & false
            result-or: false | true
        "#;
        let interpreter = run_source(source)?;
        let result_and = interpreter
            .global
            .get("result-and")
            .expect("result-and should exist");
        assert!(matches!(result_and, Value::Boolean(false)));
        let result_or = interpreter
            .global
            .get("result-or")
            .expect("result-or should exist");
        assert!(matches!(result_or, Value::Boolean(true)));
        Ok(())
    }

    #[test]
    fn null_literal_and_property_access() -> LangResult<()> {
        let source = r#"
            person: {
                name: "Filip"
            }

            existing: person.name
            missing: person.age
            explicit: null
        "#;
        let interpreter = run_source(source)?;

        let existing = interpreter
            .global
            .get("existing")
            .expect("existing should exist");
        assert!(matches!(existing, Value::String(ref s) if s == "Filip"));

        let missing = interpreter
            .global
            .get("missing")
            .expect("missing should exist");
        assert!(matches!(missing, Value::Null));

        let explicit = interpreter
            .global
            .get("explicit")
            .expect("explicit should exist");
        assert!(matches!(explicit, Value::Null));

        Ok(())
    }

    #[test]
    fn list_property_access_handles_indices() -> LangResult<()> {
        let source = r#"
            numbers: [10, 20, 30]
            first: numbers.0
            out-of-bounds: numbers.5
        "#;
        let interpreter = run_source(source)?;

        let first = interpreter.global.get("first").expect("first should exist");
        match first {
            Value::Number(n) => assert_eq!(n, 10),
            other => panic!("expected number, got {:?}", other),
        }

        let out_of_bounds = interpreter
            .global
            .get("out-of-bounds")
            .expect("out-of-bounds should exist");
        assert!(matches!(out_of_bounds, Value::Null));

        Ok(())
    }

    #[test]
    fn trace_builtin_preserves_pipeline_value() -> LangResult<()> {
        let source = r#"
            f!: (x) {
                x
                increment
                (value)! { trace!("hook", value) }
                increment
            }

            result: f!(1)
        "#;
        let interpreter = run_source(source)?;
        let value = interpreter
            .global
            .get("result")
            .expect("result should exist");
        match value {
            Value::Number(n) => assert_eq!(n, 3),
            other => panic!("expected number 3, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn currying_creates_partially_applied_function() -> LangResult<()> {
        let source = r#"
            add3: (x, y, z) { x + y + z }
            add1: add3(1)
            add2: add1(2)
            result: add2(3)
        "#;
        let interpreter = run_source(source)?;
        let result = interpreter
            .global
            .get("result")
            .expect("result should exist");
        match result {
            Value::Number(n) => assert_eq!(n, 6),
            other => panic!("expected number 6, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn currying_works_with_single_call() -> LangResult<()> {
        let source = r#"
            add3: (x, y, z) { x + y + z }
            result: add3(1, 2, 3)
        "#;
        let interpreter = run_source(source)?;
        let result = interpreter
            .global
            .get("result")
            .expect("result should exist");
        match result {
            Value::Number(n) => assert_eq!(n, 6),
            other => panic!("expected number 6, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn currying_works_with_two_arguments() -> LangResult<()> {
        let source = r#"
            add3: (x, y, z) { x + y + z }
            add1: add3(1, 2)
            result: add1(3)
        "#;
        let interpreter = run_source(source)?;
        let result = interpreter
            .global
            .get("result")
            .expect("result should exist");
        match result {
            Value::Number(n) => assert_eq!(n, 6),
            other => panic!("expected number 6, got {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn spread_operator_in_objects() -> LangResult<()> {
        let source = r#"
            x: { name: "Jim" }
            y: { ...x, age: 100 }
            z: { ...y, age: 75 }
        "#;
        let interpreter = run_source(source)?;

        let y = interpreter.global.get("y").expect("y should exist");
        match y {
            Value::Object(map) => {
                let name = map.get("name").expect("name should exist");
                assert!(matches!(name, Value::String(s) if s == "Jim"));
                let age = map.get("age").expect("age should exist");
                assert!(matches!(age, Value::Number(n) if *n == 100));
            }
            other => panic!("expected object, got {:?}", other),
        }

        let z = interpreter.global.get("z").expect("z should exist");
        match z {
            Value::Object(map) => {
                let name = map.get("name").expect("name should exist");
                assert!(matches!(name, Value::String(s) if s == "Jim"));
                let age = map.get("age").expect("age should exist");
                assert!(matches!(age, Value::Number(n) if *n == 75));
            }
            other => panic!("expected object, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn spread_operator_in_lists() -> LangResult<()> {
        let source = r#"
            a: [1, 2, 3]
            b: [...a, 4, 5]
            c: [0, ...b]
        "#;
        let interpreter = run_source(source)?;

        let b = interpreter.global.get("b").expect("b should exist");
        match b {
            Value::List(values) => {
                let expected = vec![
                    Value::Number(1),
                    Value::Number(2),
                    Value::Number(3),
                    Value::Number(4),
                    Value::Number(5),
                ];
                assert_eq!(values.len(), expected.len());
                for (actual, expected_val) in values.iter().zip(expected.iter()) {
                    assert!(Interpreter::values_equal(actual, expected_val));
                }
            }
            other => panic!("expected list, got {:?}", other),
        }

        let c = interpreter.global.get("c").expect("c should exist");
        match c {
            Value::List(values) => {
                let expected = vec![
                    Value::Number(0),
                    Value::Number(1),
                    Value::Number(2),
                    Value::Number(3),
                    Value::Number(4),
                    Value::Number(5),
                ];
                assert_eq!(values.len(), expected.len());
                for (actual, expected_val) in values.iter().zip(expected.iter()) {
                    assert!(Interpreter::values_equal(actual, expected_val));
                }
            }
            other => panic!("expected list, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn if_builtin_evaluates_correct_branch() -> LangResult<()> {
        let source = r#"
            result-true: if(true, () { "true" }, () { "false" })
            result-false: if(false, () { "true" }, () { "false" })
        "#;
        let interpreter = run_source(source)?;

        let result_true = interpreter
            .global
            .get("result-true")
            .expect("result-true should exist");
        assert!(matches!(result_true, Value::String(s) if s == "true"));

        let result_false = interpreter
            .global
            .get("result-false")
            .expect("result-false should exist");
        assert!(matches!(result_false, Value::String(s) if s == "false"));

        Ok(())
    }

    #[test]
    fn if_builtin_with_defined() -> LangResult<()> {
        let source = r#"
            maybe-value: 12345
            safe: if(defined?(maybe-value), () { maybe-value }, () { "No value" })
            
            missing: null
            fallback: if(defined?(missing), () { missing }, () { "No value" })
        "#;
        let interpreter = run_source(source)?;

        let safe = interpreter.global.get("safe").expect("safe should exist");
        match safe {
            Value::Number(n) => assert_eq!(n, 12345),
            other => panic!("expected number 12345, got {:?}", other),
        }

        let fallback = interpreter
            .global
            .get("fallback")
            .expect("fallback should exist");
        assert!(matches!(fallback, Value::String(s) if s == "No value"));

        Ok(())
    }

    #[test]
    fn defined_builtin_checks_null() -> LangResult<()> {
        let source = r#"
            test-null: null
            test-value: 42
            is-null-defined: defined?(test-null)
            is-value-defined: defined?(test-value)
        "#;
        let interpreter = run_source(source)?;

        let is_null_defined = interpreter
            .global
            .get("is-null-defined")
            .expect("is-null-defined should exist");
        assert!(matches!(is_null_defined, Value::Boolean(false)));

        let is_value_defined = interpreter
            .global
            .get("is-value-defined")
            .expect("is-value-defined should exist");
        assert!(matches!(is_value_defined, Value::Boolean(true)));

        Ok(())
    }

    #[test]
    fn every_builtin_checks_all_elements() -> LangResult<()> {
        let source = r#"
            numbers: [2, 2, 2]
            all-two: every?((n) { n = 2 }, numbers)
            
            mixed: [1, 2, 3]
            all-two-mixed: every?((n) { n = 2 }, mixed)
            
            empty: []
            all-empty: every?((n) { n = 1 }, empty)
        "#;
        let interpreter = run_source(source)?;

        let all_two = interpreter
            .global
            .get("all-two")
            .expect("all-two should exist");
        assert!(matches!(all_two, Value::Boolean(true)));

        let all_two_mixed = interpreter
            .global
            .get("all-two-mixed")
            .expect("all-two-mixed should exist");
        assert!(matches!(all_two_mixed, Value::Boolean(false)));

        let all_empty = interpreter
            .global
            .get("all-empty")
            .expect("all-empty should exist");
        assert!(matches!(all_empty, Value::Boolean(true)));

        Ok(())
    }

    #[test]
    fn some_builtin_checks_any_element() -> LangResult<()> {
        let source = r#"
            numbers: [1, 2, 3]
            has-two: some?((n) { n = 2 }, numbers)
            
            no-match: [1, 3, 5]
            has-two-no: some?((n) { n = 2 }, no-match)
            
            empty: []
            some-empty: some?((n) { n = 1 }, empty)
        "#;
        let interpreter = run_source(source)?;

        let has_two = interpreter
            .global
            .get("has-two")
            .expect("has-two should exist");
        assert!(matches!(has_two, Value::Boolean(true)));

        let has_two_no = interpreter
            .global
            .get("has-two-no")
            .expect("has-two-no should exist");
        assert!(matches!(has_two_no, Value::Boolean(false)));

        let some_empty = interpreter
            .global
            .get("some-empty")
            .expect("some-empty should exist");
        assert!(matches!(some_empty, Value::Boolean(false)));

        Ok(())
    }

    #[test]
    fn none_builtin_checks_no_elements() -> LangResult<()> {
        let source = r#"
            numbers: [1, 3, 5]
            no-zero: none?((n) { n = 0 }, numbers)
            
            has-zero: [1, 0, 3]
            no-zero-false: none?((n) { n = 0 }, has-zero)
            
            empty: []
            none-empty: none?((n) { n = 1 }, empty)
        "#;
        let interpreter = run_source(source)?;

        let no_zero = interpreter
            .global
            .get("no-zero")
            .expect("no-zero should exist");
        assert!(matches!(no_zero, Value::Boolean(true)));

        let no_zero_false = interpreter
            .global
            .get("no-zero-false")
            .expect("no-zero-false should exist");
        assert!(matches!(no_zero_false, Value::Boolean(false)));

        let none_empty = interpreter
            .global
            .get("none-empty")
            .expect("none-empty should exist");
        assert!(matches!(none_empty, Value::Boolean(true)));

        Ok(())
    }

    #[test]
    fn for_each_builtin_iterates_list() -> LangResult<()> {
        let source = r#"
            words: ["a", "b", "c"]
            result: for-each!((word)! { log!(word) }, words)
        "#;
        let interpreter = run_source(source)?;

        let result = interpreter
            .global
            .get("result")
            .expect("result should exist");
        assert!(matches!(result, Value::Null));

        Ok(())
    }

    #[test]
    fn array_destructuring_assigns_elements() -> LangResult<()> {
        let source = r#"
            [one, two]: [1, 2, 3, 4]
        "#;
        let interpreter = run_source(source)?;

        let one = interpreter.global.get("one").expect("one should exist");
        match one {
            Value::Number(n) => assert_eq!(n, 1),
            other => panic!("expected number 1, got {:?}", other),
        }

        let two = interpreter.global.get("two").expect("two should exist");
        match two {
            Value::Number(n) => assert_eq!(n, 2),
            other => panic!("expected number 2, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn array_destructuring_with_fewer_elements() -> LangResult<()> {
        let source = r#"
            [first, second, third]: [10, 20]
        "#;
        let interpreter = run_source(source)?;

        let first = interpreter.global.get("first").expect("first should exist");
        match first {
            Value::Number(n) => assert_eq!(n, 10),
            other => panic!("expected number 10, got {:?}", other),
        }

        let second = interpreter
            .global
            .get("second")
            .expect("second should exist");
        match second {
            Value::Number(n) => assert_eq!(n, 20),
            other => panic!("expected number 20, got {:?}", other),
        }

        let third = interpreter.global.get("third").expect("third should exist");
        assert!(matches!(third, Value::Null));

        Ok(())
    }

    #[test]
    fn nested_array_destructuring() -> LangResult<()> {
        let source = r#"
            [[a, b], c]: [[1, 2], 3]
        "#;
        let interpreter = run_source(source)?;

        let a = interpreter.global.get("a").expect("a should exist");
        match a {
            Value::Number(n) => assert_eq!(n, 1),
            other => panic!("expected number 1, got {:?}", other),
        }

        let b = interpreter.global.get("b").expect("b should exist");
        match b {
            Value::Number(n) => assert_eq!(n, 2),
            other => panic!("expected number 2, got {:?}", other),
        }

        let c = interpreter.global.get("c").expect("c should exist");
        match c {
            Value::Number(n) => assert_eq!(n, 3),
            other => panic!("expected number 3, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn object_destructuring_shorthand() -> LangResult<()> {
        let source = r#"
            { name, age }: { name: "John", age: 30 }
        "#;
        let interpreter = run_source(source)?;

        let name = interpreter.global.get("name").expect("name should exist");
        assert!(matches!(name, Value::String(s) if s == "John"));

        let age = interpreter.global.get("age").expect("age should exist");
        match age {
            Value::Number(n) => assert_eq!(n, 30),
            other => panic!("expected number 30, got {:?}", other),
        }

        Ok(())
    }

    #[test]
    fn nested_object_destructuring() -> LangResult<()> {
        let source = r#"
            { name: { first-name }}: { name: { first-name: "John", last-name: "Doe" } }
        "#;
        let interpreter = run_source(source)?;

        let first_name = interpreter
            .global
            .get("first-name")
            .expect("first-name should exist");
        assert!(matches!(first_name, Value::String(s) if s == "John"));

        Ok(())
    }

    #[test]
    fn object_destructuring_missing_field() -> LangResult<()> {
        let source = r#"
            { name, age }: { name: "John" }
        "#;
        let interpreter = run_source(source)?;

        let name = interpreter.global.get("name").expect("name should exist");
        assert!(matches!(name, Value::String(s) if s == "John"));

        let age = interpreter.global.get("age").expect("age should exist");
        assert!(matches!(age, Value::Null));

        Ok(())
    }
}

pub struct FunctionValue {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expression,
    pub env: Rc<Environment>,
    pub impure: bool,
}

pub struct BuiltinFunction {
    pub name: String,
    pub impure: bool,
    pub params: Vec<String>, // Parameter names for currying support
    pub func: Rc<dyn Fn(&Interpreter, &[Value]) -> LangResult<Value>>,
}

impl Clone for FunctionValue {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            params: self.params.clone(),
            body: self.body.clone(),
            env: Rc::clone(&self.env),
            impure: self.impure,
        }
    }
}

impl Clone for BuiltinFunction {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            impure: self.impure,
            params: self.params.clone(),
            func: Rc::clone(&self.func),
        }
    }
}

#[derive(Clone)]
pub struct Environment {
    values: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new(parent: Option<Rc<Environment>>) -> Rc<Self> {
        Rc::new(Self {
            values: RefCell::new(HashMap::new()),
            parent,
        })
    }

    pub fn define(&self, name: String, value: Value) -> LangResult<()> {
        let mut values = self.values.borrow_mut();
        if values.contains_key(&name) {
            return Err(LangError::Runtime(
                format!("Cannot redefine immutable binding '{}'", name),
                None,
            ));
        }
        values.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.borrow().get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
pub enum Purity {
    Pure,
    Impure,
}

impl Purity {
    fn allow_impure(self) -> bool {
        matches!(self, Purity::Impure)
    }
}

pub struct Interpreter {
    global: Rc<Environment>,
    module_cache: RefCell<HashMap<String, Rc<Environment>>>,
    entry_point_dir: Option<PathBuf>,
    loading_modules: RefCell<HashSet<String>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Environment::new(None);
        let mut interpreter = Self {
            global,
            module_cache: RefCell::new(HashMap::new()),
            entry_point_dir: None,
            loading_modules: RefCell::new(HashSet::new()),
        };
        interpreter.install_builtins();
        interpreter
    }

    pub fn with_entry_point_dir(entry_point_dir: PathBuf) -> Self {
        let global = Environment::new(None);
        let mut interpreter = Self {
            global,
            module_cache: RefCell::new(HashMap::new()),
            entry_point_dir: Some(entry_point_dir),
            loading_modules: RefCell::new(HashSet::new()),
        };
        interpreter.install_builtins();
        interpreter
    }

    fn install_builtins(&mut self) {
        self.add_builtin(BuiltinFunction {
            name: "log!".to_string(),
            impure: true,
            params: vec!["message".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 1 {
                    return Err(LangError::Runtime(
                        "Builtin 'log!' expects exactly 1 argument".to_string(),
                        None,
                    ));
                }
                let message = interpreter.value_to_string(&args[0])?;
                println!("{}", message);
                Ok(Value::Null)
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "trace!".to_string(),
            impure: true,
            params: vec!["label".to_string(), "value".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'trace!' expects exactly 2 arguments (message, value)".to_string(),
                        None,
                    ));
                }
                let message = interpreter.value_to_string(&args[0])?;
                let value_str = interpreter.value_to_string(&args[1])?;
                println!("(trace) {}: {}", message, value_str);
                Ok(args[1].clone())
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "identity".to_string(),
            impure: false,
            params: vec!["x".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 1 {
                    return Err(LangError::Runtime(
                        "Builtin 'identity' expects exactly 1 argument".to_string(),
                        None,
                    ));
                }
                Ok(args[0].clone())
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "increment".to_string(),
            impure: false,
            params: vec!["number".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 1 {
                    return Err(LangError::Runtime(
                        "Builtin 'increment' expects exactly 1 argument".to_string(),
                        None,
                    ));
                }
                match &args[0] {
                    Value::Number(n) => Ok(Value::Number(n + 1)),
                    other => Err(LangError::Runtime(
                        format!("Builtin 'increment' expected a number, found {:?}", other),
                        None,
                    )),
                }
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "decrement".to_string(),
            impure: false,
            params: vec!["number".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 1 {
                    return Err(LangError::Runtime(
                        "Builtin 'decrement' expects exactly 1 argument".to_string(),
                        None,
                    ));
                }
                match &args[0] {
                    Value::Number(n) => Ok(Value::Number(n - 1)),
                    other => Err(LangError::Runtime(
                        format!("Builtin 'decrement' expected a number, found {:?}", other),
                        None,
                    )),
                }
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "map".to_string(),
            impure: false,
            params: vec!["fn".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'map' expects 2 arguments (fn, list)".to_string(),
                        None,
                    ));
                }
                let func = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'map' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                let mut result = Vec::with_capacity(list.len());
                for item in list {
                    let mapped =
                        interpreter.call_callable(func.clone(), vec![item], Purity::Pure)?;
                    result.push(mapped);
                }
                Ok(Value::List(result))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "reduce".to_string(),
            impure: false,
            params: vec!["fn".to_string(), "init".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 3 {
                    return Err(LangError::Runtime(
                        "Builtin 'reduce' expects 3 arguments (fn, init, list)".to_string(),
                        None,
                    ));
                }
                let func = args[0].clone();
                let mut acc = args[1].clone();
                let list = match &args[2] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'reduce' expected list as third argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                for item in list {
                    acc = interpreter.call_callable(func.clone(), vec![acc, item], Purity::Pure)?;
                }
                Ok(acc)
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "filter".to_string(),
            impure: false,
            params: vec!["predicate".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'filter' expects 2 arguments (predicate, list)".to_string(),
                        None,
                    ));
                }
                let predicate = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'filter' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                let mut result = Vec::new();
                for item in list {
                    let keep = interpreter.call_callable(
                        predicate.clone(),
                        vec![item.clone()],
                        Purity::Pure,
                    )?;
                    match keep {
                        Value::Boolean(true) => result.push(item),
                        Value::Boolean(false) => {}
                        other => {
                            return Err(LangError::Runtime(
                                format!("Filter predicate must return boolean, found {:?}", other),
                                None,
                            ))
                        }
                    }
                }
                Ok(Value::List(result))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "add".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'add' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Number(a), Value::Number(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'add' requires numeric operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                Ok(Value::Number(lhs + rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "subtract".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'subtract' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Number(a), Value::Number(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'subtract' requires numeric operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                Ok(Value::Number(lhs - rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "multiply".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'multiply' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Number(a), Value::Number(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'multiply' requires numeric operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                Ok(Value::Number(lhs * rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "divide".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'divide' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Number(a), Value::Number(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'divide' requires numeric operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                if rhs == 0 {
                    return Err(LangError::Runtime(
                        "Builtin 'divide' received division by zero".to_string(),
                        None,
                    ));
                }
                Ok(Value::Number(lhs / rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "and?".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'and?' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Boolean(a), Value::Boolean(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'and?' requires boolean operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                Ok(Value::Boolean(lhs && rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "or?".to_string(),
            impure: false,
            params: vec!["a".to_string(), "b".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'or?' expects exactly 2 arguments".to_string(),
                        None,
                    ));
                }
                let (lhs, rhs) = match (&args[0], &args[1]) {
                    (Value::Boolean(a), Value::Boolean(b)) => (*a, *b),
                    (a, b) => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'or?' requires boolean operands, found {:?} and {:?}",
                                a, b
                            ),
                            None,
                        ))
                    }
                };
                Ok(Value::Boolean(lhs || rhs))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "every?".to_string(),
            impure: false,
            params: vec!["predicate".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'every?' expects 2 arguments (predicate, list)".to_string(),
                        None,
                    ));
                }
                let predicate = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'every?' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                // Returns true for empty list
                for item in list {
                    let result =
                        interpreter.call_callable(predicate.clone(), vec![item], Purity::Pure)?;
                    match result {
                        Value::Boolean(true) => continue,
                        Value::Boolean(false) => return Ok(Value::Boolean(false)),
                        other => {
                            return Err(LangError::Runtime(
                                format!(
                                    "Predicate passed to 'every?' must return boolean, found {:?}",
                                    other
                                ),
                                None,
                            ))
                        }
                    }
                }
                Ok(Value::Boolean(true))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "some?".to_string(),
            impure: false,
            params: vec!["predicate".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'some?' expects 2 arguments (predicate, list)".to_string(),
                        None,
                    ));
                }
                let predicate = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'some?' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                // Returns false for empty list
                for item in list {
                    let result =
                        interpreter.call_callable(predicate.clone(), vec![item], Purity::Pure)?;
                    match result {
                        Value::Boolean(true) => return Ok(Value::Boolean(true)),
                        Value::Boolean(false) => continue,
                        other => {
                            return Err(LangError::Runtime(
                                format!(
                                    "Predicate passed to 'some?' must return boolean, found {:?}",
                                    other
                                ),
                                None,
                            ))
                        }
                    }
                }
                Ok(Value::Boolean(false))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "none?".to_string(),
            impure: false,
            params: vec!["predicate".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'none?' expects 2 arguments (predicate, list)".to_string(),
                        None,
                    ));
                }
                let predicate = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'none?' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                // Returns true for empty list
                for item in list {
                    let result =
                        interpreter.call_callable(predicate.clone(), vec![item], Purity::Pure)?;
                    match result {
                        Value::Boolean(false) => continue,
                        Value::Boolean(true) => return Ok(Value::Boolean(false)),
                        other => {
                            return Err(LangError::Runtime(
                                format!(
                                    "Predicate passed to 'none?' must return boolean, found {:?}",
                                    other
                                ),
                                None,
                            ))
                        }
                    }
                }
                Ok(Value::Boolean(true))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "defined?".to_string(),
            impure: false,
            params: vec!["value".to_string()],
            func: Rc::new(|_, args| {
                if args.len() != 1 {
                    return Err(LangError::Runtime(
                        "Builtin 'defined?' expects exactly 1 argument".to_string(),
                        None,
                    ));
                }
                Ok(Value::Boolean(!matches!(args[0], Value::Null)))
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "if".to_string(),
            impure: false,
            params: vec!["condition".to_string(), "then-fn".to_string(), "else-fn".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 3 {
                    return Err(LangError::Runtime(
                        "Builtin 'if' expects 3 arguments (condition, then-fn, else-fn)".to_string(),
                        None,
                    ));
                }
                let condition = match &args[0] {
                    Value::Boolean(b) => *b,
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'if' requires boolean condition, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                let then_fn = match &args[1] {
                    Value::Function(f) => f.clone(),
                    Value::Builtin(_) => {
                        return Err(LangError::Runtime(
                            "Builtin 'if' requires function as second argument (then-fn)".to_string(),
                            None,
                        ))
                    }
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'if' requires function as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                let else_fn = match &args[2] {
                    Value::Function(f) => f.clone(),
                    Value::Builtin(_) => {
                        return Err(LangError::Runtime(
                            "Builtin 'if' requires function as third argument (else-fn)".to_string(),
                            None,
                        ))
                    }
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'if' requires function as third argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                // Check that functions take zero arguments (thunks)
                if then_fn.params.len() != 0 {
                    return Err(LangError::Runtime(
                        format!(
                            "Builtin 'if' requires zero-argument function as then-fn, found function with {} parameters",
                            then_fn.params.len()
                        ),
                        None,
                    ));
                }
                if else_fn.params.len() != 0 {
                    return Err(LangError::Runtime(
                        format!(
                            "Builtin 'if' requires zero-argument function as else-fn, found function with {} parameters",
                            else_fn.params.len()
                        ),
                        None,
                    ));
                }
                // Evaluate only the branch that matches the condition
                if condition {
                    interpreter.call_callable(Value::Function(then_fn), vec![], Purity::Pure)
                } else {
                    interpreter.call_callable(Value::Function(else_fn), vec![], Purity::Pure)
                }
            }),
        });

        self.add_builtin(BuiltinFunction {
            name: "for-each!".to_string(),
            impure: true,
            params: vec!["fn".to_string(), "list".to_string()],
            func: Rc::new(|interpreter, args| {
                if args.len() != 2 {
                    return Err(LangError::Runtime(
                        "Builtin 'for-each!' expects 2 arguments (fn, list)".to_string(),
                        None,
                    ));
                }
                let func = args[0].clone();
                let list = match &args[1] {
                    Value::List(items) => items.clone(),
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Builtin 'for-each!' expected list as second argument, found {:?}",
                                other
                            ),
                            None,
                        ))
                    }
                };
                // Verify the function is impure
                let is_impure = match &func {
                    Value::Function(f) => f.impure,
                    Value::Builtin(b) => b.impure,
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                            "Builtin 'for-each!' requires function as first argument, found {:?}",
                            other
                        ),
                            None,
                        ))
                    }
                };
                if !is_impure {
                    return Err(LangError::Runtime(
                        "Builtin 'for-each!' requires impure function (marked with '!')"
                            .to_string(),
                        None,
                    ));
                }
                // Iterate through list and call function for each element
                for item in list {
                    let _ = interpreter.call_callable(func.clone(), vec![item], Purity::Impure)?;
                }
                Ok(Value::Null)
            }),
        });
    }

    fn add_builtin(&mut self, builtin: BuiltinFunction) {
        let name = builtin.name.clone();
        self.global
            .define(name.clone(), Value::Builtin(Rc::new(builtin)))
            .unwrap_or_else(|_| panic!("failed to install builtin '{}'", name));
    }

    pub fn eval_program(&mut self, program: &Program) -> LangResult<()> {
        for statement in &program.statements {
            self.eval_statement(statement, Rc::clone(&self.global))?;
        }
        Ok(())
    }

    fn eval_statement(&self, statement: &Statement, env: Rc<Environment>) -> LangResult<()> {
        match statement {
            Statement::Assignment { pattern, expr } => {
                let value = self.eval_expression(expr, Rc::clone(&env), Purity::Impure)?;
                self.destructure_pattern(pattern, value, Rc::clone(&env))
            }
            Statement::Expression(expr) => {
                let _ = self.eval_expression(expr, Rc::clone(&env), Purity::Impure)?;
                Ok(())
            }
            Statement::Function(FunctionAst {
                name,
                params,
                body,
                impure,
            }) => {
                if *impure {
                    if Self::find_impure_call(body).is_none() {
                        return Err(LangError::Runtime(
                            format!(
                                "Function '{}' is marked impure but performs no impure operations",
                                name
                            ),
                            None,
                        ));
                    }
                } else if let Some(impure_call) = Self::find_impure_call(body) {
                    return Err(LangError::Runtime(
                        format!(
                            "Function '{}' must be declared impure (end the name with '!') to call '{}'",
                            name, impure_call
                        ),
                        None,
                    ));
                }
                let func = FunctionValue {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    env: Rc::clone(&env),
                    impure: *impure,
                };
                env.define(name.clone(), Value::Function(Rc::new(func)))
            }
            Statement::Use(use_stmt) => self.eval_use_statement(use_stmt, env),
            Statement::Export(_export_stmt) => {
                // Export statements are handled during module evaluation
                // They mark bindings for export but don't do anything at statement level
                Ok(())
            }
        }
    }

    fn destructure_pattern(
        &self,
        pattern: &Pattern,
        value: Value,
        env: Rc<Environment>,
    ) -> LangResult<()> {
        match pattern {
            Pattern::Identifier(name) => env.define(name.clone(), value),
            Pattern::List(patterns) => {
                let list = match value {
                    Value::List(items) => items,
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Cannot destructure non-list value {:?} with list pattern",
                                other
                            ),
                            None,
                        ))
                    }
                };

                // Match patterns to list elements
                for (i, pattern) in patterns.iter().enumerate() {
                    let element = if i < list.len() {
                        list[i].clone()
                    } else {
                        // If there are fewer elements than patterns, assign null
                        Value::Null
                    };
                    self.destructure_pattern(pattern, element, Rc::clone(&env))?;
                }

                Ok(())
            }
            Pattern::Object(fields) => {
                let object = match value {
                    Value::Object(map) => map,
                    other => {
                        return Err(LangError::Runtime(
                            format!(
                                "Cannot destructure non-object value {:?} with object pattern",
                                other
                            ),
                            None,
                        ))
                    }
                };

                // Match patterns to object fields
                for field in fields {
                    match field {
                        ObjectPatternField::Shorthand(name) => {
                            // Shorthand: { name } assigns name = object.name
                            let field_value =
                                object.get(name.as_str()).cloned().unwrap_or(Value::Null);
                            env.define(name.clone(), field_value)?;
                        }
                        ObjectPatternField::Field { name, pattern } => {
                            // Field with nested pattern: { name: pattern }
                            // Get the value from the object field and destructure it
                            let field_value =
                                object.get(name.as_str()).cloned().unwrap_or(Value::Null);
                            self.destructure_pattern(pattern, field_value, Rc::clone(&env))?;
                        }
                    }
                }

                Ok(())
            }
        }
    }

    fn eval_expression(
        &self,
        expr: &Expression,
        env: Rc<Environment>,
        purity: Purity,
    ) -> LangResult<Value> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(template) => {
                let value = self.eval_string_template(template, env, purity)?;
                Ok(Value::String(value))
            }
            Expression::Boolean(value) => Ok(Value::Boolean(*value)),
            Expression::Null => Ok(Value::Null),
            Expression::Block(expressions) => self.eval_block(expressions, env, purity),
            Expression::Lambda {
                params,
                body,
                impure,
            } => {
                // Validate impure notation - same rules as named functions
                if *impure {
                    if Self::find_impure_call(body.as_ref()).is_none() {
                        return Err(LangError::Runtime(
                            "Anonymous function is marked impure but performs no impure operations"
                                .to_string(),
                            None,
                        ));
                    }
                } else if let Some(impure_call) = Self::find_impure_call(body.as_ref()) {
                    return Err(LangError::Runtime(
                        format!(
                            "Anonymous function must be declared impure (use '!') to call '{}'",
                            impure_call
                        ),
                        None,
                    ));
                }
                let func = FunctionValue {
                    name: "<lambda>".to_string(),
                    params: params.clone(),
                    body: *body.clone(),
                    env: Rc::clone(&env),
                    impure: *impure,
                };
                Ok(Value::Function(Rc::new(func)))
            }
            Expression::Object(fields) => {
                let mut map = BTreeMap::new();
                for field in fields {
                    match field {
                        ObjectField::Field { name, value } => {
                            let field_value =
                                self.eval_expression(&value, Rc::clone(&env), purity)?;
                            map.insert(name.clone(), field_value);
                        }
                        ObjectField::Spread(expr) => {
                            let spread_value =
                                self.eval_expression(expr, Rc::clone(&env), purity)?;
                            match spread_value {
                                Value::Object(spread_map) => {
                                    // Spread all fields from the object
                                    for (key, val) in spread_map {
                                        map.insert(key, val);
                                    }
                                }
                                other => {
                                    return Err(LangError::Runtime(
                                        format!(
                                            "Spread operator expects an object, found {:?}",
                                            other
                                        ),
                                        None,
                                    ));
                                }
                            }
                        }
                    }
                }
                Ok(Value::Object(map))
            }
            Expression::List(elements) => {
                let mut values = Vec::with_capacity(elements.len());
                for element in elements {
                    match element {
                        Expression::Spread(expr) => {
                            let spread_value =
                                self.eval_expression(expr, Rc::clone(&env), purity)?;
                            match spread_value {
                                Value::List(spread_list) => {
                                    // Spread all elements from the list
                                    values.extend(spread_list);
                                }
                                other => {
                                    return Err(LangError::Runtime(
                                        format!(
                                            "Spread operator expects a list, found {:?}",
                                            other
                                        ),
                                        None,
                                    ));
                                }
                            }
                        }
                        other => {
                            values.push(self.eval_expression(other, Rc::clone(&env), purity)?);
                        }
                    }
                }
                Ok(Value::List(values))
            }
            Expression::PropertyAccess { object, property } => {
                let target = self.eval_expression(object, Rc::clone(&env), purity)?;
                self.eval_property_access(target, property)
            }
            Expression::Spread(_) => {
                // Spread expressions are only valid inside objects and lists
                // This should not be reached in normal evaluation
                Err(LangError::Runtime(
                    "Spread operator can only be used inside object or list literals".to_string(),
                    None,
                ))
            }
            Expression::Identifier(name) => env.get(name).ok_or_else(|| {
                LangError::Runtime(format!("Undefined identifier '{}'", name), None)
            }),
            Expression::Call { callee, args } => {
                let callee_value =
                    self.eval_expression(callee.as_ref(), Rc::clone(&env), purity)?;
                let evaluated_args = args
                    .iter()
                    .map(|arg| self.eval_expression(arg, Rc::clone(&env), purity))
                    .collect::<LangResult<Vec<_>>>()?;
                self.call_callable(callee_value, evaluated_args, purity)
            }
            Expression::Binary { left, op, right } => {
                let left_value = self.eval_expression(left, Rc::clone(&env), purity)?;
                let right_value = self.eval_expression(right, env, purity)?;
                self.eval_binary(op, left_value, right_value)
            }
        }
    }

    fn eval_block(
        &self,
        expressions: &[Expression],
        env: Rc<Environment>,
        purity: Purity,
    ) -> LangResult<Value> {
        let mut iter = expressions.iter();
        let first = match iter.next() {
            Some(expr) => expr,
            None => return Ok(Value::Unit),
        };

        let mut current = self.eval_expression(first, Rc::clone(&env), purity)?;

        for expr in iter {
            let value = self.eval_expression(expr, Rc::clone(&env), purity)?;
            current = match value {
                Value::Function(func) => {
                    let mut args = Vec::with_capacity(1);
                    args.push(current);
                    self.call_callable(Value::Function(Rc::clone(&func)), args, purity)?
                }
                Value::Builtin(builtin) => {
                    let mut args = Vec::with_capacity(1);
                    args.push(current);
                    self.call_callable(Value::Builtin(Rc::clone(&builtin)), args, purity)?
                }
                other => other,
            };
        }

        Ok(current)
    }

    fn eval_string_template(
        &self,
        template: &StringTemplate,
        env: Rc<Environment>,
        purity: Purity,
    ) -> LangResult<String> {
        let mut result = String::new();
        for segment in &template.segments {
            match segment {
                StringSegment::Literal(text) => result.push_str(text),
                StringSegment::Expr(expr) => {
                    let value = self.eval_expression(expr, Rc::clone(&env), purity)?;
                    let text = self.value_to_string(&value)?;
                    result.push_str(&text);
                }
            }
        }
        Ok(result)
    }

    fn eval_binary(&self, op: &BinaryOperator, left: Value, right: Value) -> LangResult<Value> {
        match op {
            BinaryOperator::Add => self.eval_addition(left, right),
            BinaryOperator::Sub => {
                let (l, r) = self.expect_numbers("subtraction", left, right)?;
                Ok(Value::Number(l - r))
            }
            BinaryOperator::Mul => {
                let (l, r) = self.expect_numbers("multiplication", left, right)?;
                Ok(Value::Number(l * r))
            }
            BinaryOperator::Div => {
                let (l, r) = self.expect_numbers("division", left, right)?;
                if r == 0 {
                    Err(LangError::Runtime("Division by zero".to_string(), None))
                } else {
                    Ok(Value::Number(l / r))
                }
            }
            BinaryOperator::Eq => self.eval_equality(left, right),
            BinaryOperator::NotEq => {
                let result = !Self::values_equal(&left, &right);
                Ok(Value::Boolean(result))
            }
            BinaryOperator::LessThan => self.eval_comparison(left, right, |l, r| l < r),
            BinaryOperator::LessThanEq => self.eval_comparison(left, right, |l, r| l <= r),
            BinaryOperator::GreaterThan => self.eval_comparison(left, right, |l, r| l > r),
            BinaryOperator::GreaterThanEq => self.eval_comparison(left, right, |l, r| l >= r),
            BinaryOperator::And => self.eval_logical("and", left, right, true),
            BinaryOperator::Or => self.eval_logical("or", left, right, false),
        }
    }

    fn expect_numbers(&self, msg: &str, left: Value, right: Value) -> LangResult<(i64, i64)> {
        let l = match left {
            Value::Number(n) => n,
            other => {
                return Err(LangError::Runtime(
                    format!(
                        "Left operand of {} must be a number, found {:?}",
                        msg, other
                    ),
                    None,
                ))
            }
        };
        let r = match right {
            Value::Number(n) => n,
            other => {
                return Err(LangError::Runtime(
                    format!(
                        "Right operand of {} must be a number, found {:?}",
                        msg, other
                    ),
                    None,
                ))
            }
        };
        Ok((l, r))
    }

    fn eval_addition(&self, left: Value, right: Value) -> LangResult<Value> {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            (left, right) => Err(LangError::Runtime(
                format!(
                    "Addition requires numeric operands, found {:?} and {:?}",
                    left, right
                ),
                None,
            )),
        }
    }

    fn eval_equality(&self, left: Value, right: Value) -> LangResult<Value> {
        let result = Self::values_equal(&left, &right);
        Ok(Value::Boolean(result))
    }

    fn eval_comparison<F>(&self, left: Value, right: Value, cmp: F) -> LangResult<Value>
    where
        F: FnOnce(i64, i64) -> bool,
    {
        let (l, r) = self.expect_numbers("comparison", left, right)?;
        Ok(Value::Boolean(cmp(l, r)))
    }

    fn eval_logical(
        &self,
        op_name: &str,
        left: Value,
        right: Value,
        is_and: bool,
    ) -> LangResult<Value> {
        let l = match left {
            Value::Boolean(b) => b,
            other => {
                return Err(LangError::Runtime(
                    format!(
                        "Left operand of {} must be boolean, found {:?}",
                        op_name, other
                    ),
                    None,
                ))
            }
        };
        let r = match right {
            Value::Boolean(b) => b,
            other => {
                return Err(LangError::Runtime(
                    format!(
                        "Right operand of {} must be boolean, found {:?}",
                        op_name, other
                    ),
                    None,
                ))
            }
        };

        Ok(Value::Boolean(if is_and { l && r } else { l || r }))
    }

    fn eval_property_access(&self, target: Value, property: &str) -> LangResult<Value> {
        match target {
            Value::Object(map) => Ok(map.get(property).cloned().unwrap_or(Value::Null)),
            Value::Null => Ok(Value::Null),
            Value::List(values) => {
                let index = property.parse::<usize>().map_err(|_| {
                    LangError::Runtime(
                        format!("List index '{}' must be a non-negative integer", property),
                        None,
                    )
                })?;
                if index < values.len() {
                    Ok(values[index].clone())
                } else {
                    Ok(Value::Null)
                }
            }
            other => Err(LangError::Runtime(
                format!("Cannot access property '{}' on value {:?}", property, other),
                None,
            )),
        }
    }

    fn call_callable(&self, callee: Value, args: Vec<Value>, purity: Purity) -> LangResult<Value> {
        match callee {
            Value::Function(func) => {
                // Check if this is a curried builtin function
                if let (Some(captured_args_value), Some(builtin_value)) = (
                    func.env.get("__curried_args__"),
                    func.env.get("__curried_builtin__"),
                ) {
                    // This is a curried builtin function
                    let captured_args = match captured_args_value {
                        Value::List(args) => args,
                        _ => {
                            return Err(LangError::Runtime(
                                "Internal error: invalid curried builtin state".to_string(),
                                None,
                            ));
                        }
                    };

                    let builtin = match builtin_value {
                        Value::Builtin(b) => b,
                        _ => {
                            return Err(LangError::Runtime(
                                "Internal error: invalid builtin in curried function".to_string(),
                                None,
                            ));
                        }
                    };

                    // Combine captured args with new args
                    let mut combined = captured_args;
                    combined.extend(args);

                    // Check if we have enough arguments now
                    if combined.len() < builtin.params.len() {
                        // Still not enough - create another curried function
                        let remaining_params = builtin.params[combined.len()..].to_vec();
                        let curried_env = Environment::new(None);
                        curried_env.define(
                            "__curried_builtin__".to_string(),
                            Value::Builtin(Rc::clone(&builtin)),
                        )?;
                        curried_env
                            .define("__curried_args__".to_string(), Value::List(combined))?;

                        let curried_func = FunctionValue {
                            name: format!("{} (curried)", builtin.name),
                            params: remaining_params,
                            body: Expression::Identifier("__placeholder__".to_string()),
                            env: curried_env,
                            impure: builtin.impure,
                        };

                        return Ok(Value::Function(Rc::new(curried_func)));
                    }

                    // Now we have enough arguments - call the builtin
                    if builtin.impure && !purity.allow_impure() {
                        return Err(LangError::Runtime(
                            format!(
                                "Cannot call impure builtin '{}' from pure context",
                                builtin.name
                            ),
                            None,
                        ));
                    }

                    let result = (builtin.func)(self, &combined)?;
                    if builtin.name.ends_with('?') && !matches!(result, Value::Boolean(_)) {
                        return Err(LangError::Runtime(
                            format!("Builtin '{}' must return a boolean value", builtin.name),
                            None,
                        ));
                    }
                    return Ok(result);
                }

                // Check if this is a curried function (has captured args)
                let (original_func, combined_args) = if let Some(captured_args_value) =
                    func.env.get("__curried_args__")
                {
                    // This is a curried function - combine captured args with new args
                    let captured_args = match captured_args_value {
                        Value::List(args) => args,
                        _ => {
                            return Err(LangError::Runtime(
                                "Internal error: invalid curried function state".to_string(),
                                None,
                            ));
                        }
                    };

                    let original_func_value =
                        func.env.get("__curried_original__").ok_or_else(|| {
                            LangError::Runtime(
                                "Internal error: curried function missing original".to_string(),
                                None,
                            )
                        })?;

                    let original_func = match original_func_value {
                        Value::Function(f) => f,
                        _ => {
                            return Err(LangError::Runtime(
                                "Internal error: invalid original function in curried function"
                                    .to_string(),
                                None,
                            ));
                        }
                    };

                    // Combine captured args with new args
                    let mut combined = captured_args;
                    combined.extend(args);

                    (original_func, combined)
                } else {
                    // Not a curried function - handle currying if needed
                    if args.len() < func.params.len() {
                        // Create a curried function that captures the provided arguments
                        let captured_args = args;
                        let remaining_params = func.params[captured_args.len()..].to_vec();

                        // Create an environment for the curried function that stores:
                        // - The original function
                        // - The captured arguments
                        let curried_env = Environment::new(Some(Rc::clone(&func.env)));

                        // Store the original function and captured args in the environment
                        // We'll use special names that won't conflict with user code
                        curried_env.define(
                            "__curried_original__".to_string(),
                            Value::Function(Rc::clone(&func)),
                        )?;

                        // Store captured arguments as a list in the environment
                        curried_env
                            .define("__curried_args__".to_string(), Value::List(captured_args))?;

                        // Create a curried function that captures the original function and args
                        // When called, it will combine captured args with new args and call the original
                        let curried_func = FunctionValue {
                            name: format!("{} (curried)", func.name),
                            params: remaining_params,
                            body: func.body.clone(),
                            env: curried_env,
                            impure: func.impure,
                        };

                        return Ok(Value::Function(Rc::new(curried_func)));
                    }

                    // Normal function call - use the function as-is
                    (Rc::clone(&func), args)
                };

                // If too many arguments, return an error
                if combined_args.len() > original_func.params.len() {
                    return Err(LangError::Runtime(
                        format!(
                            "Function '{}' expected {} arguments but received {}",
                            original_func.name,
                            original_func.params.len(),
                            combined_args.len()
                        ),
                        None,
                    ));
                }

                // If still not enough arguments, create another curried function
                if combined_args.len() < original_func.params.len() {
                    let captured_args = combined_args;
                    let remaining_params = original_func.params[captured_args.len()..].to_vec();

                    let curried_env = Environment::new(Some(Rc::clone(&original_func.env)));
                    curried_env.define(
                        "__curried_original__".to_string(),
                        Value::Function(Rc::clone(&original_func)),
                    )?;
                    curried_env
                        .define("__curried_args__".to_string(), Value::List(captured_args))?;

                    let curried_func = FunctionValue {
                        name: format!("{} (curried)", original_func.name),
                        params: remaining_params,
                        body: original_func.body.clone(),
                        env: curried_env,
                        impure: original_func.impure,
                    };

                    return Ok(Value::Function(Rc::new(curried_func)));
                }

                if original_func.impure && !purity.allow_impure() {
                    return Err(LangError::Runtime(
                        format!(
                            "Cannot call impure function '{}' from pure context",
                            original_func.name
                        ),
                        None,
                    ));
                }

                let call_env = Environment::new(Some(Rc::clone(&original_func.env)));
                for (param, value) in original_func.params.iter().zip(combined_args.into_iter()) {
                    call_env.define(param.clone(), value)?;
                }

                let next_purity = if original_func.impure {
                    Purity::Impure
                } else {
                    Purity::Pure
                };
                let result = self.eval_expression(&original_func.body, call_env, next_purity)?;
                if original_func.name.ends_with('?') && !matches!(result, Value::Boolean(_)) {
                    return Err(LangError::Runtime(
                        format!(
                            "Function '{}' must return a boolean value",
                            original_func.name
                        ),
                        None,
                    ));
                }
                Ok(result)
            }
            Value::Builtin(builtin) => {
                if builtin.impure && !purity.allow_impure() {
                    return Err(LangError::Runtime(
                        format!(
                            "Cannot call impure builtin '{}' from pure context",
                            builtin.name
                        ),
                        None,
                    ));
                }

                // Handle currying for builtin functions
                if args.len() < builtin.params.len() {
                    // Create a curried function that captures the provided arguments
                    let captured_args = args;
                    let remaining_params = builtin.params[captured_args.len()..].to_vec();

                    // Create an environment for the curried function
                    let curried_env = Environment::new(None);

                    // Store the original builtin and captured args
                    curried_env.define(
                        "__curried_builtin__".to_string(),
                        Value::Builtin(Rc::clone(&builtin)),
                    )?;
                    curried_env
                        .define("__curried_args__".to_string(), Value::List(captured_args))?;

                    // Create a curried function that will combine args when called
                    let curried_func = FunctionValue {
                        name: format!("{} (curried)", builtin.name),
                        params: remaining_params,
                        body: Expression::Identifier("__placeholder__".to_string()), // Will be handled specially
                        env: curried_env,
                        impure: builtin.impure,
                    };

                    return Ok(Value::Function(Rc::new(curried_func)));
                }

                // Call the builtin with all required arguments
                let result = (builtin.func)(self, &args)?;
                if builtin.name.ends_with('?') && !matches!(result, Value::Boolean(_)) {
                    return Err(LangError::Runtime(
                        format!("Builtin '{}' must return a boolean value", builtin.name),
                        None,
                    ));
                }
                Ok(result)
            }
            other => Err(LangError::Runtime(
                format!("Value '{:?}' is not callable", other),
                None,
            )),
        }
    }

    fn find_impure_call(expr: &Expression) -> Option<String> {
        match expr {
            Expression::Call { callee, args } => {
                if let Some(name) = Self::identifier_name(callee.as_ref()) {
                    if name.ends_with('!') {
                        return Some(name.to_string());
                    }
                }
                Self::find_impure_call(callee.as_ref())
                    .or_else(|| args.iter().find_map(|arg| Self::find_impure_call(arg)))
            }
            Expression::Identifier(name) => {
                if name.ends_with('!') {
                    Some(name.clone())
                } else {
                    None
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::find_impure_call(left).or_else(|| Self::find_impure_call(right))
            }
            Expression::Block(expressions) => expressions
                .iter()
                .find_map(|expr| Self::find_impure_call(expr)),
            Expression::Lambda { body, .. } => Self::find_impure_call(body.as_ref()),
            Expression::String(template) => Self::find_impure_call_in_template(template),
            Expression::Object(fields) => fields.iter().find_map(|field| match field {
                ObjectField::Field { value, .. } => Self::find_impure_call(value),
                ObjectField::Spread(expr) => Self::find_impure_call(expr),
            }),
            Expression::List(elements) => elements
                .iter()
                .find_map(|expr| Self::find_impure_call(expr)),
            Expression::Spread(expr) => Self::find_impure_call(expr.as_ref()),
            Expression::PropertyAccess { object, .. } => Self::find_impure_call(object),
            Expression::Boolean(_) | Expression::Number(_) | Expression::Null => None,
        }
    }

    fn find_impure_call_in_template(template: &StringTemplate) -> Option<String> {
        for segment in &template.segments {
            if let StringSegment::Expr(expr) = segment {
                if let Some(name) = Self::find_impure_call(expr) {
                    return Some(name);
                }
            }
        }
        None
    }

    fn identifier_name(expr: &Expression) -> Option<&str> {
        if let Expression::Identifier(name) = expr {
            Some(name.as_str())
        } else {
            None
        }
    }

    fn values_equal(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Unit, Value::Unit) => true,
            (Value::Null, Value::Null) => true,
            (Value::List(l), Value::List(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                l.iter()
                    .zip(r.iter())
                    .all(|(lv, rv)| Self::values_equal(lv, rv))
            }
            (Value::Object(l), Value::Object(r)) => {
                if l.len() != r.len() {
                    return false;
                }
                l.iter().all(|(key, lv)| {
                    r.get(key)
                        .map(|rv| Self::values_equal(lv, rv))
                        .unwrap_or(false)
                })
            }
            (Value::Function(l), Value::Function(r)) => Rc::ptr_eq(l, r),
            (Value::Builtin(l), Value::Builtin(r)) => Rc::ptr_eq(l, r),
            _ => false,
        }
    }

    fn eval_use_statement(&self, use_stmt: &UseStatement, env: Rc<Environment>) -> LangResult<()> {
        let module_path = match use_stmt {
            UseStatement::Single { module_path, .. } => module_path,
            UseStatement::Namespace { module_path, .. } => module_path,
            UseStatement::Selective { module_path, .. } => module_path,
        };

        let module_env = self.load_module(module_path)?;

        match use_stmt {
            UseStatement::Single { name, .. } => {
                let value = module_env.get(name).ok_or_else(|| {
                    LangError::Runtime(
                        format!("Module '{}' does not export '{}'", module_path, name),
                        None,
                    )
                })?;
                env.define(name.clone(), value)
            }
            UseStatement::Namespace { alias, .. } => {
                // Create an object with all exported values
                let mut exports = BTreeMap::new();
                let module_values = module_env.values.borrow();
                for (key, value) in module_values.iter() {
                    exports.insert(key.clone(), value.clone());
                }
                env.define(alias.clone(), Value::Object(exports))
            }
            UseStatement::Selective { names, .. } => {
                for name in names {
                    let value = module_env.get(name).ok_or_else(|| {
                        LangError::Runtime(
                            format!("Module '{}' does not export '{}'", module_path, name),
                            None,
                        )
                    })?;
                    env.define(name.clone(), value)?;
                }
                Ok(())
            }
        }
    }

    fn load_module(&self, module_path: &str) -> LangResult<Rc<Environment>> {
        // Check cache first
        {
            let cache = self.module_cache.borrow();
            if let Some(cached_env) = cache.get(module_path) {
                return Ok(Rc::clone(cached_env));
            }
        }

        // Check for cycles
        {
            let loading = self.loading_modules.borrow();
            if loading.contains(module_path) {
                return Err(LangError::Runtime(
                    format!("Import cycle detected involving module '{}'", module_path),
                    None,
                ));
            }
        }

        // Mark as loading
        {
            let mut loading = self.loading_modules.borrow_mut();
            loading.insert(module_path.to_string());
        }

        // Resolve file path
        let file_path = self.resolve_module_path(module_path)?;

        // Read and parse the module
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            LangError::Runtime(
                format!(
                    "Failed to read module '{}' (resolved to '{}'): {}",
                    module_path,
                    file_path.display(),
                    e
                ),
                None,
            )
        })?;

        let tokens = Lexer::with_source_and_file(&source, source.clone(), file_path.clone())
            .lex()
            .map_err(|e| {
                LangError::Runtime(
                    format!("Failed to lex module '{}': {}", module_path, e),
                    None,
                )
            })?;

        let mut parser = Parser::with_source_and_file(tokens, source.clone(), file_path.clone());
        let program = parser.parse_program().map_err(|e| {
            LangError::Runtime(
                format!("Failed to parse module '{}': {}", module_path, e),
                None,
            )
        })?;

        // Create module environment
        let module_env = Environment::new(None);

        // Track exports
        let mut exports = HashSet::new();

        // Evaluate module statements
        for statement in &program.statements {
            match statement {
                Statement::Export(ExportStatement { name }) => {
                    exports.insert(name.clone());
                }
                _ => {
                    self.eval_statement(statement, Rc::clone(&module_env))?;
                }
            }
        }

        // Verify all exports exist
        let module_values = module_env.values.borrow();
        for export_name in &exports {
            if !module_values.contains_key(export_name) {
                return Err(LangError::Runtime(
                    format!(
                        "Module '{}' exports '{}' but it is not defined",
                        module_path, export_name
                    ),
                    None,
                ));
            }
        }

        // Create export-only environment
        let export_env = Environment::new(None);
        {
            let mut export_values = export_env.values.borrow_mut();
            for export_name in &exports {
                if let Some(value) = module_values.get(export_name) {
                    export_values.insert(export_name.clone(), value.clone());
                }
            }
        }

        // Remove from loading set
        {
            let mut loading = self.loading_modules.borrow_mut();
            loading.remove(module_path);
        }

        // Cache and return
        {
            let mut cache = self.module_cache.borrow_mut();
            cache.insert(module_path.to_string(), Rc::clone(&export_env));
        }

        Ok(export_env)
    }

    fn resolve_module_path(&self, module_path: &str) -> LangResult<PathBuf> {
        let base_dir = self
            .entry_point_dir
            .as_ref()
            .ok_or_else(|| {
                LangError::Runtime(
                    "Module imports require entry point directory to be set".to_string(),
                    None,
                )
            })?
            .clone();

        let mut path = base_dir.join(module_path);
        path.set_extension("fip");

        if !path.exists() {
            return Err(LangError::Runtime(
                format!(
                    "Module file not found: {} (resolved from '{}')",
                    path.display(),
                    module_path
                ),
                None,
            ));
        }

        Ok(path)
    }

    fn value_to_string(&self, value: &Value) -> LangResult<String> {
        match value {
            Value::Number(n) => Ok(n.to_string()),
            Value::String(s) => Ok(s.clone()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::List(elements) => {
                let mut parts = Vec::with_capacity(elements.len());
                for element in elements {
                    parts.push(self.value_to_string(element)?);
                }
                Ok(format!("[{}]", parts.join(", ")))
            }
            Value::Object(fields) => {
                let mut parts = Vec::with_capacity(fields.len());
                for (key, value) in fields {
                    parts.push(format!("{}: {}", key, self.value_to_string(value)?));
                }
                Ok(format!("{{{}}}", parts.join(", ")))
            }
            Value::Null => Ok("null".to_string()),
            Value::Unit => Ok("()".to_string()),
            Value::Function(func) => Ok(format!("<fn {}>", func.name)),
            Value::Builtin(builtin) => Ok(format!("<builtin {}>", builtin.name)),
        }
    }
}
