use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use futures::Stream;
use serde::Deserialize;
use tokio::sync::Mutex;

#[derive(Clone)]
struct Client {
    backend: Arc<dyn Backend + Send + Sync>,
}

impl Client {
    fn new() -> Self {
        Self {
            backend: Arc::new(YFinBackend::new()),
        }
    }

    async fn list_p(&self, params: Params) -> Result<Iter, Box<dyn Error>> {
        if params.symbols.is_empty() {
            return Err(Box::new(CreateArgumentError));
        }

        let symbols_str = params.symbols.join(",");
        let body = form::Values::new();
        let mut iter = form::append_to(&body, params);
        iter.params.sym = Some(symbols_str);

        let response: Response = self.backend.call("/v7/finance/quote", &body, params.context).await?;
        let results: Vec<_> = response
            .quote_response
            .result
            .into_iter()
            .map(|v| Box::new(v) as Box<dyn Any + Send>)
            .collect();

        Ok(Iter::new(results, response.quote_response.error))
    }
}

#[derive(Clone)]
struct Params {
    context: Option<Context>,
    symbols: Vec<String>,
    sym: Option<String>,
}

impl Params {
    fn new(symbols: Vec<String>) -> Self {
        Self {
            context: None,
            symbols,
            sym: None,
        }
    }
}

#[derive(Clone)]
struct Iter {
    inner: Vec<Box<dyn Any + Send>>,
    error: Option<YfinError>,
}

impl Iter {
    fn new(inner: Vec<Box<dyn Any + Send>>, error: Option<YfinError>) -> Self {
        Self { inner, error }
    }

    fn forex_pair(&self) -> Option<&ForexPair> {
        self.inner.first().and_then(|v| v.downcast_ref::<ForexPair>())
    }

    async fn next(&mut self) -> Option<Box<dyn Any + Send>> {
        self.inner.pop()
    }

    fn err(&self) -> Option<&YfinError> {
        self.error.as_ref()
    }
}

#[async_trait]
trait Backend {
    async fn call(&self, endpoint: &str, body: &form::Values, context: Option<Context>) -> Result<Response, Box<dyn Error>>;
}

struct YFinBackend;

impl YFinBackend {
    fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Backend for YFinBackend {
    async fn call(&self, endpoint: &str, body: &form::Values, context: Option<Context>) -> Result<Response, Box<dyn Error>> {
        // Implement the call to the backend here
        Ok(Response {
            quote_response: QuoteResponse {
                result: vec![],
                error: None,
            },
        })
    }
}

struct Response {
    quote_response: QuoteResponse,
}

struct QuoteResponse {
    result: Vec<ForexPair>,
    error: Option<YfinError>,
}

#[derive(Deserialize)]
struct ForexPair;

#[derive(Debug)]
struct YfinError;

#[derive(Debug)]
struct CreateArgumentError;

impl std::fmt::Display for CreateArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Argument error")
    }
}

impl Error for CreateArgumentError {}

// Placeholder for form module
mod form {
    use super::*;

    #[derive(Default)]
    pub struct Values {
        map: HashMap<String, String>,
    }

    impl Values {
        pub fn new() -> Self {
            Self {
                map: HashMap::new(),
            }
        }
    }

    pub fn append_to<T>(_values: &Values, params: T) -> T {
        params
    }
}

struct Context;
