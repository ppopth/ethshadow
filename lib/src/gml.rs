use std::fmt;
use std::fmt::Write;
use std::time::Duration;

pub struct Gml<W: Write> {
    write: W,
    nodes: u64,
    finished: bool,
}

#[derive(Copy, Clone, Debug)]
pub struct NetworkNode {
    id: u64,
}

impl<W: Write> Gml<W> {
    pub fn new(mut write: W, directed: bool) -> Result<Self, fmt::Error> {
        writeln!(write, "graph [")?;
        if directed {
            writeln!(write, "  directed 1")?;
        }
        Ok(Self {
            write,
            nodes: 0,
            finished: false,
        })
    }

    pub fn add_node(
        &mut self,
        bandwidth_up: &str,
        bandwidth_down: &str,
        label: Option<&str>,
    ) -> Result<NetworkNode, fmt::Error> {
        let id = self.nodes;
        self.nodes += 1;
        writeln!(self.write, "  node [")?;
        writeln!(self.write, "    id {id}")?;
        if let Some(label) = label {
            writeln!(self.write, "    label \"{label}\"")?;
        }
        writeln!(self.write, "    host_bandwidth_up \"{bandwidth_up}\"")?;
        writeln!(self.write, "    host_bandwidth_down \"{bandwidth_down}\"")?;
        writeln!(self.write, "  ]")?;

        Ok(NetworkNode { id })
    }

    pub fn add_edge(
        &mut self,
        from: NetworkNode,
        to: NetworkNode,
        latency: Duration,
        packet_loss: f32,
        label: Option<&str>,
    ) -> Result<(), fmt::Error> {
        writeln!(self.write, "  edge [")?;
        writeln!(self.write, "    source {}", from.id)?;
        writeln!(self.write, "    target {}", to.id)?;
        if let Some(label) = label {
            writeln!(self.write, "    label \"{label}\"")?;
        }
        writeln!(self.write, "    latency \"{} ns\"", latency.as_nanos())?;
        writeln!(self.write, "    packet_loss {packet_loss:.3}")?;
        writeln!(self.write, "  ]")
    }

    pub fn finish(mut self) -> Result<(), fmt::Error> {
        self.finished = true;
        write!(self.write, "]")
    }
}

impl<W: Write> Drop for Gml<W> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = write!(self.write, "]");
        }
    }
}

impl NetworkNode {
    pub fn id(self) -> u64 {
        self.id
    }
}
