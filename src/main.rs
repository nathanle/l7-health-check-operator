extern crate serde_derive;
use kube::{api::{Api, ListParams, WatchEvent}, client::Client};
use k8s_openapi::api::core::v1::{Node, Pod};
use futures::{StreamExt, TryStreamExt};
use serde::{Serialize, Deserialize};
use kube_derive::CustomResource;
use schemars::JsonSchema;
use std::io::{stdout, Read, Write};
use std::net::TcpListener;
use port_check::*;
use std::net::*;
use std::time::Duration;
use std::collections::HashMap;
use crossbeam_utils::atomic::AtomicCell;


#[derive(Default)]
struct IpAndPort<'a> {
    port_number: i32,
    ip_address: &'a str,
}

#[derive(CustomResource, Serialize, Deserialize, Default, Clone, Debug, JsonSchema)]
#[kube(group = "health-check.demo.com", version = "v1", kind="NodeCheck", namespaced)]
#[allow(non_snake_case)]
pub struct HealthCheckSpec {
  pub interval: i64,
  pub port: i64,
}

async fn check_port(ip_address: String, port_number: i32) {
    println!("{}", ip_address);
    let addr = format!("{}:{}", ip_address.to_string(), port_number);
    println!("{}", addr);
    let free_port = free_local_port().unwrap();
    let is_reachable = is_port_reachable(addr);
    println!("Reachable: {}", is_reachable);
}

async fn check_pod(target_node_name: String) {
    let mut t: i32 = 0;
    let mut s: String = "".to_string();
    let client3 = Client::try_default().await;
    let pods: Api<Pod> = Api::namespaced(client3.expect("Pod AP call failed"), "default");
    let pod_list = pods.list(&ListParams::default()).await.unwrap();
    let filtered_pods: Vec<Pod> = pod_list
        .items
        .into_iter()
        .filter(|p| {
            if let Some(spec) = &p.spec {
                spec.node_name.as_deref() == Some(&target_node_name)
            } else {
                false
            }
        })
        .collect();
    println!(
        "\nFound {} pods on node '{}':",
        filtered_pods.len(),
        target_node_name
    );
    for p in filtered_pods {
        println!("  - {}", p.metadata.name.as_ref().unwrap());
           if let Some(spec) = p.spec {
               //println!("{:#?}", spec.containers);
            for container in spec.containers {
                if let Some(ports) = container.ports {
                    for port in ports {
                        println!("  Container: {}, Port: {}", container.name, port.container_port);
                        t = port.container_port;
                    }
                }
                if let Some(ref ips) = p.metadata.annotations {
                    //Need some error handling.
                    let i  = ips.get("cni.projectcalico.org/podIP").unwrap().strip_suffix("/32").unwrap();
                    s = i.to_string();
                    println!("  IP: {}", &s);
                }
            let _ = tokio::spawn(check_port(s.clone(), t));
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
                let node_name = node.metadata.name.clone().unwrap_or_default();
                println!("Node Added: {}", node_name);
                let _ = tokio::spawn(check_pod(node_name));
            }
            WatchEvent::Modified(node) => {
                let node_name = node.metadata.name.clone().unwrap_or_default();
                println!("Node Modified: {}", node_name);
                let _ = tokio::spawn(check_pod(node_name));
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
    println!("subscribing events of type health-check.demo.com/v1");
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
