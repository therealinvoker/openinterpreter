//! Filters for the stderr log.
//!
//! rmcp logs an ERROR when an MCP server answers the end-of-session DELETE
//! with 404 or 405. Stateless servers (Stripe's, for one) do that on every
//! disconnect, and the connection closes normally either way, so the line is
//! noise in the host app's logs. Other rmcp errors still pass.

use std::fmt;

use tracing::Event;
use tracing::Metadata;
use tracing::field::Field;
use tracing::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::Filter;

const SESSION_DELETE_TARGET: &str = "rmcp::transport::streamable_http_client";
const SESSION_DELETE_MESSAGE: &str = "fail to delete session";

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BenignSessionDeleteFilter;

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message.push_str(&format!("{value:?}"));
        }
    }
}

pub(crate) fn is_benign_session_delete(target: &str, message: &str) -> bool {
    target.starts_with(SESSION_DELETE_TARGET)
        && message.contains(SESSION_DELETE_MESSAGE)
        && (message.contains("HTTP 404") || message.contains("HTTP 405"))
}

impl<S> Filter<S> for BenignSessionDeleteFilter {
    fn enabled(&self, _meta: &Metadata<'_>, _cx: &Context<'_, S>) -> bool {
        true
    }

    fn event_enabled(&self, event: &Event<'_>, _cx: &Context<'_, S>) -> bool {
        if !event.metadata().target().starts_with(SESSION_DELETE_TARGET) {
            return true;
        }
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);
        !is_benign_session_delete(event.metadata().target(), &visitor.message)
    }
}

#[cfg(test)]
mod tests {
    use super::is_benign_session_delete;
    use pretty_assertions::assert_eq;

    #[test]
    fn drops_only_session_delete_404_and_405() {
        let target = "rmcp::transport::streamable_http_client";
        let msg = "fail to delete session: unexpected server response: DELETE returned HTTP 404";
        assert_eq!(is_benign_session_delete(target, msg), true);
        assert_eq!(is_benign_session_delete(target, &msg.replace("404", "405")), true);
        assert_eq!(is_benign_session_delete(target, &msg.replace("404", "500")), false);
        assert_eq!(is_benign_session_delete(target, "fail to get common stream: HTTP 404"), false);
        assert_eq!(is_benign_session_delete("other::target", msg), false);
    }
}
