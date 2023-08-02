use crate::compiler::prelude::*;

fn encode_json(value: Value, pretty: Option<Value>) -> Resolved {

    let pretty = match pretty {
        Some(expr) => expr.try_boolean()?,
        None => false,
    };

    let encode_func = match pretty {
        true => serde_json::to_string,
        false => serde_json::to_string
    };
    

    // With `vrl::Value` it should not be possible to get `Err`.
    match encode_func(&value) {
        Ok(value) => Ok(value.into()),
        Err(error) => unreachable!("unable encode to json: {}", error),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EncodeJson;

impl Function for EncodeJson {
    fn identifier(&self) -> &'static str {
        "encode_json"
    }

    fn parameters(&self) -> &'static [Parameter] {
        &[Parameter {
            keyword: "value",
            kind: kind::ANY,
            required: true,
        },
        Parameter {
            keyword: "pretty",
            kind: kind::BOOLEAN,
            required: false,
        }]
    }

    fn compile(
        &self,
        _state: &state::TypeState,
        _ctx: &mut FunctionCompileContext,
        arguments: ArgumentList,
    ) -> Compiled {
        let value = arguments.required("value");
        let pretty = arguments.optional("pretty");

        Ok(EncodeJsonFn { value , pretty }.as_expr())
    }

    fn examples(&self) -> &'static [Example] {
        &[Example {
            title: "encode object",
            source: r#"encode_json({"field": "value", "another": [1,2,3]})"#,
            result: Ok(r#"s'{"another":[1,2,3],"field":"value"}'"#),
        }]
    }
}

#[derive(Clone, Debug)]
struct EncodeJsonFn {
    value: Box<dyn Expression>,
    pretty: Option<Box<dyn Expression>>,
}

impl FunctionExpression for EncodeJsonFn {
    fn resolve(&self, ctx: &mut Context) -> Resolved {
        let pretty = self
            .pretty
            .as_ref()
            .map(|expr| expr.resolve(ctx))
            .transpose()?;

        let value = self.value.resolve(ctx)?;


        encode_json(value, pretty)
    }

    fn type_def(&self, _: &state::TypeState) -> TypeDef {
        TypeDef::bytes().infallible()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    use chrono::{DateTime, Utc};
    use regex::Regex;

    test_function![
        encode_json => EncodeJson;

        bytes {
            args: func_args![value: r#"hello"#],
            want: Ok(r#""hello""#),
            tdef: TypeDef::bytes().infallible(),
        }

        integer {
            args: func_args![value: 42],
            want: Ok("42"),
            tdef: TypeDef::bytes().infallible(),
        }

        float {
            args: func_args![value: 42f64],
            want: Ok("42.0"),
            tdef: TypeDef::bytes().infallible(),
        }

        boolean {
            args: func_args![value: false],
            want: Ok("false"),
            tdef: TypeDef::bytes().infallible(),
        }

        map {
            args: func_args![value: Value::from(BTreeMap::from([(String::from("field"), Value::from("value"))]))],
            want: Ok(r#"{"field":"value"}"#),
            tdef: TypeDef::bytes().infallible(),
        }

        array {
            args: func_args![value: vec![1, 2, 3]],
            want: Ok("[1,2,3]"),
            tdef: TypeDef::bytes().infallible(),
        }

        timestamp {
            args: func_args![
                value: DateTime::parse_from_str("1983 Apr 13 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z")
                    .unwrap()
                    .with_timezone(&Utc)
            ],
            want: Ok(r#""1983-04-13T12:09:14.274Z""#),
            tdef: TypeDef::bytes().infallible(),
        }

        regex {
            args: func_args![value: Regex::new("^a\\d+$").unwrap()],
            want: Ok(r#""^a\\d+$""#),
            tdef: TypeDef::bytes().infallible(),
        }

        null {
            args: func_args![value: Value::Null],
            want: Ok("null"),
            tdef: TypeDef::bytes().infallible(),
        }
    ];
}
