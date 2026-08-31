use super::HeadlessServer;
use crate::api::schema::{
    ClientPresentationPiParams, ClientPresentationPiTokens, ErrorBody, ErrorResponse,
    ResponseResult, SuccessResponse,
};

impl HeadlessServer {
    pub(super) fn handle_client_presentation_pi_api(
        &self,
        id: String,
        params: ClientPresentationPiParams,
    ) -> String {
        let requested_session = match crate::session::parse_target_name(&params.session) {
            Ok(Some(session)) => session,
            Ok(None) => crate::session::DEFAULT_SESSION_NAME.to_owned(),
            Err(_) => {
                return encode_error(
                    id,
                    "invalid_params",
                    "session must be a valid Herdr session name",
                );
            }
        };
        if requested_session != self.session_name {
            return encode_error(
                id,
                "session_mismatch",
                "the requested session is not served by this socket",
            );
        }

        let mut eligible_clients = self
            .clients
            .iter()
            .filter(|(_, client)| client.is_full_app_client() && client.writer.is_some());
        let Some((&client_id, _)) = eligible_clients.next() else {
            return encode_error(
                id,
                "no_attached_client",
                "no attached full-app presentation client is available",
            );
        };
        if eligible_clients.next().is_some() {
            return encode_error(
                id,
                "ambiguous_clients",
                "multiple attached full-app presentation clients are eligible",
            );
        }

        let tokens = self.app.state.sidebar_agents.rows_by_agent.get("pi");
        if tokens.is_some_and(|tokens| crate::config::validate_agent_sidebar_rows(tokens).is_err())
        {
            return encode_error(
                id,
                "invalid_client_presentation",
                "the live Pi presentation tokens are not canonical",
            );
        }

        encode_success(
            id,
            ResponseResult::ClientPresentationPi {
                session: self.session_name.clone(),
                client_id,
                tokens: ClientPresentationPiTokens(tokens.cloned()),
            },
        )
    }
}

fn encode_success(id: String, result: ResponseResult) -> String {
    serde_json::to_string(&SuccessResponse { id, result })
        .unwrap_or_else(|_| serialization_error_response())
}

fn encode_error(id: String, code: &str, message: &str) -> String {
    serde_json::to_string(&ErrorResponse {
        id,
        error: ErrorBody {
            code: code.to_owned(),
            message: message.to_owned(),
        },
    })
    .unwrap_or_else(|_| serialization_error_response())
}

fn serialization_error_response() -> String {
    r#"{"id":"","error":{"code":"serialization_error","message":"failed to encode API response"}}"#
        .to_owned()
}
