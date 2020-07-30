use std::{io, sync::mpsc};

use crate::model;

#[derive(Debug)]
pub struct Connection {
    comm: mpsc::Sender<Request>,
}

impl Connection {
    pub fn new(comm: mpsc::Sender<Request>) -> Self {
        Connection { comm }
    }

    pub fn query_formats(&self) {
        self.comm.send(Request::QueryFormats).unwrap();
    }

    pub fn query_controls(&self) {
        self.comm.send(Request::QueryControls).unwrap();
    }

    pub fn set_format(&self, fmt: &model::format::Format) {
        self.comm.send(Request::SetFormat(fmt.clone())).unwrap();
    }

    pub fn set_control(&self, ctrl: &model::control::Control) {
        self.comm.send(Request::SetControl(ctrl.clone())).unwrap();
    }
}

#[derive(Debug)]
pub enum Request {
    QueryFormats,
    QueryControls,
    SetFormat(model::format::Format),
    SetControl(model::control::Control),
}

#[derive(Debug)]
pub enum Response {
    QueryFormats(io::Result<Vec<model::format::Format>>),
    QueryControls(io::Result<Vec<model::control::Control>>),
    SetFormat(io::Result<model::format::Format>),
    SetControl(io::Result<model::control::Control>),
}
