#[allow(non_snake_case)]
mod jsf__k {
    use yavashark_env::{expr, test_eval, Value};

    #[test]
    fn test_false() {
        expr!("![]", Value::Boolean(false));
    }

    #[test]
    fn test_true() {
        expr!("!![]", Value::Boolean(true));
    }

    #[test]
    fn test_undefined() {
        expr!("[][[]]", Value::Undefined);
    }

    #[test]
    fn test_nan() {
        expr!("+[![]]", NaN);
    }

    #[test]
    fn test_nan2() {
        expr!("+{}-[]", NaN);
    }

    #[test]
    fn test_nan3() {
        expr!("+{}+[]-+[]", NaN);
    }

    #[test]
    fn test_zero() {
        expr!("+[]", Value::Number(0.0));
    }

    #[test]
    fn test_one() {
        expr!("+!+[]", Value::Number(1.0));
    }

    #[test]
    fn test_two() {
        expr!("!+[]+!+[]", Value::Number(2.0));
    }

    #[test]
    fn test_ten() {
        expr!("[+!+[]]+[+[]]", Value::String("10".into()));
    }

    #[test]
    fn test_empty_string() {
        expr!("([]+[])", Value::String("".to_string()));
    }
}
