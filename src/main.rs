use mini_redis::{Connection, Frame};
use mini_redis::Command::{self, Get, Set};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use bytes::Bytes;
use std::sync::{Arc, Mutex};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

#[tokio::main]
async fn main() {
	let lis = TcpListener::bind("127.0.0.1:6379").await.unwrap();
	println!("Listening...");
	let db = Arc::new(Mutex::new(HashMap::new()));
	loop {
		let (s, _) = lis.accept().await.unwrap();
		let db = db.clone();
		println!("Accepted");
		tokio::spawn(async move {
			process(s,db).await;
		});
	}
}

async fn process(s: TcpStream,db: Db) {
	let mut conn = Connection::new(s);
	while let Some(frame) = conn.read_frame().await.unwrap() {
		let res = match Command::from_frame(frame).unwrap() {
			Set(cmd) => {
				let mut db = db.lock().unwrap();
				db.insert(cmd.key().to_string(), cmd.value().clone());
				Frame::Simple("OK".to_string())
			}
			Get(cmd) => {
				let db = db.lock().unwrap();
				if let Some(value) = db.get(cmd.key()) {
					Frame::Bulk(value.clone().into())
				} else {
					Frame::Null
				}
			}
			cmd => panic!("Uni {:?}", cmd),
	};
	conn.write_frame(&res).await.unwrap();
}
}
