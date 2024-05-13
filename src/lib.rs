use std::io::{BufRead, StdoutLock, Write};
use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<Payload> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,
    pub in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum InitPayload {
    Init(Init),
    InitOk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init { 
    pub node_id: String,
    pub node_ids: Vec<String>,
}


pub trait Node<S, Payload> {
    fn from_init(state: S, init: Init) -> anyhow::Result<Self> where Self: Sized;
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()>;
}


pub fn main_loop<S, N, P> (init_state: S) -> anyhow::Result<()> 
where
    P: DeserializeOwned,
    N: Node<S, P>,
{



    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut stdout = std::io::stdout().lock();

    /* 
    let init_msg: Message<InitPayload> = 
        // This from_reader waits for end of file ! // EXPLAINED 1.25.54 (plus chat question)
        // serde_json::from_reader(&mut stdin)
        serde_json::Deserializer::from_reader(&mut stdin)
            .into_iter::<Message<InitPayload>>()
            .next()
            .expect("no init message received first")
            .context("init message could not be deserialized")?; 
    */

    let init_msg: Message<InitPayload> = serde_json::from_str(
        &stdin
            .next()
            .expect("no init message received")
            .context("failed to read init message from stdin")?,
    )
    .context("init message could not be deserialized")?;

    let InitPayload::Init(init) = init_msg.body.payload else {
        panic!("first message should be init");
    };
    let mut node: N = Node::from_init(init_state, init).context("node initialization failed")?;

    let reply = Message {
        src: init_msg.dst,
        dst: init_msg.src,
        body: Body { 
            id: Some(0), 
            in_reply_to: init_msg.body.id, 
            payload: InitPayload::InitOk,
        },
    };
    serde_json::to_writer(&mut stdout, &reply)
        .context("serialize response to init")?;
    stdout.write_all(b"\n")
        .context("write trailing newline")?;


    for line in stdin {
        let line = line.context("Maelstrom input from STDIN could not be read")?;
        let input = serde_json::from_str(&line)
            .context(format!("Maelstrom input from STDIN could not be deserialized {}", &line))?;
        node
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }
    
    Ok(())

}

// Explicacion 1.11.00
// Explicacion 1.19.00

// Run command!
// ../maelstrom/maelstrom test -w echo --bin target/debug/rustengan --node-count 1 --time-limit 10