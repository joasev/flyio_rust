use rustengan::*;

use std::{collections::HashMap, io::{StdoutLock, Write}};
use serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Broadcast { message: i32 },
    BroadcastOk,
    Read,
    ReadOk { messages: Vec<i32> },
    Topology { topology: HashMap<String, Vec<String>> },
    TopologyOk,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<i32>,
    topology: HashMap<String, Vec<String>>,
}
impl Node<(), Payload> for BroadcastNode {
    fn from_init(_state: (), init: Init) -> anyhow::Result<Self> {
        Ok(BroadcastNode {
            node: init.node_id,
            id: 1,
            messages: Vec::new(),
            topology: HashMap::new(),
        })
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {

        match input.body.payload {
            Payload::Broadcast { message } => {

                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body { 
                        id: Some(self.id), 
                        in_reply_to: input.body.id, 
                        payload: Payload::BroadcastOk
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to broadcast")?;
                output.write_all(b"\n")
                    .context("write trailing newline")?;
                self.id += 1;

                if !self.messages.contains(&message) {
                    self.messages.push(message);

                    if let Some(my_nodes) = self.topology.get(&self.node) {
                        for node in my_nodes {
                            let reply = Message {
                                src: self.node.clone(),
                                dst: node.clone(),
                                body: Body { 
                                    id: Some(self.id), 
                                    in_reply_to: None, 
                                    payload: Payload::Broadcast { message }
                                },
                            };
                            serde_json::to_writer(&mut *output, &reply)
                                .context("serialize response to broadcast")?;
                            output.write_all(b"\n")
                                .context("write trailing newline")?;
                            self.id += 1;
                        }
                    }
                } 
            }
            Payload::BroadcastOk => {}
            Payload::Read => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body { 
                        id: Some(self.id), 
                        in_reply_to: input.body.id, 
                        payload: Payload::ReadOk { messages: self.messages.clone() }
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to echo")?;
                output.write_all(b"\n")
                    .context("write trailing newline")?;
                self.id += 1;
            }
            Payload::ReadOk { .. } => {}
            Payload::Topology { topology } => {
                self.topology = topology;
                // Responds, but doesn't do anything with the data received
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body { 
                        id: Some(self.id), 
                        in_reply_to: input.body.id, 
                        payload: Payload::TopologyOk 
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to echo")?;
                output.write_all(b"\n")
                    .context("write trailing newline")?;
                self.id += 1;
            }
            Payload::TopologyOk => {}
        }
        
        Ok(())
    }
    
}

fn main () -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}

// Run command!
// ../maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10
// ../maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast --node-count 1 --time-limit 20 --rate 10
// ../maelstrom/maelstrom test -w broadcast --bin target/debug/broadcast-2 --node-count 5 --time-limit 20 --rate 10




