use protocol::{CancelRequest, CompletionRequest, JsonRpcMessage, MemoryRecordRequest};
use tracing::{debug, info, warn};

pub async fn dispatch(raw: &str) -> Option<String> {
    debug!("RECV(daemon) >>> {}", raw);

    let msg: JsonRpcMessage = match serde_json::from_str(raw) {
        Ok(m) => m,
        Err(e) => {
            warn!("Failed to parse JSON-RPC message: {}", e);
            info!("RECV_PARSE_FAIL(daemon) <<< {}", raw);
            return None;
        }
    };

    let resp = match msg {
        JsonRpcMessage::Request(req) => handle_completion(req).await,
        JsonRpcMessage::Cancel(req) => handle_cancel(req).await,
        JsonRpcMessage::Record(req) => handle_record(req).await,
    };

    if let Some(r) = &resp {
        debug!("SEND(daemon) <<< {}", r);
    }
    resp
}

async fn handle_completion(req: CompletionRequest) -> Option<String> {
    let scheduler = crate::scheduler::global();
    let result = scheduler.submit_completion(req).await;

    let response = protocol::CompletionResponse {
        jsonrpc: "2.0".to_string(),
        id: result.request_id,
        result: result.outcome,
    };

    serde_json::to_string(&response).ok()
}

async fn handle_cancel(req: CancelRequest) -> Option<String> {
    let scheduler = crate::scheduler::global();
    let result = scheduler.cancel_request(&req.params.session_id, &req.params.request_id).await;

    let response = protocol::CancelResponse {
        jsonrpc: "2.0".to_string(),
        id: req.id,
        result: protocol::CancelResult {
            cancelled: result,
        },
    };

    serde_json::to_string(&response).ok()
}

async fn handle_record(req: MemoryRecordRequest) -> Option<String> {
    let scheduler = crate::scheduler::global();
    scheduler
        .record_command(&req.params.cwd, &req.params.command)
        .await;

    Some(
        format!(
            r#"{{"jsonrpc":"2.0","id":"{}","result":{{"recorded":true}}}}"#,
            req.id
        ),
    )
}
