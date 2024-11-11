use std::fmt::Display;

#[derive(Clone)]
pub enum Object {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Object::String(s) => s.to_string(),
            Object::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.1}", n)
                } else {
                    format!("{}", n)
                }
            }
            Object::Boolean(b) => b.to_string(),
            Object::Nil => "nil".to_string(),
        };

        write!(f, "{}", output)
    }
}
