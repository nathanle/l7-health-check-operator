extern crate serde_derive;
use kube::{api::{Api, ListParams, WatchEvent}, client::Client};
use k8s_openapi::api::core::v1::Node;
use futures::{StreamExt, TryStreamExt};
use serde::{Serialize, Deserialize};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use std::io::{stdout, Read, Write};
use std::net::TcpListener;


#[derive(CustomResource, Serialize, Deserialize, Default, Clone, Debug, JsonSchema)]
#[kube(group = "health-check.linode.com", version = "v1", kind="NodeCheck", namespaced)]
#[allow(non_snake_case)]
pub struct HealthCheckSpec {
  pub interval: i64,
  pub port: i64,
}


async fn port_check(port: i64) {
  let bind_host = "localhost";
  let addr = format!("{}:{}", bind_host, port);
  let listener = TcpListener::bind(addr.as_str()).unwrap();
  for stream in listener.incoming() {
    match stream {
      Ok(mut s) => {
        println!("Connection accepted");
    
        let mut buf = [0; 128];
        let read_bytes = s.read(&mut buf).unwrap();
        stdout().write(&buf[0..read_bytes]).unwrap();
      }
      Err(e) => {
        println!("Error while accepting incoming connection - {}", e);
      }
    }
  }
}

async fn node_watch() {
    let client2 = Client::try_default().await;
    let nds: Api<Node> = Api::all(client2.expect("Node client connection failed"));
    let mut watcher = nds.watch(&Default::default(), "0").await.unwrap().boxed();
    while let Some(event) = watcher.try_next().await.expect("failed") {
        match event {
            WatchEvent::Added(node) => {
                println!("Node Added: {}", node.metadata.name.unwrap_or_default());
            }
            WatchEvent::Modified(_node) => {
            },
            WatchEvent::Deleted(node) => {
              println!("Deleted {}", node.metadata.name.unwrap());
            },
            WatchEvent::Error(member) => println!("error: {}", member),
            _ => {}
        }
    }
}

async fn health_check_watch() {
    let client = Client::try_default().await.expect("cannot infer client config");

    let nodecheck: Api<NodeCheck> = Api::namespaced(client, "default");
    let lp = ListParams::default();
    println!("{:#?}", &lp);
    println!("subscribing events of type health-check.linode.com/v1");
    let mut stream = nodecheck.watch(&lp, "0").await.unwrap().boxed();
    while let Some(status) = stream.try_next().await.expect("watch stream failed") {
        match status {
            WatchEvent::Added(nodecheck) => {
                println!("Check applied: {}", nodecheck.metadata.name.unwrap_or_default());
            },
            WatchEvent::Modified(nodecheck) => {
                println!("Check modifiied: {}", nodecheck.metadata.name.unwrap_or_default());
                println!("Port: {}", nodecheck.spec.port);
                println!("Interval: {}", nodecheck.spec.interval);
            },
            WatchEvent::Deleted(nodecheck) => {
                println!("Check deleted: {}", nodecheck.metadata.name.unwrap_or_default());
            },
            WatchEvent::Error(nodecheck) => println!("error: {}", nodecheck),
            _ => {}
        }
    }
}


#[tokio::main]
async fn main() {
    println!("starting hello world operator");
    let n = tokio::spawn(node_watch());
    let h = tokio::spawn(health_check_watch());
    let _ = n.await;
    let _ = h.await;
    println!("done");
}
