use serde::Serialize;

#[derive(Serialize)]
pub struct RequestParams {
    short_form: bool,
    expires: i32,
    server_data: bool,
    show_all: bool,
    show_version: bool,
    only_version: bool,
    show_server: bool,
    show_users: bool,
    show_licenses: bool,
}

pub enum Command {
    CmdLs,
}
