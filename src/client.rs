use http;

pub struct MatrixClient<'a> {
    inner: HttpClient<'a>
}

impl<'a> MatrixClient {
    pub fn new() -> Client {
        Client {}
    }
}
