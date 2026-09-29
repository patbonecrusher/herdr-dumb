use super::responses::{encode_error, encode_success};
use crate::api::schema::{PaneLinkActivateParams, ResponseResult};
use crate::app::App;
impl App {
    fn read_checked_pane_link<T>(
        &self,
        id: &str,
        params: &PaneLinkActivateParams,
        operation: &str,
        read: impl FnOnce(&crate::terminal::TerminalRuntime, u16, u16) -> T,
    ) -> Result<(crate::layout::PaneId, T), String> {
        let error = |code, message: &str| encode_error(id.to_owned(), code, message);
        let Some((ws_idx, pane_id)) = self.parse_pane_id(&params.pane_id) else {
            return Err(error("pane_not_found", "pane not found"));
        };
        if !self.state.pane_visible_on_active_surface(ws_idx, pane_id) {
            return Err(error("stale_target", "pane is no longer visible"));
        }
        let Some(runtime) =
            self.state
                .runtime_for_pane_in_workspace(&self.terminal_runtimes, ws_idx, pane_id)
        else {
            return Err(error("pane_not_found", "pane runtime not found"));
        };
        let current_offset = runtime
            .scroll_metrics()
            .map(|metrics| metrics.offset_from_bottom as u64);
        if params
            .offset_from_bottom
            .is_some_and(|expected| current_offset != Some(expected))
        {
            return Err(error(
                "stale_content",
                &format!("pane viewport changed before link {operation}"),
            ));
        }
        let content_revision = runtime.content_seq();
        if content_revision % 2 != 0
            || params
                .content_revision
                .is_some_and(|expected| expected != content_revision)
        {
            return Err(error(
                "stale_content",
                &format!("pane content changed before link {operation}"),
            ));
        }
        let value = read(runtime, params.col, params.viewport_row);
        if runtime.content_seq() != content_revision
            || runtime
                .scroll_metrics()
                .map(|metrics| metrics.offset_from_bottom as u64)
                != current_offset
        {
            return Err(error(
                "stale_content",
                &format!("pane content or viewport changed during link {operation}"),
            ));
        }
        Ok((pane_id, value))
    }

    pub(super) fn handle_pane_link_resolve(
        &mut self,
        id: String,
        params: PaneLinkActivateParams,
    ) -> String {
        match self.read_checked_pane_link(&id, &params, "resolution", |runtime, col, row| {
            runtime.link_regions_at(col, row, crate::app::actions::url_byte_range)
        }) {
            Ok((_, regions)) => encode_success(id, ResponseResult::PaneLinkResolved { regions }),
            Err(error) => error,
        }
    }

    pub(super) fn handle_pane_link_activate(
        &mut self,
        id: String,
        params: PaneLinkActivateParams,
    ) -> String {
        let (_, url) =
            match self.read_checked_pane_link(&id, &params, "activation", |runtime, col, row| {
                runtime
                    .link_target_at(col, row)
                    .and_then(crate::app::actions::url_from_link_target)
            }) {
                Ok(value) => value,
                Err(error) => return error,
            };
        encode_success(
            id,
            ResponseResult::PaneLinkActivated {
                url,
                handled: false,
            },
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::schema::{Method, Request, SuccessResponse};
    #[tokio::test]
    async fn pane_link_resolve_checks_staleness_without_side_effects() {
        let mut app = test_app();
        app.state.workspaces = vec![crate::workspace::Workspace::test_new("hover")];
        app.state.ensure_test_terminals();
        app.state.active = Some(0);
        let pane_id = app.state.workspaces[0].tabs[0].root_pane;
        let public_id = app.public_pane_id(0, pane_id).unwrap();
        let terminal_id = app.state.workspaces[0].tabs[0].panes[&pane_id]
            .attached_terminal_id
            .clone();
        let (runtime, mut rx) = crate::terminal::TerminalRuntime::test_with_channel(80, 4);
        runtime.test_process_pty_bytes(b"https://example.com");
        let revision = runtime.content_seq();
        app.terminal_runtimes.insert(terminal_id, runtime);
        let params = PaneLinkActivateParams {
            pane_id: public_id,
            viewport_row: 0,
            col: 1,
            content_revision: Some(revision),
            offset_from_bottom: Some(0),
        };
        let response = app.handle_api_request(Request {
            id: "hover".into(),
            method: Method::PaneLinkResolve(params.clone()),
        });
        assert!(
            matches!(response_result(&response), ResponseResult::PaneLinkResolved { regions } if regions.len() == 1)
        );
        assert!(
            rx.try_recv().is_err(),
            "resolution must not write to the PTY"
        );
        let activated = app.handle_pane_link_activate("activate".into(), params.clone());
        assert!(
            matches!(response_result(&activated), ResponseResult::PaneLinkActivated { url: Some(url), handled: false } if url == "https://example.com")
        );
        for stale in [
            PaneLinkActivateParams {
                content_revision: Some(revision + 2),
                ..params.clone()
            },
            PaneLinkActivateParams {
                offset_from_bottom: Some(1),
                ..params.clone()
            },
        ] {
            let response = app.handle_pane_link_resolve("hover".into(), stale);
            let response: serde_json::Value = serde_json::from_str(&response).unwrap();
            assert_eq!(response["error"]["code"], "stale_content");
        }
        let changed =
            app.read_checked_pane_link("hover", &params, "resolution", |runtime, _, _| {
                runtime.test_process_pty_bytes(b"changed");
            });
        let response: serde_json::Value = serde_json::from_str(&changed.unwrap_err()).unwrap();
        assert_eq!(response["error"]["code"], "stale_content");
        assert_eq!(
            response["error"]["message"],
            "pane content or viewport changed during link resolution"
        );
        assert!(rx.try_recv().is_err());

        app.state.active = None;
        let response: serde_json::Value =
            serde_json::from_str(&app.handle_pane_link_resolve("hover".into(), params)).unwrap();
        assert_eq!(response["error"]["code"], "stale_target");
    }

    fn test_app() -> App {
        let (_api_tx, api_rx) = tokio::sync::mpsc::unbounded_channel();
        App::new(
            &crate::config::Config::default(),
            crate::app::AppPolicy::TEST,
            None,
            api_rx,
            crate::api::EventHub::default(),
        )
    }

    fn response_result(response: &str) -> ResponseResult {
        serde_json::from_str::<SuccessResponse>(response)
            .expect("success response")
            .result
    }
}
