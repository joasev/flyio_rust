use rustengan::*;

use std::io::{StdoutLock, Write};
use serde::{Deserialize, Serialize};
use anyhow::Context;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    id: usize,
}
impl Node<(), Payload> for EchoNode {
    fn from_init(_state: (), _init: rustengan::Init) -> anyhow::Result<Self> {
        Ok(EchoNode { id: 1 })
    }

    fn step(
        &mut self,
        input: Message<Payload>,
        output: &mut StdoutLock, //&mut serde_json::Serializer<StdoutLock, PrettyFormatter>
    ) -> anyhow::Result<()> {

        match input.body.payload {
            Payload::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body { 
                        id: Some(self.id), 
                        in_reply_to: input.body.id, 
                        payload: Payload::EchoOk { echo }
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to echo")?;
                output.write_all(b"\n")
                    .context("write trailing newline")?;
                self.id += 1;
            }
            Payload::EchoOk { echo } => {}
        }
        
        Ok(())
    }
    
}

fn main () -> anyhow::Result<()> {
    main_loop::<_, EchoNode, _>(())
}

// Run command!
// ../maelstrom/maelstrom test -w echo --bin target/debug/echo --node-count 1 --time-limit 10


// ERROR. FIJARSE CODIGO DEL PIBE !!
// Commit: "Move init logic inot lib" & "Add solution to unique-ids challenge
// 1.30.00"