

pub struct EventController {
    //styles: Rc<RefCell<Box<dyn Styles>>>,
    //views: HashMap<ViewID, View>,
    //layout: Box<dyn Layout>,
    //status_bar: StatusBar,
    //current_view: String,
}

impl EventController {
    pub fn new() -> Self {
      EventController {}
    }
}

impl xi_rpc::Handler for EventController {
    type Notification = xi_rpc::RpcCall;
    type Request = xi_rpc::RpcCall;

    fn handle_notification(&mut self, ctx: &xi_rpc::RpcCtx, rpc: Self::Notification) {
        match rpc.method.as_str() {
            //"add_status_item" => self.handle_new_status_item(&rpc.params),
            //"update_status_item" => self.update_status_item(&rpc.params),
            //"plugin_started" => debug!("{}: -> {}", &rpc.method, &rpc.params),
            //"available_languages" => debug!("{}", &rpc.method),
            //"available_themes" => debug!("{}", &rpc.method),
            //"available_plugins" => debug!("{}", &rpc.method),
            //"config_changed" => debug!("{}", &rpc.method),
            //"def_style" => self.handle_style_change(&rpc.params),
            //"language_changed" => debug!("{}", &rpc.method),
            //"scroll_to" => self.handle_cursor_move(&ctx, &rpc.params),
            //"update" => self.handle_content_update(&ctx, &rpc.params),
            //"theme_changed" => debug!("{}", &rpc.method),
            //"set_path_for_view" => self.set_path_for_view(&ctx, &rpc.params),
            //"write_to_file" => self.write_to_file(&ctx, &rpc.params),
            _ => warn!("unhandled notif \"{}\" -> {}", &rpc.method, &rpc.params),
        };
    }

    fn handle_request(&mut self, _ctx: &xi_rpc::RpcCtx, rpc: Self::Request) -> Result<serde_json::Value, xi_rpc::RemoteError> {
        info!("[request] {} -> {:#?}", rpc.method, rpc.params);
        Ok(json!({}))
    }
}
