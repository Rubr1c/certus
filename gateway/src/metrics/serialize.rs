use hyper::Method;
use serde::Serializer;

pub fn serialize_method<S: Serializer>(
    method: &Method,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(method.as_str())
}
