use embedded_io_async::Read;
use picoserve::{
    response::{Connection, IntoResponse, ResponseWriter, StatusCode},
    ResponseSent,
};

pub struct StringExtractor(pub alloc::string::String);

/// Rejection used for [StringExtractor].
pub enum StringRejection {
    /// Error decoding the body as UTF-8
    BodyIsNotUtf8,
    /// Error deserializing Form
    BadForm,
}

impl IntoResponse for StringRejection {
    async fn write_to<R: Read, W: ResponseWriter<Error = R::Error>>(
        self,
        connection: Connection<'_, R>,
        response_writer: W,
    ) -> Result<ResponseSent, W::Error> {
        (StatusCode::BAD_REQUEST, "Not valid UTF-8")
            .write_to(connection, response_writer)
            .await
    }
}

impl<'r, State> picoserve::extract::FromRequest<'r, State> for StringExtractor {
    type Rejection = StringRejection;

    async fn from_request<R: embedded_io_async::Read>(
        _state: &'r State,
        _request_parts: picoserve::request::RequestParts<'r>,
        request_body: picoserve::request::RequestBody<'r, R>,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            core::str::from_utf8(
                request_body
                    .read_all()
                    .await
                    .map_err(|_| StringRejection::BadForm)?,
            )
            .map_err(|_| StringRejection::BodyIsNotUtf8)?
            .into(),
        ))
    }
}
