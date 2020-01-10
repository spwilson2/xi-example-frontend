use crate::core::ClientToClientWriter;
use super::keyboard::Keyboard;
use super::actions::Action;
use xi_rpc::Peer;


pub struct InputController {
    keyboard: Box<dyn Keyboard>,
    view_id: String,
    front_event_writer: ClientToClientWriter,
}

impl InputController {
    pub fn new(
        keyboard: Box<dyn Keyboard>,
        client_to_client_writer: ClientToClientWriter,
    ) -> Self {
        Self {
            keyboard,
            view_id: String::new(),
            front_event_writer: client_to_client_writer,
        }
    }

    pub fn open_file(&mut self, core: &dyn Peer, file_path: &str) -> Result<(), Error> {
        let view_id = core
            .send_rpc_request("new_view", &json!({ "file_path": file_path }))
            .expect("failed to create the new view");

        self.view_id = view_id.as_str().unwrap().to_string();

        self.front_event_writer.send_rpc_notification(
            "set_path_for_view",
            &json!({
                "view_id": self.view_id,
                "path": file_path,
            }),
        );

        core.send_rpc_notification("set_theme", &json!({"theme_name": "Solarized (light)" }));
        self.front_event_writer.send_rpc_notification(
            "add_status_item",
            &json!({
                "key": "change-mode",
                "value": self.mode.to_string(),
                "alignment": "left",
            }),
        );

        Ok(())
    }

    pub fn start_keyboard_event_loop(&mut self, core: &dyn Peer) -> Result<(), Error> {
        loop {
            let key_res = self.keyboard.get_next_keystroke();

            if let Some(key) = key_res {

                action = Some(Action::InsertKeyStroke(key));
                if action.is_none() {
                    continue;
                }

                let res =
                    action
                        .unwrap()
                        .execute(&self.view_id, core, &mut self.front_event_writer);

                match res {
                    Response::Continue => continue,
                    Response::Stop => break,
                    Response::SwitchToInsertMode => self.mode = Mode::Insert,
                    Response::SwitchToNormalMode => self.mode = Mode::Normal,
                    Response::SwitchToVisualMode => self.mode = Mode::Visual,
                    Response::SwitchToActionMode => self.mode = Mode::Action,
                }

                core.send_rpc_notification(
                    "edit",
                    &json!({ "method": "collapse_selections", "view_id": self.view_id}),
                );

                self.front_event_writer.send_rpc_notification(
                    "update_status_item",
                    &json!({
                        "key": "change-mode",
                        "value": self.mode.to_string(),
                    }),
                );
            }
        }

        self.front_event_writer
            .send_rpc_notification("command", &json!({"method": "exit"}));

        Ok(())
    }
}