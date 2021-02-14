use crate::{
    middleware::{Middleware, Next},
};
use crate::request::request::Request;
use crate::response::response::Response;

///  ===================== TODO =====================

const RANDOM_STRING_LEN: usize = 6;

#[derive(Debug, Clone, Default)]
pub struct RequestId;

impl RequestId {
    pub fn get(req: &Request) -> Option<&str> {
        let val = req.get_extension::<RequestIdValue>();
        val.map(|v| v.value.as_str())
    }
}

#[crate::async_trait]
impl Middleware for RequestId {
    async fn handle<'a>(&'a self, mut ctx: Request, next: Next<'a>) -> Response {
        let val = RequestIdValue::new(crate::utils::utils::gen_random_string(RANDOM_STRING_LEN));
        ctx.insert_extension(val);

        next.run(ctx).await
    }
}

#[derive(Debug, Clone, Default)]
struct RequestIdValue {
    value: String,
}

impl RequestIdValue {
    fn new(value: String) -> Self {
        RequestIdValue { value }
    }
}