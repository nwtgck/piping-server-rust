pub trait OptionHeaderBuilder {
    // Add optional header
    fn option_header<K, V>(&mut self, key: K, value_opt: Option<V>) -> &mut Self
        where http::header::HeaderName: http::HttpTryFrom<K>,
              http::header::HeaderValue: http::HttpTryFrom<V>;
}

impl OptionHeaderBuilder for http::response::Builder {
    // Add optional header
    fn option_header<K, V>(&mut self, key: K, value_opt: Option<V>) -> &mut Self
        where http::header::HeaderName: http::HttpTryFrom<K>,
              http::header::HeaderValue: http::HttpTryFrom<V> {
        if let Some(value) = value_opt {
            self.header(key, value)
        } else {
            self

        }
    }
}
