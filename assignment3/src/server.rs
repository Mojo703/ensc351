use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub struct NodeProcess {
    child: Child,
}

impl NodeProcess {
    pub fn new() -> io::Result<Self> {
        let project_dir = Path::new("server");

        let child = Command::new("node")
            .arg("server.js")
            .current_dir(project_dir)
            // Suppress stdout and stderr
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        Ok(Self { child })
    }

    pub fn end(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
