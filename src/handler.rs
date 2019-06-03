use ws::{Result, Handler, Sender, Message, Handshake, CloseCode};
use crossbeam::channel::Sender as ThreadOut;
use ws::{ErrorKind, Error};
use crate::{Hash, utils::hexstr_to_hash};

pub struct Getter {
    /// A representation of the output of the WebSocket connection.
    pub output: Sender,
    /// The json request data which is formatted string type.
    pub request: String,
    /// The sending side of a channel.
    pub result: ThreadOut<String>,
}

impl Handler for Getter {
    /// Called when the WebSocket handshake is successful and the connection is open for sending
    /// and receiving messages.
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.output.send(self.request.clone())
            .map_err(|_| Error::new(ErrorKind::Internal, "must connect"))?;

        Ok(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let txt = msg.as_text()?;
        let value: serde_json::Value = serde_json::from_str(txt)
            .map_err(|_| Error::new(ErrorKind::Internal, "Request deserialization is infallible; qed"))?;

        // println!("value: {:?}", value);

        let hex_str = match value["result"].as_str() {
            Some(res) => res.to_string(),
            None => "0x00".to_string(),
            // None => return Err(Error::new(ErrorKind::Internal, "No result in the storage key of the module")),
        };

        self.result.send(hex_str)
            .map_err(|_| Error::new(ErrorKind::Internal, "must run"))?;
        self.output.close(CloseCode::Normal)?;
        Ok(())
    }
}

pub struct HeightGetter {
    /// A representation of the output of the WebSocket connection.
    pub output: Sender,
    /// The json request data which is formatted string type.
    pub request: String,
    /// The sending side of a channel.
    pub result: ThreadOut<String>,
}

impl Handler for HeightGetter {
    /// Called when the WebSocket handshake is successful and the connection is open for sending
    /// and receiving messages.
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.output.send(self.request.clone())
            .map_err(|_| Error::new(ErrorKind::Internal, "must connect"))?;

        Ok(())
    }

    /// Called on incoming messages.
    fn on_message(&mut self, msg: Message) -> Result<()> {
        let txt = msg.as_text()?;
        let value: serde_json::Value = serde_json::from_str(txt)
            .map_err(|_| Error::new(ErrorKind::Internal, "Request deserialization is infallible; qed"))?;

        let hex_str = match value["result"]["number"].as_str() {
            Some(res) => res.to_string(),
            None => "0x00".to_string(),
            // None => return Err(Error::new(ErrorKind::Internal, "No result in the storage key of the module")),
        };

        self.result.send(hex_str)
            .map_err(|_| Error::new(ErrorKind::Internal, "must run"))?;
        self.output.close(CloseCode::Normal)?;
        Ok(())
    }
}

pub struct Submitter {
    pub output: Sender,
    pub request: String,
    pub result: ThreadOut<Hash>,
}

impl Handler for Submitter {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.output.send(self.request.clone())
            .map_err(|_| Error::new(ErrorKind::Internal, "must connect"))?;

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let txt = msg.as_text()?;
        let value: serde_json::Value = serde_json::from_str(txt)
            .map_err(|_| Error::new(ErrorKind::Internal, "Request deserialization is infallible; qed"))?;

        match value["id"].as_str() {
            Some(id) => {
                match id.parse::<u32>() {
                    Ok(_req) => {
                        match value.get("error") {
                            Some(err) => {
                                println!("(A)Response: {}", value);
                                println!("Error: {:?}", err);
                                self.output.close(CloseCode::Invalid)?;
                            },
                            None => println!("(B)Response: {:?}", value),
                        }
                    },
                    Ok(_) => {
                        println!("(Unknown request id) Response: {}", value);
                        self.output.close(CloseCode::Invalid)?;
                    },
                    Err(_) => {
                        println!("(Error assigning request id) Response: {}", value);
                        self.output.close(CloseCode::Invalid)?;
                    },
                }
            },
            None => {
                match value["method"].as_str() {
                    Some("author_extrinsicUpdate") => {
                        match value["params"]["result"].as_str() {
                            Some(_res) => {
                                println!("(E)Response: {}", value);
                            },
                            None => {
                                self.result.send(hexstr_to_hash(value["params"]["result"]["finalized"].as_str().unwrap().to_string()))
                                    .map_err(|_| Error::new(ErrorKind::Internal, "must connect"))?;

                                self.output.close(CloseCode::Normal)?;
                                println!("Finalized extrinsic hash: {:?}", hexstr_to_hash(value["params"]["result"]["finalized"].as_str().unwrap().to_string()));
                            },
                        }
                    },
                    Some(_) => {
                        println!("(Unsupported method) Response: {}", value);
                        self.output.close(CloseCode::Invalid)?;
                    },
                    None => {
                        println!("(No method in response) Response: {}", value);
                        self.output.close(CloseCode::Invalid)?;
                    },
                }
            }
        };

        Ok(())
    }
}